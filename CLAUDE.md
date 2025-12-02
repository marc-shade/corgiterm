# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**CorgiTerm** is an AI-powered terminal emulator built with Rust, GTK4, and libadwaita. It targets Linux primarily, with planned cross-platform support via `portable-pty`.

## Build & Development Commands

```bash
# Build and run
cargo build --release
./target/release/corgiterm

# Development with debug output
RUST_LOG=debug cargo run

# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p corgiterm-core
cargo test -p corgiterm-ai

# Run a single test
cargo test -p corgiterm-terminal test_process_simple_text

# Check for issues
cargo clippy --workspace
cargo fmt --all -- --check

# GTK Inspector (debug UI issues)
GTK_DEBUG=interactive ./target/release/corgiterm

# Specific crate logging
RUST_LOG=corgiterm_ai=debug cargo run
```

## Build Dependencies

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel vte291-gtk4-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev
```

## Architecture

### Workspace Crates

| Crate | Purpose |
|-------|---------|
| `corgiterm-core` | PTY spawning (Linux-only), session persistence, safe mode, command history |
| `corgiterm-ui` | GTK4/libadwaita interface, window management, AI panel, sidebar, tabs |
| `corgiterm-ai` | AI provider integration (Claude, Gemini, Ollama, OpenAI), NL→command |
| `corgiterm-config` | Configuration schema, keyboard shortcuts, theme definitions, snippets |
| `corgiterm-plugins` | WASM and Lua plugin runtimes |
| `corgiterm-terminal` | High-performance terminal backend: grid, VTE parser, GPU renderer, damage tracking |

### Global State Pattern

`app.rs` uses static `OnceLock` singletons accessed via helper functions:
- `config_manager()` - Configuration
- `session_manager()` - Project sessions
- `ai_manager()` - AI providers
- `plugin_manager()` - Plugins
- `snippets_manager()` - Command snippets
- `history_store()` - Command history for AI learning

### Async AI Pattern

AI calls use a GTK4-compatible async pattern since GTK is single-threaded:

1. Create `crossbeam_channel::unbounded()` for results
2. Spawn `std::thread` that creates a tokio runtime
3. Execute async AI call in the tokio runtime
4. Send result through channel
5. Poll channel in `glib::timeout_add_local` (GTK main loop)

See `ai_panel.rs:360-510` for the complete implementation.

### AI Provider Priority

`app.rs:init_ai()` auto-detects providers in order:
1. CLI tools (no API key): `claude` CLI, `gemini` CLI
2. Local Ollama (connectivity check via curl)
3. API key providers: Claude, OpenAI, Gemini

### AI Panel Modes

Three-mode tabbed interface (`ai_panel.rs`):

| Mode | Purpose | Inspiration |
|------|---------|-------------|
| Chat | Conversational AI | Cursor Cmd+L |
| Explain | Understand commands/errors | GitHub Copilot |
| Command | Natural language → shell | Warp # prefix |

### Terminal Backend

`corgiterm-terminal` is the high-performance backend (inspired by foot/Alacritty):
- **Grid**: 2D cell grid with damage tracking
- **Parser**: VTE-based escape sequence parsing
- **Renderer**: GPU-accelerated via wgpu/glyphon
- **Health**: Automatic recovery with soft/hard reset

## Display Server Compatibility

CorgiTerm works on both **Wayland** and **X11**:
- Uses `gio::ApplicationFlags::NON_UNIQUE` to avoid D-Bus registration timeouts on X11 desktop environments (Budgie, XFCE, etc.)
- Clipboard uses GTK4's cross-platform `display.clipboard()` API
- No display-server-specific code required

## Known Issues

### GTK4 Deprecations
These widgets are deprecated since GTK 4.10 and need migration:
- `ComboBoxText` → `DropDown`
- `FileChooserDialog` → `FileDialog`
- `ColorButton` → `ColorDialogButton`
- `style_context()` → CSS classes directly

### Platform Limitations
- PTY implementation is Linux-only (uses `openpty`, `fork`, `nix` crate)
- VTE terminal widget is GNOME-specific
- Cross-platform requires `portable-pty` abstraction

## Configuration

Location: `~/.config/corgiterm/config.toml`

Key sections: `[general]`, `[appearance]`, `[ai]`, `[ai.local]`, `[safe_mode]`

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Shift+A` | Toggle AI panel |
| `Ctrl+Shift+H/D` | Split horizontal/vertical |
| `Ctrl+Shift+M` | SSH Manager |
| `Ctrl+Shift+S` | Snippets Library |
| `Ctrl+K` | Quick switcher |
