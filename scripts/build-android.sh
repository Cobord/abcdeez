#!/bin/bash
set -e

# Build Android APK for the xilem mobile app
echo "Building Android APK..."

cd xilem-cross-platform/mobile

# Install cargo-apk if not already installed
if ! command -v cargo-apk &> /dev/null; then
    echo "Installing cargo-apk..."
    cargo install cargo-apk
fi

# Build the APK for multiple architectures
cargo apk build --release

# Copy APK to distribution directory
mkdir -p ../../dist/android
cp target/release/apk/*.apk ../../dist/android/

echo "Android APK built successfully!"
echo "APK location: dist/android/"