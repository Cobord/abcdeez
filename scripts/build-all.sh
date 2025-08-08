#!/bin/bash
set -e

echo "========================================="
echo "Building Graph Learning for all platforms"
echo "========================================="

# Build TUI binary (cross-platform CLI)
echo ""
echo "Building TUI binary..."
cd alphabet-terminal-prototype
cargo build --release --bin graph-learning-cli
mkdir -p ../dist/cli
cp target/release/graph-learning-cli ../dist/cli/
cd ..

# Build backend binary
echo ""
echo "Building backend binary..."
cd alphabet-terminal-prototype
cargo build --release --bin mock_backend --features backend-server
cp target/release/mock_backend ../dist/cli/graph-learning-backend
cd ..

# Platform-specific builds
PLATFORM=$(uname -s)

case "$PLATFORM" in
    Darwin)
        echo ""
        echo "Detected macOS - building macOS bundle..."
        bash scripts/build-macos.sh
        
        echo ""
        echo "Note: To build for other platforms, use:"
        echo "  - Android: ./scripts/build-android.sh"
        echo "  - Linux AppImage: ./scripts/build-linux-appimage.sh"
        echo "  - Windows: Use PowerShell on Windows to run scripts/build-windows.ps1"
        ;;
    
    Linux)
        echo ""
        echo "Detected Linux - building AppImage..."
        bash scripts/build-linux-appimage.sh
        
        echo ""
        echo "Building Android APK..."
        bash scripts/build-android.sh
        
        echo ""
        echo "Note: To build for other platforms:"
        echo "  - macOS: Run this script on macOS"
        echo "  - Windows: Use PowerShell on Windows to run scripts/build-windows.ps1"
        ;;
    
    MINGW*|CYGWIN*|MSYS*)
        echo ""
        echo "Detected Windows - please run scripts/build-windows.ps1 in PowerShell"
        echo ""
        echo "Note: To build for other platforms:"
        echo "  - Android: ./scripts/build-android.sh (requires Android SDK)"
        echo "  - macOS: Run this script on macOS"
        echo "  - Linux: Run this script on Linux"
        ;;
    
    *)
        echo "Unknown platform: $PLATFORM"
        exit 1
        ;;
esac

echo ""
echo "========================================="
echo "Build Summary"
echo "========================================="
echo "CLI binaries: dist/cli/"
echo "  - graph-learning-cli (TUI)"
echo "  - graph-learning-backend (Backend server)"

if [ -d "dist/macos" ]; then
    echo "macOS: dist/macos/"
fi
if [ -d "dist/linux" ]; then
    echo "Linux: dist/linux/"
fi
if [ -d "dist/android" ]; then
    echo "Android: dist/android/"
fi
if [ -d "dist/windows" ]; then
    echo "Windows: dist/windows/"
fi

echo ""
echo "To distribute via cargo-dist, run:"
echo "  cargo dist build"
echo "  cargo dist upload"