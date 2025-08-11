# ABCDeez Core Documentation

This directory contains the comprehensive mdBook documentation for the ABCDeez Core library.

## 📚 Documentation Structure

The documentation is organized into the following sections:

- **Core Concepts**: Topology system, architecture, and key features
- **Learning Models**: Learner models, Bayesian framework, memory and forgetting
- **Statistical Methods**: Ex-Gaussian distributions, response time analysis, validation
- **Cognitive Assessment**: Task generation, adaptive selection, psychological phenomena
- **Implementation Guide**: Configuration, API reference, usage examples
- **Testing & Quality**: Test suite documentation, quality assurance

## 🚀 Building the Documentation

### Prerequisites

Install mdBook and optional dependencies:

```bash
# Install mdBook
cargo install mdbook

# Install mdbook-mermaid for diagram support (optional)
cargo install mdbook-mermaid
```

### Build

To build the documentation:

```bash
./build.sh
```

Or manually:

```bash
mdbook build
```

The built documentation will be in the `book/` directory. Open `book/index.html` in your browser.

### Development Server

To serve the documentation locally with auto-reload:

```bash
mdbook serve --open
```

This will start a local server at `http://localhost:3000` and open it in your browser.

## 📖 Documentation Status

### ✅ Completed Chapters
- Introduction and Overview
- System Architecture
- Topology System Introduction
- Ex-Gaussian Distribution
- Serial Position Effects
- Memory & Forgetting
- Expected Information Gain
- API Reference Introduction

### 🚧 In Progress
All other chapters have comprehensive placeholder content and will be expanded with detailed documentation.

## 🎯 Key Features

- **Mathematical Foundations**: Detailed explanations of all mathematical models
- **Code Examples**: Extensive Rust code examples throughout
- **Research References**: Citations to cognitive science literature
- **Practical Applications**: Real-world use cases and patterns
- **API Documentation**: Complete reference for all public APIs
- **Test Documentation**: Comprehensive test suite explanation

## 📝 Contributing

To contribute to the documentation:

1. Edit the relevant Markdown files in `src/`
2. Test your changes with `mdbook serve`
3. Ensure all code examples compile
4. Submit a pull request

## 🔗 Links

- [ABCDeez Core Library](../)
- [API Documentation](https://docs.rs/abcdeez-core)
- [GitHub Repository](https://github.com/abcdeez/core)

## 📄 License

This documentation is part of the ABCDeez Core project and follows the same license.