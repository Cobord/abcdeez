#!/usr/bin/env bash

# Xcode prebuild helper: rebuild Rust XCFramework only when sources changed
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

CONFIGURATION=${CONFIGURATION:-Debug}
SDK_NAME=${SDK_NAME:-${SDK:-iphonesimulator}}

PROFILE=debug
if [[ "$CONFIGURATION" != "Debug" ]]; then
  PROFILE=release
fi

# Compute a hash of Rust source and Cargo files for relevant crates
HASH_INPUT=$(mktemp)
{
  find ../xilem-app ../web-backend ../core ./mobile -type f \
    \( -name "*.rs" -o -name "Cargo.toml" -o -name "Cargo.lock" \) \
    -print0 | sort -z | xargs -0 shasum | awk '{print $1}';
} > "$HASH_INPUT"
SRC_HASH=$(shasum "$HASH_INPUT" | awk '{print $1}')
rm -f "$HASH_INPUT"

CACHE_DIR=".build-cache/$PROFILE/$SDK_NAME"
mkdir -p "$CACHE_DIR"
STAMP_FILE="$CACHE_DIR/rust.src.sha1"

NEEDS_BUILD=1
if [[ -f "$STAMP_FILE" ]]; then
  if [[ "$SRC_HASH" == "$(cat "$STAMP_FILE")" ]]; then
    NEEDS_BUILD=0
  fi
fi

if [[ $NEEDS_BUILD -eq 0 ]]; then
  echo "Rust sources unchanged for $CONFIGURATION/$SDK_NAME; skipping rebuild"
  exit 0
fi

echo "Rebuilding Rust XCFramework for $CONFIGURATION ($PROFILE) ..."
if [[ "$PROFILE" == "release" ]]; then
  ./make_xcframework.sh --release
else
  ./make_xcframework.sh
fi

echo "$SRC_HASH" > "$STAMP_FILE"
echo "Done. Updated $STAMP_FILE"


