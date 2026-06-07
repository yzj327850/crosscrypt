#!/bin/bash
set -e

echo "CrossCrypt Portable Packaging Script"
echo "====================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VERSION=$(grep "^version" "$PROJECT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)

cd "$PROJECT_DIR"

# Create package directory
PKG_DIR="$PROJECT_DIR/dist/crosscrypt-portable-$VERSION"
mkdir -p "$PKG_DIR"

# Build release
echo "Building release..."
cargo build --release

# Copy binaries
echo "Copying binaries..."
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    cp "$PROJECT_DIR/target/release/crosscrypt.exe" "$PKG_DIR/"
    cp "$PROJECT_DIR/portable/windows/autorun.inf" "$PKG_DIR/"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    mkdir -p "$PKG_DIR/CrossCrypt.app/Contents/MacOS"
    mkdir -p "$PKG_DIR/CrossCrypt.app/Contents/Resources"
    cp "$PROJECT_DIR/target/release/crosscrypt" "$PKG_DIR/CrossCrypt.app/Contents/MacOS/"
    cp "$PROJECT_DIR/portable/macos/CrossCrypt.app/Contents/Info.plist" "$PKG_DIR/CrossCrypt.app/Contents/"
else
    cp "$PROJECT_DIR/target/release/crosscrypt" "$PKG_DIR/"
    cp "$PROJECT_DIR/portable/linux/crosscrypt.desktop" "$PKG_DIR/"
fi

# Copy documentation
cp "$PROJECT_DIR/README.md" "$PKG_DIR/"
cp "$PROJECT_DIR/docs/SECURITY.md" "$PKG_DIR/" 2>/dev/null || true
cp "$PROJECT_DIR/docs/DESIGN.md" "$PKG_DIR/" 2>/dev/null || true

# Create archive
echo "Creating archive..."
cd "$PROJECT_DIR/dist"
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    zip -r "crosscrypt-portable-$VERSION.zip" "crosscrypt-portable-$VERSION"
else
    tar czf "crosscrypt-portable-$VERSION.tar.gz" "crosscrypt-portable-$VERSION"
fi

echo ""
echo "Package created: $PKG_DIR"
echo "Archive in: $PROJECT_DIR/dist/"
ls -la "$PROJECT_DIR/dist/"
