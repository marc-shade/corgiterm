#!/bin/bash
# CorgiTerm Icon Installation Script
# Installs app icons and desktop entry to user's local directories

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ASSETS_DIR="$PROJECT_DIR/assets"

# Determine install prefix
if [ -n "$DESTDIR" ]; then
    PREFIX="$DESTDIR"
elif [ "$(id -u)" = "0" ]; then
    PREFIX="/usr/share"
else
    PREFIX="$HOME/.local/share"
fi

ICONS_DIR="$PREFIX/icons/hicolor"
APPS_DIR="$PREFIX/applications"
METAINFO_DIR="$PREFIX/metainfo"

echo "CorgiTerm Icon Installer"
echo "========================"
echo "Installing to: $PREFIX"
echo ""

# Create directories
mkdir -p "$ICONS_DIR/scalable/apps"
mkdir -p "$ICONS_DIR/symbolic/apps"
mkdir -p "$APPS_DIR"
mkdir -p "$METAINFO_DIR"

# Install scalable icon
echo "Installing scalable icon..."
cp "$ASSETS_DIR/icons/hicolor/scalable/apps/dev.corgiterm.CorgiTerm.svg" \
   "$ICONS_DIR/scalable/apps/"

# Install symbolic icon
echo "Installing symbolic icon..."
cp "$ASSETS_DIR/icons/hicolor/symbolic/apps/dev.corgiterm.CorgiTerm-symbolic.svg" \
   "$ICONS_DIR/symbolic/apps/"

# Install desktop entry
echo "Installing desktop entry..."
cp "$ASSETS_DIR/dev.corgiterm.CorgiTerm.desktop" "$APPS_DIR/"

# Install metainfo
echo "Installing metainfo..."
cp "$ASSETS_DIR/dev.corgiterm.CorgiTerm.metainfo.xml" "$METAINFO_DIR/"

# Update icon cache if available
if command -v gtk-update-icon-cache &> /dev/null; then
    echo "Updating icon cache..."
    gtk-update-icon-cache -f -t "$ICONS_DIR" 2>/dev/null || true
fi

# Update desktop database if available
if command -v update-desktop-database &> /dev/null; then
    echo "Updating desktop database..."
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi

echo ""
echo "Installation complete!"
echo ""
echo "To verify the icon is installed, you can run:"
echo "  gtk4-icon-browser"
echo ""
echo "Or check if the icon is found:"
echo "  ls $ICONS_DIR/scalable/apps/dev.corgiterm.CorgiTerm.svg"
