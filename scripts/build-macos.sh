#!/bin/bash
set -e

# Build macOS application bundle
echo "Building macOS bundle..."

# Build the desktop app for macOS
cd xilem-cross-platform/desktop
cargo build --release

# Create app bundle structure
APP_NAME="Graph Learning"
BUNDLE_NAME="$APP_NAME.app"
BUNDLE_DIR="../../dist/macos/$BUNDLE_NAME"

rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy binary
cp target/release/xilem_abcdeez_desktop "$BUNDLE_DIR/Contents/MacOS/$APP_NAME"

# Create Info.plist
cat > "$BUNDLE_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>Graph Learning</string>
    <key>CFBundleIdentifier</key>
    <string>org.graphlearning.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Graph Learning</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Install cargo-bundle if needed for DMG creation
if ! command -v cargo-bundle &> /dev/null; then
    echo "Installing cargo-bundle..."
    cargo install cargo-bundle
fi

echo "macOS bundle built successfully!"
echo "Bundle location: dist/macos/$BUNDLE_NAME"

# Create DMG
echo "Creating DMG installer..."
hdiutil create -volname "$APP_NAME" -srcfolder "$BUNDLE_DIR" -ov -format UDZO "../../dist/macos/$APP_NAME.dmg"
echo "DMG created: dist/macos/$APP_NAME.dmg"