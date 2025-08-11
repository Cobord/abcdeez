#!/bin/bash
set -e

# Build Linux AppImage
echo "Building Linux AppImage..."

cd xilem-cross-platform/desktop

# Build the application
cargo build --release

# Create AppDir structure
APP_NAME="GraphLearning"
APP_DIR="../../dist/linux/$APP_NAME.AppDir"

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

# Copy binary
cp target/release/xilem_abcdeez_desktop "$APP_DIR/usr/bin/$APP_NAME"

# Create desktop entry
cat > "$APP_DIR/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Graph Learning
Exec=GraphLearning
Icon=GraphLearning
Categories=Education;Science;
Comment=Adaptive graph-coded learning system
EOF

# Create a simple icon (you should replace this with a real icon)
convert -size 256x256 xc:blue -fill white -draw "circle 128,128 128,64" \
    "$APP_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png" 2>/dev/null || \
    echo "ImageMagick not found, skipping icon creation"

# Create AppRun script
cat > "$APP_DIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/GraphLearning" "$@"
EOF

chmod +x "$APP_DIR/AppRun"

# Download appimagetool if not present
APPIMAGETOOL="../../tools/appimagetool-x86_64.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool..."
    mkdir -p ../../tools
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
        -O "$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
fi

# Create AppImage
echo "Creating AppImage..."
ARCH=x86_64 "$APPIMAGETOOL" "$APP_DIR" "../../dist/linux/$APP_NAME-x86_64.AppImage"

echo "Linux AppImage built successfully!"
echo "AppImage location: dist/linux/$APP_NAME-x86_64.AppImage"