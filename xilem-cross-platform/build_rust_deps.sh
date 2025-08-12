#!/usr/bin/env bash

# based on https://github.com/mozilla/glean/blob/main/build-scripts/xc-universal-binary.sh

set -euxo pipefail

PATH=$PATH:$HOME/.cargo/bin

# Set minimum iOS deployment target to match Xcode project (12.0)
export IPHONEOS_DEPLOYMENT_TARGET=12.0
export CARGO_BUILD_RUSTFLAGS="-C link-arg=-mios-version-min=12.0"

# Determine if we are building in Debug or Release mode
RELFLAG=
TARGET_DIR_SUFFIX=debug
if [[ "${CONFIGURATION:-Debug}" != "Debug" ]]; then
	RELFLAG=--release
	TARGET_DIR_SUFFIX=release
fi

if [[ -n "${DEVELOPER_SDK_DIR:-}" ]]; then
    # Assume we're in Xcode, which means we're probably cross-compiling.
    # Ensure host (proc-macro/build-script) crates link against macOS SDK libs
    MACOS_SDK_DIR="$(xcrun --sdk macosx --show-sdk-path)"
    export LIBRARY_PATH="${MACOS_SDK_DIR}/usr/lib:${LIBRARY_PATH:-}"
fi

# add homebrew bin path, as it's the most commonly used package manager on macOS
# this is needed for cmake on apple arm processors as it's not available by default
export PATH="$PATH:/opt/homebrew/bin"

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
	IS_SIMULATOR=1
fi

IS_MACABI=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-macabi" ]; then
	IS_MACABI=1
fi

# Ensure host builds do not inherit iOS SDKROOT (breaks proc-macro/linking)
unset SDKROOT || true

for arch in $ARCHS; do
	case "$arch" in
	x86_64)
		if [ $IS_SIMULATOR -eq 1 ]; then
			# Intel iOS simulator
			export CFLAGS_x86_64_apple_ios="-target x86_64-apple-ios -mios-simulator-version-min=12.0"
			target="x86_64-apple-ios"
			cargo rustc --crate-type staticlib $RELFLAG --lib --target $target --package xilem_abcdeez_mobile
		else
			echo "Building for x86_64, but not a simulator build. What's going on?" >&2
			exit 2
		fi
		;;
	arm64)
        if [ $IS_SIMULATOR -eq 1 ]; then
            # M1 iOS simulator (use modern clang target spelling)
            export CFLAGS_aarch64_apple_ios_sim="-target arm64-apple-ios-simulator -mios-simulator-version-min=12.0"
            export CC_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find clang)"
            export AR_aarch64_apple_ios_sim="$(xcrun --sdk iphonesimulator --find ar)"
			target="aarch64-apple-ios-sim"
			cargo rustc --crate-type staticlib $RELFLAG --lib --target $target --package xilem_abcdeez_mobile
		else
			# Hardware iOS targets
            export CFLAGS_aarch64_apple_ios="-target aarch64-apple-ios -mios-version-min=12.0"
            export CC_aarch64_apple_ios="$(xcrun --sdk iphoneos --find clang)"
            export AR_aarch64_apple_ios="$(xcrun --sdk iphoneos --find ar)"
			target="aarch64-apple-ios"
			cargo rustc --crate-type staticlib $RELFLAG --lib --target $target --package xilem_abcdeez_mobile
		fi
		;;
	esac

	# After building, copy from ../target/$target/$TARGET_DIR_SUFFIX to ./target/$target/$TARGET_DIR_SUFFIX so Xcode can find it
	if [ -n "${target:-}" ]; then
		mkdir -p ./target/$target/$TARGET_DIR_SUFFIX
		cp -r ../target/$target/$TARGET_DIR_SUFFIX/* ./target/$target/$TARGET_DIR_SUFFIX/
	fi
done
