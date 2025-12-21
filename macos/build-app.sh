#!/bin/bash
# CorgiTerm macOS App Bundle Builder
# Creates CorgiTerm.app in the target directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/release"
APP_NAME="CorgiTerm"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
# Get version from workspace package definition
VERSION=$(grep -A1 '\[workspace.package\]' "$PROJECT_ROOT/Cargo.toml" | grep 'version' | sed 's/.*"\(.*\)".*/\1/' | tr -d '[:space:]')
if [ -z "$VERSION" ] || [ "$VERSION" = "version.workspace=true" ]; then
    VERSION="0.1.0"
fi

echo "=== Building CorgiTerm.app v$VERSION ==="

# Check for required tools
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust."
    exit 1
fi

# Set up environment for Homebrew dependencies
if [ -d "/opt/homebrew" ]; then
    # Apple Silicon
    export PKG_CONFIG_PATH="/opt/homebrew/opt/lua@5.4/lib/pkgconfig:/opt/homebrew/lib/pkgconfig:/opt/homebrew/opt/icu4c@78/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
elif [ -d "/usr/local/Homebrew" ]; then
    # Intel Mac
    export PKG_CONFIG_PATH="/usr/local/opt/lua@5.4/lib/pkgconfig:/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LIBRARY_PATH="/usr/local/lib:$LIBRARY_PATH"
fi

# Build release binary
echo "Building release binary..."
cd "$PROJECT_ROOT"
cargo build --release

if [ ! -f "$BUILD_DIR/corgiterm" ]; then
    echo "Error: Build failed - binary not found"
    exit 1
fi

# Clean previous app bundle
rm -rf "$APP_BUNDLE"

# Create app bundle structure
echo "Creating app bundle structure..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary
cp "$BUILD_DIR/corgiterm" "$APP_BUNDLE/Contents/MacOS/"

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$APP_BUNDLE/Contents/"

# Update version in Info.plist
sed -i '' "s/<string>0.1.0<\/string>/<string>$VERSION<\/string>/" "$APP_BUNDLE/Contents/Info.plist"

# Create/copy icon
if [ -f "$SCRIPT_DIR/AppIcon.icns" ]; then
    cp "$SCRIPT_DIR/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/"
elif [ -f "$PROJECT_ROOT/assets/icons/corgiterm.svg" ]; then
    echo "Converting SVG to icns..."
    "$SCRIPT_DIR/create-icns.sh" "$PROJECT_ROOT/assets/icons/corgiterm.svg" "$APP_BUNDLE/Contents/Resources/AppIcon.icns" 2>/dev/null || {
        echo "Warning: Could not create icon. Using placeholder."
        # Create a simple placeholder icon
        mkdir -p /tmp/corgiterm-icon.iconset
        for size in 16 32 64 128 256 512; do
            sips -z $size $size "$PROJECT_ROOT/assets/icons/corgiterm.svg" --out "/tmp/corgiterm-icon.iconset/icon_${size}x${size}.png" 2>/dev/null || true
        done
        iconutil -c icns /tmp/corgiterm-icon.iconset -o "$APP_BUNDLE/Contents/Resources/AppIcon.icns" 2>/dev/null || true
        rm -rf /tmp/corgiterm-icon.iconset
    }
fi

# Create PkgInfo
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

# Set executable permissions
chmod +x "$APP_BUNDLE/Contents/MacOS/corgiterm"

# Ad-hoc code sign the app bundle (prevents "corrupted" error on other Macs)
echo "Code signing app bundle..."
codesign --force --deep --sign - "$APP_BUNDLE"

# Verify signature
codesign --verify --verbose "$APP_BUNDLE" && echo "Code signature verified" || echo "Warning: Code signature verification failed"

echo ""
echo "=== Build Complete ==="
echo "App bundle created at: $APP_BUNDLE"
echo ""
echo "To install, run:"
echo "  cp -r \"$APP_BUNDLE\" /Applications/"
echo ""
echo "Or create a DMG with:"
echo "  $SCRIPT_DIR/create-dmg.sh"
