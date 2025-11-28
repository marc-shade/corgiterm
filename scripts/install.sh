#!/bin/bash
# CorgiTerm installation script

set -e

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
SHAREDIR="${PREFIX}/share/corgiterm"
DESKTOPDIR="${PREFIX}/share/applications"
ICONDIR="${PREFIX}/share/icons/hicolor"

echo "🐕 Installing CorgiTerm..."

# Check if built
if [ ! -f "target/release/corgiterm" ]; then
    echo "Release build not found. Building..."
    cargo build --release
fi

# Install binary
echo "Installing binary to ${BINDIR}..."
install -Dm755 target/release/corgiterm "${BINDIR}/corgiterm"

# Install assets
echo "Installing assets to ${SHAREDIR}..."
install -Dm644 assets/themes/*.toml "${SHAREDIR}/themes/" 2>/dev/null || true

# Install desktop file
echo "Installing desktop file..."
cat > /tmp/corgiterm.desktop << EOF
[Desktop Entry]
Name=CorgiTerm
Comment=AI-Powered Terminal Emulator
Exec=corgiterm
Icon=dev.corgiterm.CorgiTerm
Terminal=false
Type=Application
Categories=System;TerminalEmulator;
Keywords=terminal;console;shell;command;
StartupWMClass=corgiterm
EOF
install -Dm644 /tmp/corgiterm.desktop "${DESKTOPDIR}/dev.corgiterm.CorgiTerm.desktop"

# Install icon (if exists)
if [ -f "assets/icons/corgiterm.svg" ]; then
    echo "Installing icon..."
    install -Dm644 assets/icons/corgiterm.svg "${ICONDIR}/scalable/apps/dev.corgiterm.CorgiTerm.svg"
fi

echo "✅ Installation complete!"
echo ""
echo "Run with: corgiterm"
echo "Uninstall with: $0 --uninstall"
