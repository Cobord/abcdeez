#!/bin/bash
set -e

echo "Building ABCDeez Android APK..."

# Check for required tools
if ! command -v cargo-apk &> /dev/null; then
    echo "Installing cargo-apk..."
    cargo install cargo-apk
fi

# Set up Android SDK if not configured
if [ -z "$ANDROID_SDK_ROOT" ]; then
    echo "Warning: ANDROID_SDK_ROOT not set. Please configure Android SDK."
    echo "You can install it via Android Studio or command line tools."
    exit 1
fi

# Build for ARM64 (modern devices)
echo "Building for ARM64..."
cargo apk build --release --target aarch64-linux-android

# Build for ARMv7 (older devices)
echo "Building for ARMv7..."
cargo apk build --release --target armv7-linux-androideabi

# Create output directory
mkdir -p dist/android

# Copy APKs to dist
cp target/release/apk/*.apk dist/android/ 2>/dev/null || echo "APKs will be in target/release/apk/"

echo "Android build complete!"
echo "APKs location: target/release/apk/"