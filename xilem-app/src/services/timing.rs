// High-precision timing service for response time measurement

use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PrecisionTimer {
    start: Instant,
    marks: Vec<(String, Duration)>,
    splits: HashMap<String, Duration>,
}

impl PrecisionTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            marks: Vec::new(),
            splits: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.marks.clear();
        self.splits.clear();
    }

    pub fn mark(&mut self, label: impl Into<String>) {
        let elapsed = self.start.elapsed();
        self.marks.push((label.into(), elapsed));
    }

    pub fn split(&mut self, label: impl Into<String>) -> Duration {
        let elapsed = self.start.elapsed();
        let label = label.into();
        self.splits.insert(label.clone(), elapsed);
        elapsed
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn get_elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn get_elapsed_micros(&self) -> u128 {
        self.start.elapsed().as_micros()
    }

    pub fn get_elapsed_nanos(&self) -> u128 {
        self.start.elapsed().as_nanos()
    }

    pub fn get_split(&self, label: &str) -> Option<Duration> {
        self.splits.get(label).copied()
    }

    pub fn get_marks(&self) -> &[(String, Duration)] {
        &self.marks
    }

    pub fn get_last_mark(&self) -> Option<&(String, Duration)> {
        self.marks.last()
    }

    pub fn get_mark_delta(&self, from: &str, to: &str) -> Option<Duration> {
        let from_time = self.marks.iter().find(|(l, _)| l == from)?.1;
        let to_time = self.marks.iter().find(|(l, _)| l == to)?.1;
        
        if to_time > from_time {
            Some(to_time - from_time)
        } else {
            None
        }
    }
}

impl Default for PrecisionTimer {
    fn default() -> Self {
        Self::new()
    }
}

// Task-specific timer for measuring response times
#[derive(Debug, Clone)]
pub struct TaskTimer {
    timer: PrecisionTimer,
    task_started: bool,
    first_input_time: Option<Duration>,
    hint_requested_time: Option<Duration>,
}

impl TaskTimer {
    pub fn new() -> Self {
        Self {
            timer: PrecisionTimer::new(),
            task_started: false,
            first_input_time: None,
            hint_requested_time: None,
        }
    }

    pub fn start_task(&mut self) {
        self.timer.reset();
        self.task_started = true;
        self.first_input_time = None;
        self.hint_requested_time = None;
    }

    pub fn mark_first_input(&mut self) {
        if self.task_started && self.first_input_time.is_none() {
            self.first_input_time = Some(self.timer.get_elapsed());
        }
    }

    pub fn mark_hint_requested(&mut self) {
        if self.task_started && self.hint_requested_time.is_none() {
            self.hint_requested_time = Some(self.timer.get_elapsed());
        }
    }

    pub fn get_response_time_ms(&self) -> u128 {
        self.timer.get_elapsed_ms()
    }

    pub fn get_response_time_micros(&self) -> u128 {
        self.timer.get_elapsed_micros()
    }

    pub fn get_first_input_delay_ms(&self) -> Option<u128> {
        self.first_input_time.map(|d| d.as_millis())
    }

    pub fn get_hint_delay_ms(&self) -> Option<u128> {
        self.hint_requested_time.map(|d| d.as_millis())
    }

    pub fn stop_task(&mut self) -> TaskTimingResult {
        let result = TaskTimingResult {
            total_time_ms: self.timer.get_elapsed_ms(),
            total_time_micros: self.timer.get_elapsed_micros(),
            first_input_delay_ms: self.get_first_input_delay_ms(),
            hint_requested_delay_ms: self.get_hint_delay_ms(),
        };
        
        self.task_started = false;
        result
    }
}

impl Default for TaskTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TaskTimingResult {
    pub total_time_ms: u128,
    pub total_time_micros: u128,
    pub first_input_delay_ms: Option<u128>,
    pub hint_requested_delay_ms: Option<u128>,
}

// Session-level timer for tracking overall session metrics
#[derive(Debug, Clone)]
pub struct SessionTimer {
    timer: PrecisionTimer,
    task_timers: Vec<TaskTimingResult>,
    pause_start: Option<Instant>,
    total_pause_duration: Duration,
    session_started: bool,
}

impl SessionTimer {
    pub fn new() -> Self {
        Self {
            timer: PrecisionTimer::new(),
            task_timers: Vec::new(),
            pause_start: None,
            total_pause_duration: Duration::ZERO,
            session_started: false,
        }
    }

    pub fn start_session(&mut self) {
        self.timer.reset();
        self.task_timers.clear();
        self.pause_start = None;
        self.total_pause_duration = Duration::ZERO;
        self.session_started = true;
    }

    pub fn add_task_result(&mut self, result: TaskTimingResult) {
        self.task_timers.push(result);
    }

    pub fn pause(&mut self) {
        if self.session_started && self.pause_start.is_none() {
            self.pause_start = Some(Instant::now());
        }
    }

    pub fn resume(&mut self) {
        if let Some(pause_start) = self.pause_start.take() {
            self.total_pause_duration += pause_start.elapsed();
        }
    }

    pub fn get_active_time(&self) -> Duration {
        let total = self.timer.get_elapsed();
        let current_pause = self.pause_start.map(|p| p.elapsed()).unwrap_or(Duration::ZERO);
        total - self.total_pause_duration - current_pause
    }

    pub fn get_active_time_ms(&self) -> u128 {
        self.get_active_time().as_millis()
    }

    pub fn get_total_time_ms(&self) -> u128 {
        self.timer.get_elapsed_ms()
    }

    pub fn get_pause_time_ms(&self) -> u128 {
        let current_pause = self.pause_start.map(|p| p.elapsed()).unwrap_or(Duration::ZERO);
        (self.total_pause_duration + current_pause).as_millis()
    }

    pub fn get_average_response_time_ms(&self) -> Option<f64> {
        if self.task_timers.is_empty() {
            return None;
        }
        
        let total: u128 = self.task_timers.iter().map(|t| t.total_time_ms).sum();
        Some(total as f64 / self.task_timers.len() as f64)
    }

    pub fn get_median_response_time_ms(&self) -> Option<u128> {
        if self.task_timers.is_empty() {
            return None;
        }
        
        let mut times: Vec<u128> = self.task_timers.iter().map(|t| t.total_time_ms).collect();
        times.sort_unstable();
        
        let mid = times.len() / 2;
        if times.len() % 2 == 0 {
            Some((times[mid - 1] + times[mid]) / 2)
        } else {
            Some(times[mid])
        }
    }

    pub fn get_task_count(&self) -> usize {
        self.task_timers.len()
    }

    pub fn stop_session(&mut self) -> SessionTimingResult {
        SessionTimingResult {
            total_time_ms: self.get_total_time_ms(),
            active_time_ms: self.get_active_time_ms(),
            pause_time_ms: self.get_pause_time_ms(),
            task_count: self.task_timers.len(),
            average_response_time_ms: self.get_average_response_time_ms(),
            median_response_time_ms: self.get_median_response_time_ms(),
            task_results: self.task_timers.clone(),
        }
    }
}

impl Default for SessionTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SessionTimingResult {
    pub total_time_ms: u128,
    pub active_time_ms: u128,
    pub pause_time_ms: u128,
    pub task_count: usize,
    pub average_response_time_ms: Option<f64>,
    pub median_response_time_ms: Option<u128>,
    pub task_results: Vec<TaskTimingResult>,
}

// Utility functions for timing analysis
pub fn calculate_reaction_time_percentiles(times: &[u128]) -> PercentileResult {
    if times.is_empty() {
        return PercentileResult::default();
    }
    
    let mut sorted = times.to_vec();
    sorted.sort_unstable();
    
    let len = sorted.len();
    let p25_idx = len / 4;
    let p50_idx = len / 2;
    let p75_idx = (3 * len) / 4;
    let p90_idx = (9 * len) / 10;
    let p95_idx = (19 * len) / 20;
    let p99_idx = (99 * len) / 100;
    
    PercentileResult {
        min: *sorted.first().unwrap(),
        p25: sorted[p25_idx],
        p50: sorted[p50_idx],
        p75: sorted[p75_idx],
        p90: sorted.get(p90_idx).copied().unwrap_or(sorted[len - 1]),
        p95: sorted.get(p95_idx).copied().unwrap_or(sorted[len - 1]),
        p99: sorted.get(p99_idx).copied().unwrap_or(sorted[len - 1]),
        max: *sorted.last().unwrap(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct PercentileResult {
    pub min: u128,
    pub p25: u128,
    pub p50: u128,
    pub p75: u128,
    pub p90: u128,
    pub p95: u128,
    pub p99: u128,
    pub max: u128,
}