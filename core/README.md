# Alphabet Terminal Prototype

A Bayesian cognitive modeling system for adaptive learning, implementing Expected Information Gain (EIG) for optimal task selection.

**[📚 Full Documentation](https://abcdeez.fg-goose.online)**

## Quick Start

```bash
# Run the terminal UI
cargo run --release

# Run with custom config
cargo run --release -- --config my-config.toml
```

## Features

- **Bayesian Inference**: Maintains probability distributions over learner knowledge
- **Adaptive Task Selection**: Uses Expected Information Gain to select optimal tasks
- **Strategy Detection**: Identifies cognitive strategies from response patterns
- **Response Time Modeling**: Ex-Gaussian models capture temporal dynamics
- **Multi-Platform UI**: Terminal, web server, and native GUI interfaces

## Installation

```bash
# Clone the repository
git clone https://github.com/emberian/abcdeez.git
cd abcdeez/alphabet-terminal-prototype

# Build and run
cargo build --release
cargo run --release
```

## Usage

### As a Library

```rust
use alphabet_terminal_prototype::prelude::*;

let topology = Topology::alphabet();
let mut learner = LearnerModel::new("student_001", &topology);
let mut scheduler = AdaptiveScheduler::new(learner, topology);

// Run learning session
for _ in 0..50 {
    let task = scheduler.select_next_task();
    // Present task and get response...
    scheduler.update(response);
}
```

### Configuration

Create a `config.toml` file:

```toml
[learner]
initial_learning_rate = 0.1

[scheduler]
strategy = "maximize_eig"
min_task_difficulty = 0.1
max_task_difficulty = 0.9

[bayesian]
monte_carlo_samples = 1000
```

## Documentation

- **[Online Documentation](https://abcdeez.fg-goose.online)** - Full mdBook with mathematical foundations
- **[API Reference](https://abcdeez.fg-goose.online/api_reference.html)** - Complete API documentation
- **Local Docs**: Run `cd book && ./serve.sh` for local documentation server

## Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## License

MIT OR Apache-2.0