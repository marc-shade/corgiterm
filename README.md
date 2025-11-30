# CorgiTerm 🐕

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

### 🤖 AI-Powered (Inspired by Warp, Cursor, Copilot, and Cline)
- **Multiple Providers** - Claude CLI, Gemini CLI, Ollama (local/remote), or API keys
- **Three AI Modes** in a slide-out panel:
  - **Chat Mode** - Conversational AI assistant for questions and help
  - **Explain Mode** - Paste commands or errors to understand them
  - **Command Mode** - Natural language → shell commands (Warp-style # prefix)
- **Auto-Detection** - Automatically finds and uses available AI providers
- **Zero Config** - Works out of the box with Claude CLI or local Ollama

### ⚡ For Power Users
- **GPU Rendering** - Silky smooth 144fps performance
- **500+ Settings** - Customize everything
- **Plugin System** - WASM and Lua extensibility
- **SSH Manager** - Visual connection management
- **Searchable History** - Never lose output again

### 🖥️ Terminal Features
- **True Color** - Full 24-bit RGB color support (16 million colors)
- **Text Attributes** - Bold, dim, italic, underline, strikethrough, inverse
- **Scrollback Buffer** - 10,000 lines of history with mouse wheel scroll
- **URL Detection** - Automatic URL highlighting with Ctrl+click to open
- **Quick Switcher** - VS Code-style tab switching with `Ctrl+K`
- **Context Menu** - Right-click for Copy, Paste, Select All, Find
- **Bell Notifications** - Visual tab indicator for terminal bell events
- **Dynamic Titles** - Tab titles update based on current directory/command

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
| Toggle AI Panel | `Ctrl+Shift+A` |
| Quick Switcher | `Ctrl+K` |
| SSH Manager | `Ctrl+Shift+M` |
| Snippets Library | `Ctrl+Shift+S` |
| ASCII Art Generator | `Ctrl+Shift+G` |
| Split Horizontal | `Ctrl+Shift+H` |
| Split Vertical | `Ctrl+Shift+D` |
| Close Pane | `Ctrl+Shift+W` |
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
| Open URL | `Ctrl+Click` on URL |
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
default_provider = "auto"  # Auto-detect best available provider

[ai.local]
enabled = true
endpoint = "http://localhost:11434"  # Ollama server
model = "codellama"

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

## Roadmap

### v0.2.0 - Quality & Polish ✓
- [x] AI Panel with Chat/Explain/Command modes
- [x] CLI provider auto-detection (Claude CLI, Gemini CLI)
- [x] Fix GTK4 4.10 deprecation warnings
- [x] Wire AI Execute button to terminal
- [x] Split Panes (Ctrl+Shift+H/D/W)

### v0.3.0 - Cross-Platform ✓
- [x] Windows support with ConPTY (via portable-pty)
- [x] macOS support with native PTY (via portable-pty)
- [x] Flatpak packaging for Linux (see `flatpak/README.md`)

### v0.4.0 - Feature Complete ✓
- [x] SSH Manager with saved hosts (Ctrl+Shift+M)
- [x] Snippets Library (Ctrl+Shift+S)
- [x] Theme Creator with live preview
- [x] AI Command History learning
- [x] ASCII Art Generator (Ctrl+Shift+G)

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
