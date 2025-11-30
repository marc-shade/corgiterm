# CorgiTerm Flatpak Build

This directory contains the Flatpak packaging for CorgiTerm.

## Prerequisites

```bash
# Fedora
sudo dnf install flatpak flatpak-builder

# Ubuntu/Debian
sudo apt install flatpak flatpak-builder

# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install GNOME SDK (runtime version 46)
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08
```

## Generate Cargo Sources

Before building, you need to generate the `cargo-sources.json` file:

```bash
# Install flatpak-cargo-generator
pip install aiohttp toml

# Download the generator script
curl -O https://raw.githubusercontent.com/nicholasbishop/nickel/main/nickel-flatpak/nickel_flatpak/flatpak_cargo_generator.py

# Or use the one from flatpak-builder-tools
git clone https://github.com/nicholasbishop/nickel.git
cd nickel/nickel-flatpak

# Generate from project root
cd /path/to/corgiterm
python3 flatpak_cargo_generator.py Cargo.lock -o flatpak/cargo-sources.json
```

## Build

```bash
cd flatpak

# Build locally (for testing)
flatpak-builder --user --install --force-clean build-dir dev.corgiterm.CorgiTerm.yml

# Build for distribution
flatpak-builder --repo=repo --force-clean build-dir dev.corgiterm.CorgiTerm.yml
flatpak build-bundle repo corgiterm.flatpak dev.corgiterm.CorgiTerm
```

## Run

```bash
flatpak run dev.corgiterm.CorgiTerm
```

## Validate AppStream Data

```bash
flatpak run org.freedesktop.appstream-glib validate dev.corgiterm.CorgiTerm.metainfo.xml
```

## Directory Structure

```
flatpak/
├── dev.corgiterm.CorgiTerm.yml          # Flatpak manifest
├── dev.corgiterm.CorgiTerm.desktop      # Desktop entry
├── dev.corgiterm.CorgiTerm.metainfo.xml # AppStream metadata
├── cargo-sources.json                    # Generated cargo dependencies
├── icons/
│   └── hicolor/
│       └── scalable/
│           └── apps/
│               └── dev.corgiterm.CorgiTerm.svg
└── README.md                             # This file
```

## Publishing to Flathub

1. Fork https://github.com/flathub/flathub
2. Create a new branch with app ID as name
3. Add manifest and required files
4. Submit pull request
5. Wait for review and approval

See: https://github.com/flathub/flathub/wiki/App-Submission
