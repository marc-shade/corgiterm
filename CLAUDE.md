# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Build
cargo build                          # Debug build
cargo build --release                # Release build

# Run
cargo run                            # Debug run
./target/release/corgiterm           # Release run
RUST_LOG=debug cargo run             # Debug logging enabled
RUST_LOG=corgiterm_ai=debug cargo run # Crate-specific logging

# Testing
cargo test --workspace               # All tests
cargo test -p corgiterm-core         # Single crate
cargo test -- --nocapture            # With output
cargo test --test integration        # Integration tests only

# Code quality
cargo fmt --all                      # Format code
cargo fmt --all -- --check           # Check formatting
cargo clippy --workspace             # Lint

# Hot-reload development
cargo install cargo-watch
cargo watch -x run

# GTK debugging
GTK_DEBUG=interactive ./target/release/corgiterm
```

## System Dependencies

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel vte291-gtk4-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev
```

## Architecture

This is a Rust workspace with 5 crates:

| Crate | Purpose |
|-------|---------|
| `corgiterm-core` | Terminal emulation, PTY spawning, sessions, safe mode |
| `corgiterm-ui` | GTK4/libadwaita interface, window management |
| `corgiterm-ai` | AI providers (Claude CLI, Gemini CLI, Ollama, APIs) |
| `corgiterm-config` | Configuration loading/saving, theme definitions |
| `corgiterm-plugins` | WASM and Lua plugin system |

### Global Managers (`crates/corgiterm-ui/src/app.rs`)

Static singletons accessed via helper functions:
- `config_manager()` - Configuration
- `session_manager()` - Project sessions
- `ai_manager()` - AI providers
- `plugin_manager()` - Plugins

### AI Panel (`crates/corgiterm-ui/src/ai_panel.rs`)

Three-mode tabbed interface:
1. **Chat** - Conversational AI (Cursor-style)
2. **Explain** - Command/error explanation (Copilot-style)
3. **Command** - Natural language to shell (Warp-style)

Uses `crossbeam_channel` + spawned tokio thread for async AI calls, polls with `glib::timeout_add_local`.

### Provider Priority (`crates/corgiterm-ai/src/providers.rs`)

Auto-detection order:
1. CLI tools (no API key): `claude` CLI, `gemini` CLI
2. Local Ollama (connectivity checked)
3. API key providers: Claude API, OpenAI, Gemini API

### Window Layout (`crates/corgiterm-ui/src/window.rs`)

- `content_paned` = Sidebar + Terminal (horizontal Paned)
- `ai_revealer` = Slide-out AI panel (Revealer)
- Keyboard shortcuts defined around line 420

## Configuration

Location: `~/.config/corgiterm/config.toml`

Key AI settings:
```toml
[ai]
enabled = true
default_provider = "auto"  # or "claude-cli", "ollama", etc.

[ai.local]
endpoint = "http://localhost:11434"  # Ollama
model = "codellama"
```

## Known TODOs

Find with: `grep -r "TODO" --include="*.rs" crates/`

Key items:
- `ai_panel.rs:384` - Wire Execute button to terminal
- `split_pane.rs:194` - Proper pane closing
- `mcp.rs:175` - MCP tool execution

## Commit Style

Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
feat(ai): add Claude provider support
fix(terminal): handle unicode combining characters
docs: update installation instructions
```

## Detailed Documentation

See [HANDOFF.md](HANDOFF.md) for comprehensive architecture details, component relationships, and agent orchestration recommendations.
