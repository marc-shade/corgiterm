# CLAUDE.md

This file provides guidance to Claude Code when working with CorgiTerm.

## Project Overview

**CorgiTerm** is a next-generation, AI-powered terminal emulator built with Rust, GTK4, and libadwaita. It makes the command line accessible to everyone - from nervous beginners to power users.

**Status:** Active Development - Alpha
**Platform Support:** Linux (primary), Windows and macOS (planned)

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/corgiterm

# Development with debug output
RUST_LOG=debug cargo run

# Run tests
cargo test --workspace

# Check for issues
cargo clippy --workspace
cargo fmt --all -- --check
```

## Architecture

### Workspace Structure

```
corgiterm/
├── src/main.rs                 # Entry point - GTK4 app initialization
├── crates/
│   ├── corgiterm-core/         # Terminal emulation, PTY, sessions
│   │   ├── pty.rs              # PTY spawning (Linux-only currently)
│   │   ├── terminal.rs         # VTE terminal wrapper
│   │   ├── session.rs          # Session/project persistence
│   │   ├── safe_mode.rs        # Command safety analysis
│   │   ├── history.rs          # Command history
│   │   ├── learning.rs         # Command learning/suggestions
│   │   └── ascii_art.rs        # ASCII art generation
│   │
│   ├── corgiterm-ui/           # GTK4/libadwaita interface
│   │   ├── app.rs              # Global managers (config, AI, plugins)
│   │   ├── window.rs           # Main window layout, keyboard shortcuts
│   │   ├── ai_panel.rs         # AI assistant (Chat/Explain/Command modes)
│   │   ├── sidebar.rs          # Project folder sidebar
│   │   ├── tab_bar.rs          # Terminal tabs management
│   │   ├── terminal_view.rs    # Terminal widget with VTE
│   │   ├── split_pane.rs       # Split pane support (partial)
│   │   ├── ssh_manager.rs      # SSH connection manager (partial)
│   │   ├── snippets.rs         # Command snippets library
│   │   ├── theme_creator.rs    # Visual theme builder
│   │   └── ascii_art_dialog.rs # ASCII art generator UI
│   │
│   ├── corgiterm-ai/           # AI provider integration
│   │   ├── providers.rs        # Claude, Gemini, Ollama, OpenAI
│   │   ├── natural_language.rs # NL to command translation
│   │   └── mcp.rs              # MCP protocol support (partial)
│   │
│   ├── corgiterm-config/       # Configuration management
│   │   ├── schema.rs           # Config structs (serde)
│   │   ├── shortcuts.rs        # Keyboard shortcut parsing
│   │   └── themes.rs           # Built-in theme definitions
│   │
│   └── corgiterm-plugins/      # Plugin system
│       ├── lua_runtime.rs      # Lua plugin support
│       └── wasm_runtime.rs     # WASM plugin support
│
├── assets/                     # Themes, icons, fonts
└── docs/                       # Documentation
```

### Key Components

#### AI Panel (`ai_panel.rs`)

Three-mode tabbed interface inspired by modern AI tools:

| Mode | Inspiration | Purpose |
|------|-------------|---------|
| Chat | Cursor Cmd+L | Conversational AI assistant |
| Explain | GitHub Copilot | Understand commands/errors |
| Command | Warp # prefix | Natural language → shell |

Uses `crossbeam_channel` + spawned tokio thread for async AI calls.

#### AI Providers (`providers.rs`)

Auto-detection priority:
1. CLI tools (no API key): `claude` CLI, `gemini` CLI
2. Local Ollama (connectivity check)
3. API key providers: Claude, OpenAI, Gemini

#### Global Managers (`app.rs`)

Static singletons accessed via helper functions:
- `config_manager()` - Configuration
- `session_manager()` - Project sessions
- `ai_manager()` - AI providers
- `plugin_manager()` - Plugins

## Build Dependencies

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel vte291-gtk4-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev
```

## Configuration

Location: `~/.config/corgiterm/config.toml`

```toml
[general]
shell = "/bin/zsh"
restore_sessions = true

[appearance]
theme = "Corgi Dark"
font_family = "Source Code Pro"
font_size = 11.0

[ai]
enabled = true
default_provider = "auto"

[ai.local]
endpoint = "http://localhost:11434"
model = "codellama"
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Shift+A` | Toggle AI panel |
| `Ctrl+K` | Quick switcher |
| `Ctrl+Shift+C/V` | Copy/Paste |
| `Ctrl++/-/0` | Zoom in/out/reset |
| `Ctrl+Q` | Quit |

## Development Roadmap

### v0.2.0 - Quality & Polish
- [x] AI Panel with Chat/Explain/Command modes
- [x] CLI provider auto-detection
- [ ] Fix GTK4 4.10 deprecation warnings
- [ ] Complete Split Panes implementation
- [ ] Wire Execute button to terminal
- [ ] Complete MCP tool execution

### v0.3.0 - Cross-Platform
- [ ] Abstract PTY layer with `portable-pty`
- [ ] Windows support with ConPTY
- [ ] macOS support with native PTY
- [ ] Flatpak packaging

### v0.4.0 - Features
- [ ] SSH Manager with saved hosts
- [ ] Snippets Library completion
- [ ] Theme Creator with live preview
- [ ] AI Command History learning

## Known Issues

### GTK4 Deprecations (54 warnings)
The following widgets are deprecated since GTK 4.10:
- `ComboBoxText` → Use `DropDown`
- `FileChooserDialog` → Use `FileDialog`
- `ColorButton` → Use `ColorDialogButton`
- `style_context()` → Use CSS classes directly

### Platform Limitations
- PTY implementation is Linux-only (uses `openpty`, `fork`, `nix` crate)
- VTE terminal widget is GNOME-specific
- Cross-platform requires `portable-pty` abstraction

## Code Style

- Run `cargo fmt --all` before committing
- Run `cargo clippy --workspace` to check for issues
- Follow Conventional Commits for commit messages
- Use system environment calls instead of hard-coded paths

## Debugging

```bash
# Full debug logging
RUST_LOG=trace ./target/release/corgiterm

# Specific crate logging
RUST_LOG=corgiterm_ai=debug cargo run

# GTK Inspector
GTK_DEBUG=interactive ./target/release/corgiterm
```

## Agent Team Recommendations

For efficient development, use specialized agents:

| Task Type | Agent | Focus |
|-----------|-------|-------|
| Feature design | System Architect | GTK4 widgets, async patterns |
| Implementation | Coder | Rust, GTK4 bindings |
| Quality | Code Reviewer | Safety, idiomatic Rust |
| Issues | Deep Debugger | GTK signals, async channels |
| Research | Researcher | New integrations, MCP |

---

*Last updated: November 30, 2025*
