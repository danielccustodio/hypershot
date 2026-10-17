#!/usr/bin/env bash
set -euo pipefail

# Build macOS Application Bundle (.app) and Disk Image (.dmg) for HyperShot
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="0.1.0"
APP_NAME="HyperShot.app"
DMG_NAME="HyperShot-v${VERSION}-macOS.dmg"
BUILD_DIR="$PROJECT_ROOT/dist/macos-build"

echo "============================================================"
echo "  🍎 Building HyperShot macOS Bundle & DMG"
echo "============================================================"

# 1. Compile release binary if needed
if [ ! -f "$PROJECT_ROOT/target/release/hypershot" ]; then
    echo "Compiling release binary..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi

# 2. Setup macOS .app structure
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$APP_NAME/Contents/MacOS"
mkdir -p "$BUILD_DIR/$APP_NAME/Contents/Resources"

# 3. Copy binary and Info.plist
install -m 755 "$PROJECT_ROOT/target/release/hypershot" "$BUILD_DIR/$APP_NAME/Contents/MacOS/hypershot"
cp "$PROJECT_ROOT/installer/macos/Info.plist" "$BUILD_DIR/$APP_NAME/Contents/"

# If icon.png exists, generate icon.icns if sips/iconutil available
if [ -f "$PROJECT_ROOT/assets/icon.png" ] && command -v iconutil >/dev/null 2>&1; then
    ICONSET_DIR="$BUILD_DIR/AppIcon.iconset"
    mkdir -p "$ICONSET_DIR"
    sips -z 16 16     "$PROJECT_ROOT/assets/icon.png" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null
    sips -z 32 32     "$PROJECT_ROOT/assets/icon.png" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null
    sips -z 32 32     "$PROJECT_ROOT/assets/icon.png" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null
    sips -z 64 64     "$PROJECT_ROOT/assets/icon.png" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null
    sips -z 128 128   "$PROJECT_ROOT/assets/icon.png" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null
    sips -z 256 256   "$PROJECT_ROOT/assets/icon.png" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
    iconutil -c icns "$ICONSET_DIR" -o "$BUILD_DIR/$APP_NAME/Contents/Resources/AppIcon.icns"
    rm -rf "$ICONSET_DIR"
fi

# 4. Prepare DMG directory with symlink to /Applications
mkdir -p "$PROJECT_ROOT/dist"
DMG_SRC="$BUILD_DIR/dmg-source"
mkdir -p "$DMG_SRC"
cp -R "$BUILD_DIR/$APP_NAME" "$DMG_SRC/"
ln -s /Applications "$DMG_SRC/Applications"

# 5. Create DMG
rm -f "$PROJECT_ROOT/dist/$DMG_NAME"
hdiutil create -volname "HyperShot" -srcfolder "$DMG_SRC" -ov -format UDZO "$PROJECT_ROOT/dist/$DMG_NAME"
rm -rf "$BUILD_DIR"

echo "✅ Generated: dist/$DMG_NAME"
