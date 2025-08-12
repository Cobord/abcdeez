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

# Generate build hash for cache busting
BUILD_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "dev")
BUILD_TIME=$(date +%s)
echo "Build hash: $BUILD_HASH, Build time: $BUILD_TIME"

# Copy PWA files to pkg directory
echo "Copying PWA files..."
cp manifest.json pkg/ 2>/dev/null || echo "Warning: manifest.json not found"

# Update service worker with build hash
if [ -f service-worker.js ]; then
    sed "s/const CACHE_VERSION = 'v1'/const CACHE_VERSION = 'v1-$BUILD_HASH'/" service-worker.js > pkg/service-worker.js
else
    echo "Warning: service-worker.js not found"
fi

# Update index.html to include version in script imports
if [ -f index.html ]; then
    sed "s/abcdeez\.js/abcdeez.js?v=$BUILD_HASH/" index.html > pkg/index.html
else
    echo "Warning: index.html not found"
fi

# Create dist directory
mkdir -p dist

# Copy all files to dist
cp -r pkg/* dist/

echo "Web PWA build complete! Output in dist/"
