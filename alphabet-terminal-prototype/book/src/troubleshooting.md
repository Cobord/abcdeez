# Troubleshooting

This guide helps diagnose and resolve common issues with the alphabet-terminal-prototype system.

## Common Issues

### Installation Problems

#### Compilation Errors

**Problem**: Build fails with compilation errors
```
error[E0433]: failed to resolve: use of undeclared crate or module `statrs`
```

**Solution**: Ensure all dependencies are properly specified:
```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build --release

# If using specific features
cargo build --features "cli export parallel"
```

#### Linking Errors

**Problem**: Linker errors on macOS/Linux
```
error: linking with `cc` failed: exit status: 1
```

**Solution**: Install system dependencies:
```bash
# macOS
brew install pkg-config

# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# Fedora
sudo dnf install pkg-config openssl-devel
```

### Runtime Errors

#### Panic on Startup

**Problem**: Application panics immediately
```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value'
```

**Solution**: Check configuration file:
```rust
// Add better error handling
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        eprintln!("Debug: {:?}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::load()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    // ...
}
```

#### Memory Issues

**Problem**: Out of memory errors
```
memory allocation of 137438953472 bytes failed
```

**Solution**: Reduce computational parameters:
```toml
# config.toml
[system]
monte_carlo_samples = 100  # Reduce from 1000
cache_size_mb = 128        # Reduce from 256

[bayesian]
mcmc_iterations = 1000     # Reduce from 10000
```

### Model Convergence Issues

#### MCMC Not Converging

**Problem**: Gelman-Rubin statistic > 1.1
```
Warning: MCMC chains have not converged (R-hat = 1.45)
```

**Solution**:
```rust
// Increase iterations
config.bayesian.mcmc_iterations = 50000;
config.bayesian.mcmc_burnin = 25000;

// Use better initialization
let initial_values = self.compute_mle_estimates();
mcmc.initialize(initial_values);

// Adjust step size
mcmc.tune_step_size(target_acceptance = 0.65);
```

#### Numerical Instability

**Problem**: NaN or Inf values in computations
```
Error: Numerical instability detected: NaN in posterior mean
```

**Solution**:
```rust
// Add numerical checks
fn safe_log_sum_exp(values: &[f64]) -> f64 {
    let max_val = values.iter()
        .filter(|v| v.is_finite())
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    
    if !max_val.is_finite() {
        return f64::NEG_INFINITY;
    }
    
    let sum_exp: f64 = values.iter()
        .filter(|v| v.is_finite())
        .map(|&x| (x - max_val).exp())
        .sum();
    
    max_val + sum_exp.ln()
}

// Use stable implementations
let result = stable_computation()
    .unwrap_or_else(|_| {
        warn!("Computation failed, using fallback");
        fallback_value()
    });
```

### Performance Issues

#### Slow Task Generation

**Problem**: Tasks take too long to generate
```
Task generation took 5.2 seconds
```

**Solution**: Use caching and parallel generation:
```rust
// Pre-generate task cache
let task_cache = TaskCache::new(1000);
task_cache.populate_async().await;

// Parallel generation
let tasks: Vec<Task> = (0..n)
    .into_par_iter()
    .map(|_| generator.generate_task())
    .collect();
```

#### High CPU Usage

**Problem**: 100% CPU usage during idle
```
PID   %CPU  COMMAND
12345 99.5  alphabet-terminal
```

**Solution**: Add appropriate delays and yields:
```rust
// In main loop
loop {
    if let Some(task) = scheduler.get_next_task() {
        process_task(task);
    } else {
        // Yield CPU when no work
        thread::sleep(Duration::from_millis(10));
    }
}

// Use async for better resource usage
tokio::time::sleep(Duration::from_millis(100)).await;
```

### Data Export Issues

#### Export Fails Silently

**Problem**: No output file created
```rust
exporter.export_to_file(&data, "output.json")?;
// No file appears
```

**Solution**: Check permissions and error handling:
```rust
// Better error handling
match exporter.export_to_file(&data, path) {
    Ok(_) => println!("Exported to {}", path.display()),
    Err(e) => {
        eprintln!("Export failed: {}", e);
        
        // Check common issues
        if !path.parent().map_or(true, |p| p.exists()) {
            eprintln!("Directory does not exist");
        }
        
        if let Err(e) = std::fs::write(path, "test") {
            eprintln!("No write permission: {}", e);
        }
    }
}
```

#### Corrupted Export Files

**Problem**: Export files are unreadable
```
Error: Invalid JSON at line 1523
```

**Solution**: Use atomic writes:
```rust
// Write to temporary file first
let temp_path = format!("{}.tmp", path.display());
exporter.export_to_file(&data, &temp_path)?;

// Validate before moving
if validate_export_file(&temp_path)? {
    fs::rename(temp_path, path)?;
} else {
    return Err("Export validation failed".into());
}
```

## Debugging Techniques

### Enable Debug Logging

```rust
// Set up env_logger
env_logger::Builder::from_env(
    env_logger::Env::default()
        .default_filter_or("alphabet_terminal_prototype=debug")
).init();

// Add debug output
log::debug!("Task selected: {:?}", task);
log::debug!("EIG value: {}", eig);
```

### Use Debug Assertions

```rust
#[cfg(debug_assertions)]
{
    assert!(probability >= 0.0 && probability <= 1.0, 
           "Invalid probability: {}", probability);
    assert!(self.is_valid_state(), "Invalid state detected");
}
```

### Profiling

```bash
# CPU profiling with flamegraph
cargo install flamegraph
cargo build --release
flamegraph -o flame.svg target/release/alphabet-terminal-prototype

# Memory profiling with valgrind
valgrind --tool=massif target/release/alphabet-terminal-prototype
ms_print massif.out.*
```

### Test Specific Components

```rust
#[cfg(test)]
mod troubleshooting_tests {
    #[test]
    fn test_edge_case() {
        // Isolate problematic component
        let model = BayesianLearnerModel::new(&topology);
        
        // Test with extreme values
        let task = create_extreme_task();
        let result = model.expected_information_gain(&task);
        
        assert!(result.is_finite());
        assert!(result >= 0.0);
    }
}
```

## Error Messages Reference

### Model Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `InvalidTopology` | Malformed graph structure | Verify topology has valid nodes and edges |
| `NodeNotFound` | Reference to non-existent node | Check node labels match topology |
| `CycleDetected` | Unexpected cycle in DAG | Use appropriate topology type |

### Statistical Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `SingularMatrix` | Matrix not invertible | Add regularization term |
| `ConvergenceFailure` | Optimization didn't converge | Increase iterations or adjust parameters |
| `InsufficientData` | Too few samples | Collect more data or use stronger priors |

### System Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `ConfigNotFound` | Missing config file | Create config.toml or use defaults |
| `PermissionDenied` | Can't write to directory | Check file permissions |
| `PortInUse` | Server port already bound | Change port or kill other process |

## Performance Diagnostics

### Identify Bottlenecks

```rust
use std::time::Instant;

fn diagnose_performance() {
    let mut timings = HashMap::new();
    
    let start = Instant::now();
    let task = generator.generate_task();
    timings.insert("task_generation", start.elapsed());
    
    let start = Instant::now();
    let eig = model.monte_carlo_eig(&task, 1000);
    timings.insert("eig_computation", start.elapsed());
    
    // Print slowest operations
    let mut sorted: Vec<_> = timings.iter().collect();
    sorted.sort_by_key(|&(_, duration)| duration);
    
    for (operation, duration) in sorted.iter().rev().take(5) {
        println!("{}: {:?}", operation, duration);
    }
}
```

### Memory Profiling

```rust
fn check_memory_usage() {
    use jemalloc_ctl::{stats, epoch};
    
    // Update statistics
    epoch::mib().unwrap().advance().unwrap();
    
    let allocated = stats::allocated::mib().unwrap().read().unwrap();
    let resident = stats::resident::mib().unwrap().read().unwrap();
    
    println!("Allocated: {} MB", allocated / 1_048_576);
    println!("Resident: {} MB", resident / 1_048_576);
    
    if allocated > 1_073_741_824 { // 1 GB
        warn!("High memory usage detected");
        self.clear_caches();
    }
}
```

## Getting Help

### Diagnostic Information

When reporting issues, include:

```rust
fn collect_diagnostic_info() -> DiagnosticReport {
    DiagnosticReport {
        version: env!("CARGO_PKG_VERSION"),
        rust_version: rustc_version::version().unwrap(),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        config: AppConfig::load().ok(),
        error_trace: std::backtrace::Backtrace::capture(),
    }
}
```

### Community Resources

- GitHub Issues: Report bugs and request features
- Discord: Real-time help and discussion
- Documentation: Comprehensive guides and API reference
- Examples: Working code samples

### Minimal Reproducible Example

```rust
// Provide minimal code that reproduces the issue
fn reproduce_issue() {
    let topology = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topology);
    
    // This causes the error
    let task = Task {
        task_type: TaskType::KJump { 
            start: "Z".to_string(), 
            k: 100  // This might be the issue
        },
        // ...
    };
    
    let result = model.expected_information_gain(&task);
    // Error occurs here
}
```

## Summary

Troubleshooting guide covers:
- **Common issues**: Installation, runtime, convergence
- **Performance problems**: Slow generation, high CPU
- **Data export issues**: File creation, corruption
- **Debugging techniques**: Logging, profiling, testing
- **Error reference**: Common errors and solutions
- **Diagnostics**: Performance and memory profiling
- **Getting help**: Reporting issues effectively

This comprehensive troubleshooting guide helps quickly identify and resolve issues with the alphabet-terminal-prototype system.