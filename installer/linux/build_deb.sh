#!/usr/bin/env bash
set -euo pipefail

# Build Debian / Ubuntu package (.deb) for HyperShot
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="0.1.0"
ARCH="amd64"
PKG_NAME="hypershot_${VERSION}_${ARCH}"
BUILD_DIR="$PROJECT_ROOT/dist/$PKG_NAME"

echo "============================================================"
echo "  📦 Building HyperShot Debian (.deb) Package"
echo "============================================================"

# 1. Compile release binary if needed
if [ ! -f "$PROJECT_ROOT/target/release/hypershot" ]; then
    echo "Compiling release binary..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi

# 2. Setup Debian package layout
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$BUILD_DIR/usr/share/doc/hypershot"

# 3. Create control file
cat <<EOF > "$BUILD_DIR/DEBIAN/control"
Package: hypershot
Version: $VERSION
Section: graphics
Priority: optional
Architecture: $ARCH
Depends: libc6, libx11-6, libxcb1
Maintainer: Daniel Custodio <daniel@example.com>
Description: Blazing-fast screenshot and annotation tool
 HyperShot combines the speed of Lightshot (<15ms trigger) with
 the privacy redaction, auto-counter badges and power of Flameshot,
 plus full-page continuous vertical scroll capture.
Homepage: https://github.com/danielcustodio/hypershot
EOF

# 4. Copy files
install -m 755 "$PROJECT_ROOT/target/release/hypershot" "$BUILD_DIR/usr/bin/hypershot"
install -m 644 "$PROJECT_ROOT/installer/linux/hypershot.desktop" "$BUILD_DIR/usr/share/applications/hypershot.desktop"
install -m 644 "$PROJECT_ROOT/assets/hypershot.svg" "$BUILD_DIR/usr/share/icons/hicolor/scalable/apps/hypershot.svg"
install -m 644 "$PROJECT_ROOT/LICENSE-MIT" "$BUILD_DIR/usr/share/doc/hypershot/copyright"
install -m 644 "$PROJECT_ROOT/README.md" "$BUILD_DIR/usr/share/doc/hypershot/README.md"

# 5. Build .deb package
mkdir -p "$PROJECT_ROOT/dist"
dpkg-deb --build "$BUILD_DIR" "$PROJECT_ROOT/dist/${PKG_NAME}.deb"
rm -rf "$BUILD_DIR"

echo "✅ Generated: dist/${PKG_NAME}.deb"
