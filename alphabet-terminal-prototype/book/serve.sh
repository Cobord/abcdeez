#!/bin/bash
# Local development server for the book

set -e

echo "📚 Starting mdBook development server..."

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook is not installed!"
    echo "Install with: cargo install mdbook"
    exit 1
fi

# Check if mdbook-katex is installed (optional but recommended)
if ! command -v mdbook-katex &> /dev/null; then
    echo "⚠️  mdbook-katex is not installed (math rendering may not work)"
    echo "Install with: cargo install mdbook-katex"
fi

# Change to book directory
cd "$(dirname "$0")"

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf build

# Serve with live reload
echo "🚀 Starting server at http://localhost:3000"
echo "   (Changes will auto-reload)"
mdbook serve --open