#!/bin/bash
set -e

echo "Building ABCDeez iOS app..."

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: iOS builds require macOS with Xcode installed"
    exit 1
fi

# Check for Xcode
if ! command -v xcodebuild &> /dev/null; then
    echo "Error: Xcode is required for iOS builds"
    echo "Please install Xcode from the Mac App Store"
    exit 1
fi

# Install cargo-xcode if not available
if ! command -v cargo-xcode &> /dev/null; then
    echo "Installing cargo-xcode..."
    cargo install cargo-xcode
fi

# Build for iOS simulator (x86_64)
echo "Building for iOS Simulator..."
cargo build --release --target x86_64-apple-ios

# Build for iOS devices (ARM64)
echo "Building for iOS devices..."
cargo build --release --target aarch64-apple-ios

# Create universal binary
echo "Creating universal binary..."
mkdir -p dist/ios

lipo -create \
    target/x86_64-apple-ios/release/xilem_abcdeez_mobile \
    target/aarch64-apple-ios/release/xilem_abcdeez_mobile \
    -output dist/ios/abcdeez

echo "iOS build complete!"
echo "Universal binary: dist/ios/abcdeez"
echo "Note: To create an .ipa file, you'll need to use Xcode with proper signing certificates"