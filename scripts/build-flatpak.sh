#!/bin/bash
# Build CorgiTerm Flatpak package
#
# Prerequisites:
#   flatpak install flathub org.gnome.Sdk//47 org.gnome.Platform//47
#   flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
#
# This script builds and optionally installs the CorgiTerm Flatpak

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/flatpak-build"
REPO_DIR="$PROJECT_DIR/flatpak-repo"

cd "$PROJECT_DIR"

echo "=== Building CorgiTerm Flatpak ==="
echo "Project directory: $PROJECT_DIR"

# Check for required tools
if ! command -v flatpak-builder &> /dev/null; then
    echo "Error: flatpak-builder not found. Install with:"
    echo "  sudo dnf install flatpak-builder  # Fedora"
    echo "  sudo apt install flatpak-builder  # Ubuntu/Debian"
    exit 1
fi

# Check for required runtimes
echo "Checking for required Flatpak runtimes..."
if ! flatpak info org.gnome.Sdk//47 &> /dev/null; then
    echo "Installing GNOME SDK 47..."
    flatpak install -y flathub org.gnome.Sdk//47
fi

if ! flatpak info org.gnome.Platform//47 &> /dev/null; then
    echo "Installing GNOME Platform 47..."
    flatpak install -y flathub org.gnome.Platform//47
fi

if ! flatpak info org.freedesktop.Sdk.Extension.rust-stable//24.08 &> /dev/null; then
    echo "Installing Rust SDK extension..."
    flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
fi

# Build options
INSTALL_USER=""
if [[ "$1" == "--install" ]]; then
    INSTALL_USER="--user --install"
    echo "Will install after build"
fi

echo "Building..."
flatpak-builder --force-clean $INSTALL_USER "$BUILD_DIR" dev.corgiterm.CorgiTerm.yml

echo ""
echo "=== Build complete! ==="

if [[ -z "$INSTALL_USER" ]]; then
    echo ""
    echo "To install locally, run:"
    echo "  $0 --install"
    echo ""
    echo "Or manually:"
    echo "  flatpak-builder --user --install --force-clean $BUILD_DIR dev.corgiterm.CorgiTerm.yml"
fi

echo ""
echo "To run:"
echo "  flatpak run dev.corgiterm.CorgiTerm"
