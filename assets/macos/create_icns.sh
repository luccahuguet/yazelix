#!/bin/bash
# Script to create macOS .icns icon from PNG images
# Run this script on macOS to generate the yazelix.icns icon file

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICON_DIR="$SCRIPT_DIR/../icons"
ICONSET_DIR="$SCRIPT_DIR/yazelix.iconset"
APP_RESOURCES="$SCRIPT_DIR/Yazelix.app/Contents/Resources"

echo "Creating iconset directory..."
mkdir -p "$ICONSET_DIR"

# Copy and rename PNG files to the iconset format required by macOS
# macOS iconset requires specific naming: icon_<size>x<size>.png and icon_<size>x<size>@2x.png
echo "Copying icon files..."
cp "$ICON_DIR/48x48/yazelix.png" "$ICONSET_DIR/icon_24x24@2x.png"    # 48x48 for 24pt @2x
cp "$ICON_DIR/64x64/yazelix.png" "$ICONSET_DIR/icon_32x32@2x.png"    # 64x64 for 32pt @2x
cp "$ICON_DIR/128x128/yazelix.png" "$ICONSET_DIR/icon_128x128.png"   # 128x128 for 128pt
cp "$ICON_DIR/256x256/yazelix.png" "$ICONSET_DIR/icon_128x128@2x.png" # 256x256 for 128pt @2x
cp "$ICON_DIR/256x256/yazelix.png" "$ICONSET_DIR/icon_256x256.png"   # 256x256 for 256pt

# Also add smaller sizes for better compatibility
cp "$ICON_DIR/48x48/yazelix.png" "$ICONSET_DIR/icon_16x16@2x.png"    # Alternative usage

echo "Converting iconset to icns..."
iconutil -c icns "$ICONSET_DIR" -o "$APP_RESOURCES/yazelix.icns"

echo "Cleaning up..."
rm -rf "$ICONSET_DIR"

echo "✅ Icon created successfully at: $APP_RESOURCES/yazelix.icns"
echo ""
echo "Note: This icon is optional. The .app will work without it,"
echo "but having it provides a better visual experience in Finder and Spotlight."
