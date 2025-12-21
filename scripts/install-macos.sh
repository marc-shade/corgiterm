#!/bin/bash
#
# CorgiTerm One-Line Installer for macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/marc-shade/corgiterm/main/scripts/install-macos.sh | bash
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

CORGI="🐕"

print_step() {
    echo -e "${BLUE}${CORGI} $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# Banner
echo ""
echo -e "${GREEN}"
echo "   ██████╗ ██████╗ ██████╗  ██████╗ ██╗████████╗███████╗██████╗ ███╗   ███╗"
echo "  ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██║╚══██╔══╝██╔════╝██╔══██╗████╗ ████║"
echo "  ██║     ██║   ██║██████╔╝██║  ███╗██║   ██║   █████╗  ██████╔╝██╔████╔██║"
echo "  ██║     ██║   ██║██╔══██╗██║   ██║██║   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║"
echo "  ╚██████╗╚██████╔╝██║  ██║╚██████╔╝██║   ██║   ███████╗██║  ██║██║ ╚═╝ ██║"
echo "   ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝"
echo -e "${NC}"
echo "  The terminal that teaches you as you go."
echo ""

# Check macOS
if [[ "$(uname)" != "Darwin" ]]; then
    print_error "This installer is for macOS only. See README for Linux instructions."
fi

# Detect architecture
ARCH=$(uname -m)
print_step "Detected architecture: $ARCH"

if [[ "$ARCH" == "arm64" ]]; then
    DMG_NAME="CorgiTerm-0.1.0-macos-apple-silicon.dmg"
    BREW_PREFIX="/opt/homebrew"
elif [[ "$ARCH" == "x86_64" ]]; then
    DMG_NAME="CorgiTerm-0.1.0-macos-intel.dmg"
    BREW_PREFIX="/usr/local"
else
    print_error "Unsupported architecture: $ARCH"
fi

# Check/Install Homebrew
print_step "Checking for Homebrew..."
if ! command -v brew &> /dev/null; then
    print_warning "Homebrew not found. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

    # Add brew to PATH for this session
    if [[ "$ARCH" == "arm64" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    else
        eval "$(/usr/local/bin/brew shellenv)"
    fi
    print_success "Homebrew installed"
else
    print_success "Homebrew found"
fi

# Install GTK4 and libadwaita
print_step "Installing GTK4 and libadwaita..."
if brew list gtk4 &>/dev/null && brew list libadwaita &>/dev/null; then
    print_success "Dependencies already installed"
else
    brew install gtk4 libadwaita
    print_success "Dependencies installed"
fi

# Get latest release URL
print_step "Finding latest release..."
RELEASE_URL="https://github.com/marc-shade/corgiterm/releases/latest/download/${DMG_NAME}"

# Download DMG
DOWNLOAD_PATH="/tmp/${DMG_NAME}"
print_step "Downloading CorgiTerm..."
if curl -fSL -o "$DOWNLOAD_PATH" "$RELEASE_URL" 2>/dev/null; then
    print_success "Downloaded ${DMG_NAME}"
else
    # Try v1.0.0 explicitly if latest redirect doesn't work
    RELEASE_URL="https://github.com/marc-shade/corgiterm/releases/download/v1.0.0/${DMG_NAME}"
    curl -fSL -o "$DOWNLOAD_PATH" "$RELEASE_URL" || print_error "Failed to download DMG"
    print_success "Downloaded ${DMG_NAME}"
fi

# Mount DMG
print_step "Installing CorgiTerm..."
MOUNT_OUTPUT=$(hdiutil attach "$DOWNLOAD_PATH" -nobrowse 2>&1)
# Extract mount point: find line with /Volumes/, take everything after last tab
MOUNT_POINT=$(echo "$MOUNT_OUTPUT" | grep '/Volumes/' | sed 's/.*	//' | tr -d '\n')

if [[ -z "$MOUNT_POINT" ]] || [[ ! -d "$MOUNT_POINT" ]]; then
    echo "Mount output:"
    echo "$MOUNT_OUTPUT"
    print_error "Failed to mount DMG - could not find mount point"
fi
print_success "Mounted at $MOUNT_POINT"

# Copy to Applications (remove old version first)
if [[ -d "/Applications/CorgiTerm.app" ]]; then
    print_warning "Removing existing installation..."
    rm -rf "/Applications/CorgiTerm.app"
fi

cp -R "${MOUNT_POINT}/CorgiTerm.app" /Applications/
print_success "Installed to /Applications"

# Unmount
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true
rm -f "$DOWNLOAD_PATH"

# Clear quarantine (macOS Gatekeeper)
print_step "Clearing quarantine flags..."
xattr -c /Applications/CorgiTerm.app 2>/dev/null || true
find /Applications/CorgiTerm.app -type f -exec xattr -c {} \; 2>/dev/null || true
print_success "Quarantine cleared"

# Success!
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}${CORGI} CorgiTerm installed successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Launch from:"
echo "    • Applications folder"
echo "    • Spotlight: Press Cmd+Space, type 'CorgiTerm'"
echo "    • Terminal: open -a CorgiTerm"
echo ""
echo "  Quick tips:"
echo "    • Ctrl+Shift+A  →  Open AI assistant"
echo "    • Ctrl+Shift+S  →  Open snippets library"
echo "    • Safe Mode is ON by default"
echo ""

# Ask to launch (only in interactive mode)
if [[ -t 0 ]]; then
    read -p "Launch CorgiTerm now? [Y/n] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        open -a CorgiTerm
        print_success "CorgiTerm launched!"
    fi
else
    echo "Run 'open -a CorgiTerm' to launch."
fi
