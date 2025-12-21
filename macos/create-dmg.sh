#!/bin/bash
# CorgiTerm DMG Installer Creator
# Creates a drag-to-install DMG with Applications folder shortcut

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

# Determine architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    ARCH_SUFFIX="apple-silicon"
else
    ARCH_SUFFIX="intel"
fi

DMG_NAME="CorgiTerm-${VERSION}-macos-${ARCH_SUFFIX}"
DMG_PATH="$BUILD_DIR/$DMG_NAME.dmg"
DMG_TEMP="$BUILD_DIR/dmg-temp"

echo "=== Creating CorgiTerm DMG Installer ==="
echo "Version: $VERSION"
echo "Architecture: $ARCH_SUFFIX"

# Check if app bundle exists
if [ ! -d "$APP_BUNDLE" ]; then
    echo "App bundle not found. Building first..."
    "$SCRIPT_DIR/build-app.sh"
fi

# Clean previous DMG files
rm -rf "$DMG_TEMP"
rm -f "$DMG_PATH"

# Create DMG staging directory
echo "Preparing DMG contents..."
mkdir -p "$DMG_TEMP"

# Copy app bundle
cp -R "$APP_BUNDLE" "$DMG_TEMP/"

# Create Applications symlink
ln -s /Applications "$DMG_TEMP/Applications"

# Create background image directory and README
mkdir -p "$DMG_TEMP/.background"
cat > "$DMG_TEMP/README.txt" << 'HEREDOC'
CorgiTerm - AI-Powered Terminal

INSTALLATION:
Drag CorgiTerm.app to the Applications folder.

FIRST RUN:
If macOS shows "app cannot be opened", right-click and select "Open".

REQUIREMENTS:
- macOS 12 (Monterey) or later
- GTK4 and libadwaita (install via: brew install gtk4 libadwaita)

For more info: https://github.com/marc-shade/corgiterm
HEREDOC

# Calculate DMG size (app size + 50MB buffer)
APP_SIZE=$(du -sm "$APP_BUNDLE" | cut -f1)
DMG_SIZE=$((APP_SIZE + 50))

echo "Creating DMG image (${DMG_SIZE}MB)..."

# Create temporary DMG
hdiutil create -srcfolder "$DMG_TEMP" \
    -volname "$APP_NAME" \
    -fs HFS+ \
    -fsargs "-c c=64,a=16,e=16" \
    -format UDRW \
    -size ${DMG_SIZE}m \
    "$BUILD_DIR/temp.dmg"

# Mount the DMG
echo "Configuring DMG appearance..."
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "$BUILD_DIR/temp.dmg" | grep "/Volumes/" | sed 's/.*\/Volumes/\/Volumes/')

# Set DMG window appearance using AppleScript
osascript << EOF
tell application "Finder"
    tell disk "$APP_NAME"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set bounds of container window to {400, 100, 900, 450}
        set theViewOptions to the icon view options of container window
        set arrangement of theViewOptions to not arranged
        set icon size of theViewOptions to 100
        set position of item "$APP_NAME.app" of container window to {125, 175}
        set position of item "Applications" of container window to {375, 175}
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
EOF

# Unmount
sync
hdiutil detach "$MOUNT_DIR"

# Convert to compressed DMG
echo "Compressing DMG..."
hdiutil convert "$BUILD_DIR/temp.dmg" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "$DMG_PATH"

# Cleanup
rm -rf "$DMG_TEMP"
rm -f "$BUILD_DIR/temp.dmg"

# Calculate checksum
CHECKSUM=$(shasum -a 256 "$DMG_PATH" | cut -d' ' -f1)

echo ""
echo "=== DMG Created Successfully ==="
echo "File: $DMG_PATH"
echo "Size: $(du -h "$DMG_PATH" | cut -f1)"
echo "SHA256: $CHECKSUM"
echo ""
echo "To install:"
echo "  1. Open $DMG_NAME.dmg"
echo "  2. Drag CorgiTerm to Applications"
echo "  3. Eject the disk image"
