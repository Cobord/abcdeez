use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Sample, SampleFormat, SampleRate, StreamConfig};
use hound::{WavSpec, WavWriter};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// Audio recorder using CPAL for cross-platform microphone capture
#[derive(Debug)]
pub struct AudioRecorder {
    host: Host,
    input_device: Option<Device>,
    recording_thread: Option<thread::JoinHandle<Result<(), AudioError>>>,
    is_recording: Arc<Mutex<bool>>,
    sample_rate: u32,
    channels: u16,
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub format: AudioFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    F32,
    I16,
    U16,
}

#[derive(Debug)]
pub enum AudioError {
    DeviceNotFound,
    ConfigNotSupported,
    StreamError(String),
    FileError(String),
    RecordingInProgress,
    NotRecording,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AudioError::DeviceNotFound => write!(f, "Audio input device not found"),
            AudioError::ConfigNotSupported => write!(f, "Audio configuration not supported"),
            AudioError::StreamError(e) => write!(f, "Audio stream error: {}", e),
            AudioError::FileError(e) => write!(f, "File error: {}", e),
            AudioError::RecordingInProgress => write!(f, "Recording already in progress"),
            AudioError::NotRecording => write!(f, "No recording in progress"),
        }
    }
}

impl std::error::Error for AudioError {}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1, // Mono for think-aloud protocols
            buffer_size: 4096,
            format: AudioFormat::F32,
        }
    }
}

impl AudioRecorder {
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        
        Ok(AudioRecorder {
            host,
            input_device: None,
            recording_thread: None,
            is_recording: Arc::new(Mutex::new(false)),
            sample_rate: 44100,
            channels: 1,
        })
    }

    pub fn initialize(&mut self, config: AudioConfig) -> Result<(), AudioError> {
        // Get the default input device
        let input_device = self.host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound)?;

        // Verify the device supports our desired configuration
        let supported_configs = input_device
            .supported_input_configs()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let mut compatible_config = None;
        for config_range in supported_configs {
            if config_range.channels() >= config.channels
                && config_range.min_sample_rate() <= SampleRate(config.sample_rate)
                && config_range.max_sample_rate() >= SampleRate(config.sample_rate)
            {
                compatible_config = Some(config_range.with_sample_rate(SampleRate(config.sample_rate)));
                break;
            }
        }

        let _stream_config = compatible_config.ok_or(AudioError::ConfigNotSupported)?;

        self.input_device = Some(input_device);
        self.sample_rate = config.sample_rate;
        self.channels = config.channels;

        Ok(())
    }

    pub fn start_recording(&mut self, output_path: PathBuf) -> Result<(), AudioError> {
        let mut is_recording = self.is_recording.lock().unwrap();
        if *is_recording {
            return Err(AudioError::RecordingInProgress);
        }

        let device = self.input_device.as_ref()
            .ok_or(AudioError::DeviceNotFound)?
            .clone();

        // Create a ring buffer for audio data
        let rb = RingBuffer::<f32>::new(48000 * 10); // 10 seconds buffer
        let (mut producer, consumer) = rb.split();

        // Build the input stream
        let config = StreamConfig {
            channels: self.channels,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let is_recording_clone = Arc::clone(&self.is_recording);
        
        // Create the input stream
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let is_rec = is_recording_clone.lock().unwrap();
                if *is_rec {
                    for &sample in data {
                        if producer.push(sample).is_err() {
                            // Buffer is full, skip this sample
                            break;
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Audio input stream error: {}", err);
            },
            None,
        ).map_err(|e| AudioError::StreamError(e.to_string()))?;

        // Start the stream
        stream.play().map_err(|e| AudioError::StreamError(e.to_string()))?;

        *is_recording = true;

        // Spawn a thread to write audio data to file
        let is_recording_thread = Arc::clone(&self.is_recording);
        let sample_rate = self.sample_rate;
        let channels = self.channels;
        
        let recording_thread = thread::spawn(move || -> Result<(), AudioError> {
            let spec = WavSpec {
                channels,
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };

            let mut writer = WavWriter::create(&output_path, spec)
                .map_err(|e| AudioError::FileError(e.to_string()))?;

            let mut consumer = consumer;
            let start_time = Instant::now();
            
            // Keep the stream alive
            let _stream = stream;

            while {
                let is_rec = is_recording_thread.lock().unwrap();
                *is_rec
            } {
                // Write samples from the ring buffer to the file
                let mut samples_written = 0;
                while let Some(sample) = consumer.pop() {
                    writer.write_sample(sample)
                        .map_err(|e| AudioError::FileError(e.to_string()))?;
                    samples_written += 1;
                    
                    // Process in batches to avoid holding the lock too long
                    if samples_written >= 1024 {
                        break;
                    }
                }

                // Small sleep to prevent busy waiting
                thread::sleep(Duration::from_millis(10));
            }

            // Finalize the WAV file
            writer.finalize()
                .map_err(|e| AudioError::FileError(e.to_string()))?;

            println!("Audio recording saved to: {:?}", output_path);
            println!("Recording duration: {:.2} seconds", start_time.elapsed().as_secs_f64());

            Ok(())
        });

        self.recording_thread = Some(recording_thread);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(), AudioError> {
        let mut is_recording = self.is_recording.lock().unwrap();
        if !*is_recording {
            return Err(AudioError::NotRecording);
        }

        *is_recording = false;
        drop(is_recording); // Release the lock

        // Wait for the recording thread to finish
        if let Some(thread) = self.recording_thread.take() {
            thread.join().unwrap_or_else(|_| {
                Err(AudioError::StreamError("Recording thread panicked".to_string()))
            })?;
        }

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }

    pub fn get_available_devices(&self) -> Vec<String> {
        self.host.input_devices()
            .map(|devices| {
                devices.filter_map(|device| device.name().ok()).collect()
            })
            .unwrap_or_default()
    }

    pub fn get_supported_configs(&self) -> Result<Vec<(u32, u16)>, AudioError> {
        let device = self.input_device.as_ref()
            .ok_or(AudioError::DeviceNotFound)?;

        let configs = device
            .supported_input_configs()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let mut result = Vec::new();
        for config in configs {
            let min_rate = config.min_sample_rate().0;
            let max_rate = config.max_sample_rate().0;
            let channels = config.channels();
            
            // Add common sample rates within the supported range
            for &rate in &[8000, 16000, 22050, 44100, 48000, 96000] {
                if rate >= min_rate && rate <= max_rate {
                    result.push((rate, channels));
                }
            }
        }

        result.sort();
        result.dedup();
        Ok(result)
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        if self.is_recording() {
            let _ = self.stop_recording();
        }
    }
}

/// Helper function to create audio file paths
pub fn create_audio_file_path(participant_id: &str, session_id: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("recording_{}_{}__{}.wav", participant_id, session_id, timestamp);
    
    // Create audio directory if it doesn't exist
    let audio_dir = PathBuf::from("audio_recordings");
    std::fs::create_dir_all(&audio_dir).unwrap_or_else(|e| {
        eprintln!("Failed to create audio directory: {}", e);
    });
    
    audio_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audio_recorder_creation() {
        let recorder = AudioRecorder::new();
        assert!(recorder.is_ok());
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_create_audio_file_path() {
        let path = create_audio_file_path("test_participant", "test_session");
        assert!(path.to_string_lossy().contains("recording_test_participant_test_session"));
        assert!(path.extension() == Some(std::ffi::OsStr::new("wav")));
    }
}