#!/bin/bash
set -e

echo "Building ABCDeez Web PWA..."

# Install wasm-pack if not available
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module
echo "Building WASM module..."
wasm-pack build --target web --out-name abcdeez --release

# Copy PWA files to pkg directory
echo "Copying PWA files..."
cp manifest.json pkg/ 2>/dev/null || echo "Warning: manifest.json not found"
cp service-worker.js pkg/ 2>/dev/null || echo "Warning: service-worker.js not found"
cp index.html pkg/ 2>/dev/null || echo "Warning: index.html not found"

# Create dist directory
mkdir -p dist

# Copy all files to dist
cp -r pkg/* dist/

echo "Web PWA build complete! Output in dist/"