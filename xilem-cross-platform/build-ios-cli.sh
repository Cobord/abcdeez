#!/usr/bin/env bash

# iOS CLI Build Script for xilem-cross-platform
# This script builds the iOS app without using Xcode GUI
# Based on the existing Xcode project configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Building xilem_abcdeez_mobile iOS app...${NC}"

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}Error: iOS builds require macOS with Xcode installed${NC}"
    exit 1
fi

# Check for Xcode command line tools
if ! command -v xcodebuild &> /dev/null; then
    echo -e "${RED}Error: xcodebuild not found. Please install Xcode or Xcode Command Line Tools${NC}"
    echo "Run: xcode-select --install"
    exit 1
fi

# Configuration variables (can be overridden via environment)
CONFIGURATION=${CONFIGURATION:-"Debug"}
SDK=${SDK:-"iphoneos"}
DESTINATION=${DESTINATION:-"generic/platform=iOS"}
DEVELOPMENT_TEAM=${DEVELOPMENT_TEAM:-"992772WV7K"}
BUNDLE_IDENTIFIER=${BUNDLE_IDENTIFIER:-"online.fg-goose.abcdeez"}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            CONFIGURATION="Release"
            shift
            ;;
        --simulator)
            SDK="iphonesimulator"
            # Use generic iOS Simulator destination
            DESTINATION="generic/platform=iOS Simulator"
            # Ensure Rust and cc build scripts get correct simulator flags
            export LLVM_TARGET_TRIPLE_SUFFIX="-simulator"
            shift
            ;;
        --device)
            SDK="iphoneos"
            DESTINATION="generic/platform=iOS"
            unset LLVM_TARGET_TRIPLE_SUFFIX
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --release       Build in Release mode (default: Debug)"
            echo "  --simulator     Build for iOS Simulator"
            echo "  --device        Build for iOS device (default)"
            echo "  --clean         Clean build folders before building"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set Rust toolchain targets based on SDK
if [[ "$SDK" == "iphonesimulator" ]]; then
    # Determine simulator architecture
    if [[ $(uname -m) == "arm64" ]]; then
        RUST_TARGETS="aarch64-apple-ios-sim"
        ARCHS="arm64"
    else
        RUST_TARGETS="x86_64-apple-ios"
        ARCHS="x86_64"
    fi
else
    RUST_TARGETS="aarch64-apple-ios"
    ARCHS="arm64"
fi

echo -e "${YELLOW}Configuration:${NC}"
echo "  Mode: $CONFIGURATION"
echo "  SDK: $SDK"
echo "  Destination: $DESTINATION"
echo "  Architecture: $ARCHS"
echo "  Team ID: $DEVELOPMENT_TEAM"
echo "  Bundle ID: $BUNDLE_IDENTIFIER"
echo ""

# Install required Rust targets if not present
for target in $RUST_TARGETS; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo -e "${YELLOW}Installing Rust target: $target${NC}"
        rustup target add "$target"
    fi
done

# Clean if requested
if [[ "$CLEAN_BUILD" == "true" ]]; then
    echo -e "${YELLOW}Cleaning build directories...${NC}"
    rm -rf build
    rm -rf DerivedData
    cargo clean
fi

# Step 1: Build Rust dependencies
echo -e "${GREEN}Step 1: Building Rust dependencies...${NC}"

# Export necessary environment variables for the build script
export CONFIGURATION="$CONFIGURATION"
export ARCHS="$ARCHS"
export DEVELOPER_SDK_DIR=$(xcrun --sdk "$SDK" --show-sdk-path)

if [[ "$SDK" == "iphonesimulator" ]]; then
    export LLVM_TARGET_TRIPLE_SUFFIX="-simulator"
fi

# Run the Rust build script
./build_rust_deps.sh

# Step 2: Build the iOS app using xcodebuild
echo -e "${GREEN}Step 2: Building iOS app with xcodebuild...${NC}"

# Create build directory
BUILD_DIR="build"
mkdir -p "$BUILD_DIR"

# Build command
XCODEBUILD_CMD="xcodebuild \
    -project xilem_abcdeez_mobile.xcodeproj \
    -scheme xilem_abcdeez_mobile \
    -configuration $CONFIGURATION \
    -sdk $SDK \
    -destination \"$DESTINATION\" \
    -derivedDataPath DerivedData \
    DEVELOPMENT_TEAM=$DEVELOPMENT_TEAM \
    PRODUCT_BUNDLE_IDENTIFIER=$BUNDLE_IDENTIFIER \
    ONLY_ACTIVE_ARCH=NO \
    ARCHS=\"$ARCHS\""

# Add code signing options for device builds
if [[ "$SDK" == "iphoneos" ]]; then
    XCODEBUILD_CMD="$XCODEBUILD_CMD \
        CODE_SIGN_IDENTITY=\"Apple Development\" \
        CODE_SIGN_STYLE=Automatic"
else
    # Simulator builds don't need code signing
    XCODEBUILD_CMD="$XCODEBUILD_CMD \
        CODE_SIGN_IDENTITY=\"\" \
        CODE_SIGNING_REQUIRED=NO \
        CODE_SIGNING_ALLOWED=NO"
fi

# Execute the build
echo "Running: $XCODEBUILD_CMD build"
eval $XCODEBUILD_CMD build

# Step 3: Package the app (optional, for device builds)
if [[ "$SDK" == "iphoneos" && "$CONFIGURATION" == "Release" ]]; then
    echo -e "${GREEN}Step 3: Creating archive for distribution...${NC}"
    
    ARCHIVE_PATH="$BUILD_DIR/xilem_abcdeez_mobile.xcarchive"
    
    eval $XCODEBUILD_CMD archive -archivePath "$ARCHIVE_PATH"
    
    if [[ -f "$ARCHIVE_PATH" ]]; then
        echo -e "${GREEN}Archive created: $ARCHIVE_PATH${NC}"
        
        # Optional: Export IPA
        echo -e "${YELLOW}To export IPA, run:${NC}"
        echo "xcodebuild -exportArchive -archivePath \"$ARCHIVE_PATH\" -exportPath \"$BUILD_DIR\" -exportOptionsPlist ExportOptions.plist"
    fi
fi

# Find and display the built app location
APP_PATH=$(find DerivedData -name "xilem_abcdeez_mobile.app" -type d | head -n 1)

if [[ -n "$APP_PATH" ]]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo -e "${GREEN}App location: $APP_PATH${NC}"
    
    # Display app size
    APP_SIZE=$(du -sh "$APP_PATH" | cut -f1)
    echo -e "${YELLOW}App size: $APP_SIZE${NC}"
    
    # For simulator builds, offer to install and run
    if [[ "$SDK" == "iphonesimulator" ]]; then
        echo ""
        echo -e "${YELLOW}To install and run on simulator:${NC}"
        echo "xcrun simctl install booted \"$APP_PATH\""
        echo "xcrun simctl launch booted $BUNDLE_IDENTIFIER"
    fi
else
    echo -e "${RED}⚠️  Build completed but app bundle not found${NC}"
    exit 1
fi
