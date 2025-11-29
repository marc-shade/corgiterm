# CorgiTerm 🐕

```
   ∩＿∩
  (・ω・)  The friendliest terminal ever.
  /　 つ   AI-powered, accessible, beautiful.
```

**CorgiTerm** is a next-generation, AI-powered terminal emulator that makes the command line accessible to everyone - from nervous beginners to power users who demand maximum control.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4.x-green.svg)](https://gtk.org/)

## Features

### 🎯 For Everyone
- **Safe Mode** - Preview commands before execution with risk assessment
- **Natural Language Input** - Type "show large files" instead of memorizing syntax
- **Modern UI** - Clean, beautiful interface inspired by the best productivity apps
- **Project Organization** - Group terminals by project, just like your IDE

### 🤖 AI-Powered
- **Claude, OpenAI, Gemini, Local LLMs** - Choose your AI provider
- **Command Translation** - Natural language → shell commands
- **Smart Explanations** - Understand what commands do before running them
- **MCP Protocol** - Native support for AI agent tools

### ⚡ For Power Users
- **GPU Rendering** - Silky smooth 144fps performance
- **500+ Settings** - Customize everything
- **Plugin System** - WASM and Lua extensibility
- **SSH Manager** - Visual connection management
- **Searchable History** - Never lose output again

## Installation

### From Source (Linux)

```bash
# Install dependencies (Fedora)
sudo dnf install gtk4-devel libadwaita-devel

# Build
git clone https://github.com/corgiterm/corgiterm
cd corgiterm
cargo build --release

# Run
./target/release/corgiterm
```

### Flatpak (Coming Soon)
```bash
flatpak install dev.corgiterm.CorgiTerm
```

## Quick Start

```bash
# Launch CorgiTerm
corgiterm

# Open a project directory
corgiterm --project ~/projects/myapp

# Start with Safe Mode enabled
corgiterm --safe-mode

# Execute a command
corgiterm -e "npm run dev"
```

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New Tab | `Ctrl+T` |
| Close Tab | `Ctrl+W` |
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` |
| Zoom In | `Ctrl++` or `Ctrl+=` |
| Zoom Out | `Ctrl+-` |
| Reset Zoom | `Ctrl+0` |
| Search in Terminal | `Ctrl+Shift+F` |
| Next Match | `Enter` (in search) |
| Previous Match | `Shift+Enter` (in search) |
| Close Search | `Escape` |
| Switch to Tab 1-9 | `Ctrl+1` to `Ctrl+9` |
| Next Tab | `Ctrl+Tab` |
| Previous Tab | `Ctrl+Shift+Tab` |
| Open File | `Ctrl+Shift+O` |
| Quit | `Ctrl+Q` |

## Configuration

Configuration lives in `~/.config/corgiterm/config.toml`:

```toml
[general]
shell = "/bin/zsh"
restore_sessions = true

[appearance]
theme = "Corgi Dark"
font_family = "Source Code Pro"
font_size = 11.0
opacity = 1.0

[ai]
enabled = true
default_provider = "claude"
natural_language = true

[safe_mode]
enabled = false
preview_dangerous_only = true
```

## Themes

CorgiTerm comes with the beautiful "Corgi Collection" themes:

- **Corgi Dark** - Warm, cozy dark theme (default)
- **Corgi Light** - Clean, readable light theme
- **Corgi Sunset** - Warm pinks and oranges for evening coding
- **Pembroke** - Regal theme inspired by the Pembroke Welsh Corgi

## Architecture

```
corgiterm/
├── crates/
│   ├── corgiterm-core/     # Terminal emulation, PTY, sessions
│   ├── corgiterm-ui/       # GTK4/libadwaita interface
│   ├── corgiterm-ai/       # AI provider integration
│   ├── corgiterm-config/   # Configuration management
│   └── corgiterm-plugins/  # WASM/Lua plugin system
└── src/main.rs             # Application entry point
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone
git clone https://github.com/corgiterm/corgiterm
cd corgiterm

# Install dev dependencies
cargo install cargo-watch

# Run with hot-reload
cargo watch -x run

# Run tests
cargo test --workspace

# Format code
cargo fmt --all
```

## Mascot

Meet **Pixel**, our NES-style tri-color Corgi mascot! 🐕

```
   ∩＿∩
  (・ω・)  "Woof! Let me help you with that command!"
  /　 つ
```

## Privacy Promise

CorgiTerm respects your privacy:

- ✅ **No login required** - Use immediately
- ✅ **No telemetry by default** - Opt-in only
- ✅ **Open source** - Audit the code yourself
- ✅ **Local-first** - AI can run locally with Ollama

## License

CorgiTerm is dual-licensed under MIT and Apache 2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/), [GTK4](https://gtk.org/), and [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- Terminal emulation powered by [VTE](https://gitlab.gnome.org/GNOME/vte)
- Inspired by [iTerm2](https://iterm2.com/), [Warp](https://warp.dev/), and [Alacritty](https://alacritty.org/)

---

**Made with ❤️ by the CorgiTerm Team**
