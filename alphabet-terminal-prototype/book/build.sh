#!/bin/bash
# Build the book locally to test

set -e

echo "📚 Building mdBook..."

# Change to book directory
cd "$(dirname "$0")"

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook is not installed!"
    echo ""
    echo "Install with one of these methods:"
    echo "  cargo install mdbook"
    echo "  brew install mdbook (macOS)"
    echo "  Download from: https://github.com/rust-lang/mdBook/releases"
    exit 1
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf build

# Build the book
echo "🔨 Building book..."
mdbook build

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📁 Output is in: $(pwd)/build/"
    echo ""
    echo "To view locally, run:"
    echo "  cd $(pwd)/build && python3 -m http.server 8000"
    echo "  Then open: http://localhost:8000"
else
    echo "❌ Build failed!"
    exit 1
fi