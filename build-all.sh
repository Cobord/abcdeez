#!/bin/bash
# Comprehensive build script for ABCDeez across all platforms

set -e

echo "========================================"
echo "ABCDeez Multi-Platform Build System"
echo "========================================"

# Parse command line arguments
BUILD_DESKTOP=false
BUILD_WEB=false
BUILD_ANDROID=false
BUILD_IOS=false
BUILD_ALL=false

if [ $# -eq 0 ]; then
    BUILD_ALL=true
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        --desktop)
            BUILD_DESKTOP=true
            shift
            ;;
        --web)
            BUILD_WEB=true
            shift
            ;;
        --android)
            BUILD_ANDROID=true
            shift
            ;;
        --ios)
            BUILD_IOS=true
            shift
            ;;
        --all)
            BUILD_ALL=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--desktop] [--web] [--android] [--ios] [--all]"
            exit 1
            ;;
    esac
done

if [ "$BUILD_ALL" = true ]; then
    BUILD_DESKTOP=true
    BUILD_WEB=true
    BUILD_ANDROID=true
    BUILD_IOS=true
fi

# Create dist directory
mkdir -p dist

# Build Desktop (macOS, Linux, Windows)
if [ "$BUILD_DESKTOP" = true ]; then
    echo ""
    echo "Building Desktop Applications..."
    echo "--------------------------------"
    
    # Use cargo-dist for desktop builds
    echo "Running cargo-dist build..."
    dist build --artifacts=local --no-local-paths
    
    echo "✓ Desktop builds complete"
fi

# Build Web PWA
if [ "$BUILD_WEB" = true ]; then
    echo ""
    echo "Building Web PWA..."
    echo "-------------------"
    
    if [ -f "xilem-cross-platform/web/build.sh" ]; then
        cd xilem-cross-platform/web
        ./build.sh
        cd ../..
    else
        echo "Warning: Web build script not found"
    fi
    
    echo "✓ Web PWA build complete"
fi

# Build Android
if [ "$BUILD_ANDROID" = true ]; then
    echo ""
    echo "Building Android APK..."
    echo "-----------------------"
    
    if [ -f "xilem-cross-platform/mobile/build-android.sh" ]; then
        cd xilem-cross-platform/mobile
        ./build-android.sh
        cd ../..
    else
        echo "Warning: Android build script not found"
    fi
    
    echo "✓ Android build complete"
fi

# Build iOS
if [ "$BUILD_IOS" = true ]; then
    echo ""
    echo "Building iOS App..."
    echo "-------------------"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if [ -f "xilem-cross-platform/mobile/build-ios.sh" ]; then
            cd xilem-cross-platform/mobile
            ./build-ios.sh
            cd ../..
        else
            echo "Warning: iOS build script not found"
        fi
        echo "✓ iOS build complete"
    else
        echo "Skipping iOS build (requires macOS)"
    fi
fi

echo ""
echo "========================================"
echo "Build Summary"
echo "========================================"
echo ""
echo "Completed builds:"
[ "$BUILD_DESKTOP" = true ] && echo "  ✓ Desktop (macOS, Linux, Windows)"
[ "$BUILD_WEB" = true ] && echo "  ✓ Web PWA"
[ "$BUILD_ANDROID" = true ] && echo "  ✓ Android APK"
[ "$BUILD_IOS" = true ] && echo "  ✓ iOS App"
echo ""
echo "Build artifacts are available in:"
echo "  - Desktop: target/distrib/"
echo "  - Web PWA: xilem-cross-platform/web/dist/"
echo "  - Android: target/release/apk/"
echo "  - iOS: dist/ios/"
echo ""
echo "To create a release, push a version tag:"
echo "  git tag v0.1.0"
echo "  git push origin v0.1.0"
echo ""
