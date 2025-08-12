#!/usr/bin/env bash

set -euo pipefail

CONFIGURATION=Debug
PROFILE=debug
if [[ "${1:-}" == "--release" ]]; then
  CONFIGURATION=Release
  PROFILE=release
fi

echo "Building XCFramework (configuration: $CONFIGURATION)"

# Ensure targets exist
rustup target add aarch64-apple-ios >/dev/null 2>&1 || true
rustup target add aarch64-apple-ios-sim >/dev/null 2>&1 || true

# Common env for proc-macros/cc crates
export IPHONEOS_DEPLOYMENT_TARGET=12.0
export CARGO_BUILD_RUSTFLAGS="-C link-arg=-mios-version-min=12.0"
export SQLX_OFFLINE=1

# Point host (proc-macro) linking at macOS SDK to avoid iOS SDK pollution
MACOS_SDK_DIR="$(xcrun --sdk macosx --show-sdk-path)"
export LIBRARY_PATH="${MACOS_SDK_DIR}/usr/lib:${LIBRARY_PATH:-}"
unset SDKROOT || true

# Build static libs for sim and device
echo "- Building (simulator) aarch64-apple-ios-sim"
export CFLAGS_aarch64_apple_ios_sim="-target arm64-apple-ios-simulator -mios-simulator-version-min=12.0"
export CC_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find clang)"
export AR_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find ar)"
cargo rustc -p xilem_abcdeez_mobile --lib --crate-type staticlib --target aarch64-apple-ios-sim $( [[ "$PROFILE" == release ]] && echo --release )

echo "- Building (device) aarch64-apple-ios"
export CFLAGS_aarch64_apple_ios="-target aarch64-apple-ios -mios-version-min=12.0"
export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
cargo rustc -p xilem_abcdeez_mobile --lib --crate-type staticlib --target aarch64-apple-ios $( [[ "$PROFILE" == release ]] && echo --release )

SIM_LIB="../target/aarch64-apple-ios-sim/${PROFILE}/libxilem_abcdeez_mobile.a"
DEV_LIB="../target/aarch64-apple-ios/${PROFILE}/libxilem_abcdeez_mobile.a"

if [[ ! -f "$SIM_LIB" || ! -f "$DEV_LIB" ]]; then
  echo "Error: expected static libraries not found"
  exit 2
fi

# Also mirror artifacts to project-local ./target tree for Xcode linking compatibility
echo "- Mirroring build artifacts to ./target for Xcode"
mkdir -p ./target/aarch64-apple-ios-sim/${PROFILE}
mkdir -p ./target/aarch64-apple-ios/${PROFILE}
cp -R ../target/aarch64-apple-ios-sim/${PROFILE}/* ./target/aarch64-apple-ios-sim/${PROFILE}/ || true
cp -R ../target/aarch64-apple-ios/${PROFILE}/* ./target/aarch64-apple-ios/${PROFILE}/ || true

OUT_DIR="dist"
mkdir -p "$OUT_DIR"
XC_OUT="$OUT_DIR/xilem_abcdeez_mobile.xcframework"
rm -rf "$XC_OUT"

echo "- Creating XCFramework"
xcodebuild -create-xcframework \
  -library "$DEV_LIB" \
  -headers ios-src \
  -library "$SIM_LIB" \
  -headers ios-src \
  -output "$XC_OUT"

echo "✅ XCFramework created: $XC_OUT"
echo "Add it to Xcode: Project → target → Frameworks, Libraries, and Embedded Content → + → Add Other… → Add Files… → select $XC_OUT (Do Not Embed)."


