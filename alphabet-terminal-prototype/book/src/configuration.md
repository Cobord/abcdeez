# Configuration Guide

This guide covers all configuration options for the alphabet-terminal-prototype system, including runtime parameters, environment variables, and configuration files.

## Configuration File

### Default Configuration

```toml
# config.toml

[system]
# Computational parameters
monte_carlo_samples = 1000
parallel_threads = 8
cache_size_mb = 256

[learner]
# Learning parameters
initial_learning_rate = 0.1
initial_forgetting_rate = 0.01
min_memory_strength = 0.0
max_memory_strength = 1.0

# Response time model
rt_mu_init = 1000.0      # milliseconds
rt_sigma_init = 200.0
rt_tau_init = 300.0

[scheduler]
# Adaptive scheduling
strategy = "maximize_eig"  # Options: maximize_eig, epsilon_greedy, ucb, thompson
epsilon = 0.1              # For epsilon-greedy
exploration_weight = 2.0   # For UCB

# Task selection constraints
min_task_difficulty = 0.1
max_task_difficulty = 0.9
min_spacing_minutes = 1
max_session_minutes = 30

[bayesian]
# Prior parameters
prior_mean = 0.0
prior_variance = 1.0
observation_noise = 0.1

# MCMC settings
mcmc_chains = 4
mcmc_iterations = 10000
mcmc_burnin = 5000
mcmc_thin = 10

[hints]
# Hint generation
min_struggle_time_seconds = 30
max_struggle_time_seconds = 120
error_threshold = 3
hint_fade_rate = 0.1

[export]
# Data export
format = "json"  # Options: json, csv, parquet, sqlite
compression = "gzip"  # Options: none, gzip, zstd
anonymize = false
buffer_size = 1000
```

### Loading Configuration

```rust
use config::{Config, File, Environment};

pub struct AppConfig {
    pub system: SystemConfig,
    pub learner: LearnerConfig,
    pub scheduler: SchedulerConfig,
    pub bayesian: BayesianConfig,
    pub hints: HintConfig,
    pub export: ExportConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::builder()
            // Start with default configuration
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            
            // Layer on local configuration file
            .add_source(File::with_name("config").required(false))
            
            // Layer on environment-specific configuration
            .add_source(File::with_name(&format!("config.{}", 
                env::var("ENV").unwrap_or_else(|_| "development".to_string())))
                .required(false))
            
            // Layer on environment variables (with prefix ALPHABET_)
            .add_source(Environment::with_prefix("ALPHABET"))
            
            .build()?;
        
        config.try_deserialize()
    }
}
```

## Environment Variables

Override configuration via environment variables:

```bash
# System configuration
export ALPHABET_SYSTEM_MONTE_CARLO_SAMPLES=5000
export ALPHABET_SYSTEM_PARALLEL_THREADS=16

# Learner configuration
export ALPHABET_LEARNER_INITIAL_LEARNING_RATE=0.2
export ALPHABET_LEARNER_RT_MU_INIT=800

# Scheduler configuration
export ALPHABET_SCHEDULER_STRATEGY="thompson"
export ALPHABET_SCHEDULER_MAX_SESSION_MINUTES=45

# Bayesian configuration
export ALPHABET_BAYESIAN_PRIOR_VARIANCE=2.0
export ALPHABET_BAYESIAN_MCMC_ITERATIONS=20000

# Export configuration
export ALPHABET_EXPORT_FORMAT="parquet"
export ALPHABET_EXPORT_ANONYMIZE=true
```

## Runtime Configuration

### Dynamic Parameter Updates

```rust
pub struct RuntimeConfig {
    params: Arc<RwLock<ConfigParams>>,
    update_channel: Sender<ConfigUpdate>,
}

impl RuntimeConfig {
    pub fn update_param(&self, key: &str, value: ConfigValue) -> Result<(), ConfigError> {
        let mut params = self.params.write().unwrap();
        
        // Validate parameter
        self.validate_param(key, &value)?;
        
        // Update parameter
        params.set(key, value.clone());
        
        // Notify subscribers
        self.update_channel.send(ConfigUpdate { key: key.to_string(), value })?;
        
        Ok(())
    }
    
    pub fn watch<F>(&self, key: &str, callback: F) 
    where 
        F: Fn(ConfigValue) + Send + 'static 
    {
        let mut receiver = self.update_channel.subscribe();
        let key = key.to_string();
        
        tokio::spawn(async move {
            while let Ok(update) = receiver.recv().await {
                if update.key == key {
                    callback(update.value);
                }
            }
        });
    }
}
```

## Command-Line Arguments

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Configuration file path
    #[clap(short, long, default_value = "config.toml")]
    pub config: PathBuf,
    
    /// Learning rate override
    #[clap(long)]
    pub learning_rate: Option<f64>,
    
    /// Number of Monte Carlo samples
    #[clap(long)]
    pub mc_samples: Option<usize>,
    
    /// Scheduling strategy
    #[clap(long, possible_values = &["maximize_eig", "epsilon_greedy", "ucb", "thompson"])]
    pub strategy: Option<String>,
    
    /// Export format
    #[clap(long, possible_values = &["json", "csv", "parquet", "sqlite"])]
    pub export_format: Option<String>,
    
    /// Enable debug mode
    #[clap(short, long)]
    pub debug: bool,
    
    /// Verbosity level
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,
}

impl Args {
    pub fn apply_to_config(&self, config: &mut AppConfig) {
        if let Some(lr) = self.learning_rate {
            config.learner.initial_learning_rate = lr;
        }
        
        if let Some(samples) = self.mc_samples {
            config.system.monte_carlo_samples = samples;
        }
        
        if let Some(ref strategy) = self.strategy {
            config.scheduler.strategy = strategy.parse().unwrap();
        }
        
        if let Some(ref format) = self.export_format {
            config.export.format = format.parse().unwrap();
        }
    }
}
```

## Profile-Based Configuration

### Multiple Profiles

```toml
# profiles.toml

[profile.beginner]
learner.initial_learning_rate = 0.05
scheduler.min_task_difficulty = 0.1
scheduler.max_task_difficulty = 0.5
hints.min_struggle_time_seconds = 15

[profile.intermediate]
learner.initial_learning_rate = 0.1
scheduler.min_task_difficulty = 0.3
scheduler.max_task_difficulty = 0.8
hints.min_struggle_time_seconds = 30

[profile.advanced]
learner.initial_learning_rate = 0.15
scheduler.min_task_difficulty = 0.5
scheduler.max_task_difficulty = 1.0
hints.min_struggle_time_seconds = 60

[profile.research]
system.monte_carlo_samples = 10000
bayesian.mcmc_iterations = 50000
export.format = "parquet"
export.anonymize = true
```

### Profile Selection

```rust
impl AppConfig {
    pub fn with_profile(profile: &str) -> Result<Self, ConfigError> {
        let mut config = Self::load()?;
        
        let profile_config = Config::builder()
            .add_source(File::with_name("profiles").required(false))
            .build()?;
        
        if let Ok(profile_settings) = profile_config.get_table(&format!("profile.{}", profile)) {
            config.merge_settings(profile_settings)?;
        }
        
        Ok(config)
    }
}
```

## Validation

### Parameter Validation

```rust
pub struct ConfigValidator {
    rules: Vec<ValidationRule>,
}

impl ConfigValidator {
    pub fn validate(&self, config: &AppConfig) -> Result<(), ValidationError> {
        // Range checks
        self.check_range("learning_rate", config.learner.initial_learning_rate, 0.001, 1.0)?;
        self.check_range("forgetting_rate", config.learner.initial_forgetting_rate, 0.0, 0.1)?;
        
        // Consistency checks
        if config.scheduler.min_task_difficulty >= config.scheduler.max_task_difficulty {
            return Err(ValidationError::Inconsistent(
                "min_task_difficulty must be less than max_task_difficulty".to_string()
            ));
        }
        
        // Dependency checks
        if config.scheduler.strategy == SchedulingStrategy::EpsilonGreedy {
            self.check_range("epsilon", config.scheduler.epsilon, 0.0, 1.0)?;
        }
        
        Ok(())
    }
    
    fn check_range(&self, name: &str, value: f64, min: f64, max: f64) -> Result<(), ValidationError> {
        if value < min || value > max {
            return Err(ValidationError::OutOfRange {
                parameter: name.to_string(),
                value,
                min,
                max,
            });
        }
        Ok(())
    }
}
```

## Hot Reloading

```rust
pub struct ConfigWatcher {
    config_path: PathBuf,
    watcher: RecommendedWatcher,
    reload_handler: Arc<dyn Fn(AppConfig) + Send + Sync>,
}

impl ConfigWatcher {
    pub fn watch(config_path: PathBuf, handler: impl Fn(AppConfig) + Send + Sync + 'static) -> Result<Self, Error> {
        let handler = Arc::new(handler);
        let handler_clone = handler.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, Error>| {
            if let Ok(event) = res {
                if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any)) {
                    if let Ok(config) = AppConfig::load() {
                        handler_clone(config);
                    }
                }
            }
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        Ok(ConfigWatcher {
            config_path,
            watcher,
            reload_handler: handler,
        })
    }
}
```

## Presets

### Task Difficulty Presets

```rust
pub enum DifficultyPreset {
    VeryEasy,
    Easy,
    Medium,
    Hard,
    VeryHard,
    Adaptive,
}

impl DifficultyPreset {
    pub fn apply(&self, config: &mut SchedulerConfig) {
        match self {
            DifficultyPreset::VeryEasy => {
                config.min_task_difficulty = 0.0;
                config.max_task_difficulty = 0.3;
            }
            DifficultyPreset::Easy => {
                config.min_task_difficulty = 0.1;
                config.max_task_difficulty = 0.4;
            }
            DifficultyPreset::Medium => {
                config.min_task_difficulty = 0.3;
                config.max_task_difficulty = 0.7;
            }
            DifficultyPreset::Hard => {
                config.min_task_difficulty = 0.5;
                config.max_task_difficulty = 0.9;
            }
            DifficultyPreset::VeryHard => {
                config.min_task_difficulty = 0.7;
                config.max_task_difficulty = 1.0;
            }
            DifficultyPreset::Adaptive => {
                // Will be adjusted based on performance
                config.min_task_difficulty = 0.0;
                config.max_task_difficulty = 1.0;
            }
        }
    }
}
```

## Performance Tuning

### Optimization Profiles

```rust
pub enum OptimizationProfile {
    LowLatency,    // Minimize response time
    HighAccuracy,  // Maximum model accuracy
    Balanced,      // Balance speed and accuracy
    LowMemory,     // Minimize memory usage
}

impl OptimizationProfile {
    pub fn apply(&self, config: &mut SystemConfig) {
        match self {
            OptimizationProfile::LowLatency => {
                config.monte_carlo_samples = 100;
                config.parallel_threads = 1;
                config.cache_size_mb = 512;
            }
            OptimizationProfile::HighAccuracy => {
                config.monte_carlo_samples = 10000;
                config.parallel_threads = num_cpus::get();
                config.cache_size_mb = 1024;
            }
            OptimizationProfile::Balanced => {
                config.monte_carlo_samples = 1000;
                config.parallel_threads = num_cpus::get() / 2;
                config.cache_size_mb = 256;
            }
            OptimizationProfile::LowMemory => {
                config.monte_carlo_samples = 500;
                config.parallel_threads = 2;
                config.cache_size_mb = 64;
            }
        }
    }
}
```

## Logging Configuration

```toml
[logging]
level = "info"  # Options: trace, debug, info, warn, error
format = "json" # Options: text, json
output = "stdout" # Options: stdout, stderr, file
file_path = "alphabet.log"
max_file_size_mb = 100
max_files = 10

[logging.filters]
alphabet_terminal_prototype = "debug"
tokio = "warn"
hyper = "warn"
```

## Summary

Configuration options provide:
- **File-based config**: TOML configuration files
- **Environment variables**: Override any setting
- **Command-line args**: Runtime parameter changes
- **Profiles**: Predefined configurations
- **Validation**: Ensure valid parameters
- **Hot reloading**: Dynamic configuration updates
- **Presets**: Common configuration patterns
- **Performance tuning**: Optimization profiles

This flexible configuration system allows the alphabet-terminal-prototype to be adapted for different use cases, from research to production deployment.