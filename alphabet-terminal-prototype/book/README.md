# Alphabet Terminal Prototype Documentation

This directory contains the mdBook documentation for the alphabet-terminal-prototype cognitive modeling system.

## Quick Start

### Local Development

```bash
# Install mdBook
cargo install mdbook

# Optional: Install math support
cargo install mdbook-katex

# Serve locally with hot reload
./serve.sh

# Or manually:
mdbook serve --open
```

### Build

```bash
# Build static HTML
mdbook build

# Output will be in build/
```

## Structure

- `src/` - Markdown source files
- `book.toml` - mdBook configuration
- `build/` - Generated HTML output (git-ignored)

## GitHub Actions

The following workflows automate documentation:

- **book.yml** - Build and deploy to GitHub Pages on main branch
- **book-pr.yml** - Generate preview for pull requests (requires Netlify)
- **book-release.yml** - Create versioned documentation on releases
- **book-check.yml** - Lint and validate documentation

### Setup GitHub Pages

1. Go to Settings → Pages
2. Set Source to "GitHub Actions"
3. The book will be published at: `https://[username].github.io/abcdeez/`

### Setup PR Previews (Optional)

1. Create a Netlify account
2. Create a new site
3. Add secrets to GitHub:
   - `NETLIFY_AUTH_TOKEN`
   - `NETLIFY_SITE_ID`

## Writing Documentation

### Math Support

Use LaTeX syntax:
```markdown
Inline math: $\mu = 0$

Display math:
$$
f(x) = \frac{1}{\sigma\sqrt{2\pi}} e^{-\frac{(x-\mu)^2}{2\sigma^2}}
$$
```

### Code Examples

```markdown
```rust
pub fn example() -> Result<(), Error> {
    // Code here
}
` ` `  // (without spaces)
```

### Internal Links

```markdown
See [Bayesian Inference](./bayesian_inference.md)
```

## Chapters

Core concepts:
- Mathematical Foundations
- Bayesian Inference  
- Expected Information Gain
- Ex-Gaussian Distribution

System components:
- System Architecture
- Topology Models
- Learner Model
- Task Generation

Advanced topics:
- Hierarchical Bayesian Models
- Transfer Learning
- Performance Prediction
- Memory and Forgetting

## Contributing

1. Edit markdown files in `src/`
2. Run `./serve.sh` to preview changes
3. Commit and push - GitHub Actions will deploy

## License

Same as parent project