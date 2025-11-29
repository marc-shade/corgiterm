# CorgiTerm Developer Handoff

**Date:** November 29, 2025
**Last Developer:** Claude (AI Assistant)
**Project Status:** Active Development - Alpha

---

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/corgiterm

# Development build with debug output
RUST_LOG=debug cargo run
```

## Project Architecture

```
corgiterm/
├── src/main.rs                 # Entry point - GTK4 app initialization
├── crates/
│   ├── corgiterm-core/         # Terminal emulation, PTY, sessions
│   │   ├── terminal.rs         # VTE terminal wrapper
│   │   ├── pty.rs              # PTY spawning and management
│   │   ├── session.rs          # Session/project persistence
│   │   ├── safe_mode.rs        # Command safety analysis
│   │   └── history.rs          # Command history
│   │
│   ├── corgiterm-ui/           # GTK4/libadwaita interface
│   │   ├── app.rs              # Global managers (config, AI, plugins, sessions)
│   │   ├── window.rs           # Main window layout and keyboard shortcuts
│   │   ├── ai_panel.rs         # AI assistant panel (Chat/Explain/Command modes)
│   │   ├── sidebar.rs          # Project folder sidebar
│   │   ├── tab_bar.rs          # Terminal tabs management
│   │   ├── terminal_view.rs    # Terminal widget with VTE
│   │   ├── dialogs.rs          # Settings/preferences dialogs
│   │   ├── theme.rs            # CSS theme loading
│   │   ├── split_pane.rs       # Split pane support (WIP)
│   │   └── widgets/            # Custom widgets
│   │       ├── natural_language_input.rs  # (Deprecated - use AI panel)
│   │       ├── safe_mode_preview.rs       # Command preview before execution
│   │       └── session_thumbnail.rs       # Terminal thumbnails
│   │
│   ├── corgiterm-ai/           # AI provider integration
│   │   ├── lib.rs              # AiManager, Message, Role types
│   │   ├── providers.rs        # All AI providers (Claude, Gemini, Ollama, etc.)
│   │   ├── completions.rs      # Completion types
│   │   ├── natural_language.rs # NL to command translation patterns
│   │   └── mcp.rs              # MCP protocol support (WIP)
│   │
│   ├── corgiterm-config/       # Configuration management
│   │   ├── lib.rs              # ConfigManager, config loading/saving
│   │   ├── schema.rs           # All config structs (serde)
│   │   └── themes.rs           # Built-in theme definitions
│   │
│   └── corgiterm-plugins/      # Plugin system
│       ├── lib.rs              # PluginManager
│       ├── api.rs              # Plugin API definitions
│       ├── loader.rs           # Plugin discovery
│       ├── lua_runtime.rs      # Lua plugin support
│       └── wasm_runtime.rs     # WASM plugin support (WIP)
│
├── assets/
│   ├── themes/                 # Theme JSON files
│   ├── icons/                  # App icons
│   └── fonts/                  # Bundled fonts
│
└── docs/                       # Documentation (sparse)
```

## Key Components

### 1. AI Panel (`ai_panel.rs`)

The AI panel has **three distinct modes** inspired by modern AI-powered terminals:

| Mode | Inspiration | Purpose |
|------|-------------|---------|
| **Chat** | Cursor Cmd+L | Conversational AI assistant |
| **Explain** | GitHub Copilot /explain | Understand commands/errors |
| **Command** | Warp's # prefix | Natural language → shell command |

**Tab order:** Chat → Explain → Command

**How it works:**
- Uses `crossbeam_channel` for async AI calls
- Spawns thread with tokio runtime for async provider calls
- Polls with `glib::timeout_add_local` to update UI
- Toggle with header button or `Ctrl+Shift+A`

### 2. AI Providers (`providers.rs`)

**Provider priority (auto-detection):**
1. CLI tools (no API key needed): `claude` CLI, `gemini` CLI
2. Local Ollama (connectivity check via curl)
3. API key providers: Claude API, OpenAI, Gemini API

**Available providers:**
- `ClaudeCliProvider` - Uses `claude` command (OAuth-based)
- `GeminiCliProvider` - Uses `gemini` command (OAuth-based)
- `OllamaProvider` - Local/remote Ollama server
- `ClaudeProvider` - Anthropic API with key
- `OpenAiProvider` - OpenAI API with key
- `GeminiProvider` - Google AI API with key

**Known issues:**
- Claude CLI outputs errors to stdout, not stderr (fixed)
- Gemini CLI `-p` flag deprecated, use positional args (fixed)

### 3. Configuration (`config.toml`)

Location: `~/.config/corgiterm/config.toml`

```toml
[ai]
enabled = true
default_provider = "auto"  # or "claude-cli", "ollama", "claude", etc.

[ai.local]
enabled = true
endpoint = "http://localhost:11434"  # Ollama server
model = "codellama"

[ai.claude]
model = "claude-sonnet-4-20250514"
# api_key = "sk-..."  # Optional

[ai.openai]
model = "gpt-4o"
# api_key = "sk-..."  # Optional

[ai.gemini]
model = "gemini-2.0-flash"
# api_key = "..."  # Optional
```

### 4. Window Layout (`window.rs`)

```
┌─────────────────────────────────────────────────────────────┐
│ Header Bar                                    [AI Toggle]   │
├─────────────────────────────────────────────────────────────┤
│ Tab Bar                                                     │
├────────────┬────────────────────────────────┬───────────────┤
│            │                                │               │
│  Sidebar   │      Terminal View             │   AI Panel    │
│ (Projects) │                                │  (Revealer)   │
│            │                                │               │
│            ├────────────────────────────────┤               │
│            │ Safe Mode Preview (when shown) │               │
└────────────┴────────────────────────────────┴───────────────┘
```

**Key layout code:**
- `content_paned` = Sidebar + Terminal area (horizontal Paned)
- `ai_revealer` = Slide-out AI panel (Revealer widget)
- `content_box` = content_paned + ai_revealer (horizontal Box)

### 5. Global Managers (`app.rs`)

Static singletons initialized at startup:

```rust
CONFIG_MANAGER: OnceLock<Arc<RwLock<ConfigManager>>>
SESSION_MANAGER: OnceLock<Arc<RwLock<SessionManager>>>
AI_MANAGER: OnceLock<Arc<RwLock<AiManager>>>
PLUGIN_MANAGER: OnceLock<Arc<RwLock<PluginManager>>>
```

Access via: `config_manager()`, `session_manager()`, `ai_manager()`, `plugin_manager()`

## Recent Changes (Nov 29, 2025)

1. **AI Panel Redesign**
   - Three-mode tabbed interface (Chat/Explain/Command)
   - Removed bottom "Type naturally" input field
   - Removed sidebar "AI Assistant" section
   - All AI interaction now in right panel

2. **Provider Auto-Detection**
   - Priority: CLI → Ollama → API keys
   - `default_provider = "auto"` finds best available
   - Ollama connectivity check before adding

3. **CLI Provider Fixes**
   - Claude CLI: Capture stdout for error messages
   - Gemini CLI: Use positional args (not `-p`)

4. **UI Cleanup**
   - Panel collapse gives terminal full width
   - Fixed ampersand markup warning in settings

## Known TODOs

Search codebase for `TODO:` comments:

```bash
grep -r "TODO" --include="*.rs" crates/
```

**Key TODOs:**
- `ai_panel.rs:384` - Wire Execute button to terminal
- `ai_panel.rs:390` - Wire Explain button to switch modes
- `split_pane.rs:194` - Proper pane closing with tree restructuring
- `mcp.rs:175` - Implement MCP tool execution
- `terminal_view.rs:1350` - Read /proc/<pid>/cwd for shell process

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p corgiterm-core
cargo test -p corgiterm-ai
```

**Note:** Test coverage is minimal. Most testing has been manual.

## Build Dependencies

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel vte291-gtk4-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev
```

## Keyboard Shortcuts

Defined in `window.rs` around line 420:

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Shift+A` | Toggle AI panel |
| `Ctrl+K` | Quick switcher |
| `Ctrl+Shift+C` | Copy |
| `Ctrl+Shift+V` | Paste |
| `Ctrl++` / `Ctrl+=` | Zoom in |
| `Ctrl+-` | Zoom out |
| `Ctrl+0` | Reset zoom |
| `Ctrl+Q` | Quit |

## Configuration Files

| File | Purpose |
|------|---------|
| `~/.config/corgiterm/config.toml` | Main configuration |
| `~/.config/corgiterm/sessions.json` | Saved projects |
| `~/.config/corgiterm/plugins/` | Plugin directory |

## Debugging

```bash
# Enable all debug logging
RUST_LOG=trace ./target/release/corgiterm

# Enable specific crate logging
RUST_LOG=corgiterm_ai=debug ./target/release/corgiterm

# GTK Inspector
GTK_DEBUG=interactive ./target/release/corgiterm
```

## Code Style

- Use `cargo fmt --all` before committing
- Use `cargo clippy --workspace` to check for issues
- Follow existing patterns in codebase

## Git Workflow

```bash
# Current branch
git branch  # main

# Recent commits
git log --oneline -10
```

## Roadmap (from README)

- [ ] ASCII Art Generator
- [ ] Flatpak Package
- [ ] SSH Manager
- [ ] Split Panes (partial implementation exists)
- [ ] Snippets Library
- [ ] AI Command History
- [ ] Theme Creator

## Recommended Agent Team

For efficient development on CorgiTerm, assemble a team of specialized agents:

### Core Team (Always Active)

| Agent | Role | Focus Areas |
|-------|------|-------------|
| **System Architect** | Design & Architecture | GTK4 widget hierarchy, async patterns, crate organization |
| **Coder** | Implementation | Rust code, trait implementations, async/await, GTK4 bindings |
| **Code Reviewer** | Quality Assurance | Safety, idiomatic Rust, memory management, GTK lifecycle |

### Specialized Agents (Task-Dependent)

| Agent | When to Use | Example Tasks |
|-------|-------------|---------------|
| **Deep Debugger** | Runtime issues | GTK signal debugging, async channel issues, VTE problems |
| **Researcher** | New integrations | MCP protocol, new AI providers, WASM plugin system |
| **Tester** | Quality gates | Integration tests, GTK widget testing strategies |
| **UI/UX (frontend-design)** | Visual polish | Theme refinement, CSS styling, layout optimization |

### Agent Orchestration Patterns

**For Feature Development:**
```
1. System Architect → Design widget structure and data flow
2. Coder → Implement the feature
3. Code Reviewer → Review before merge
4. Tester → Validate functionality
```

**For Bug Fixes:**
```
1. Deep Debugger → Root cause analysis
2. Coder → Implement fix
3. Code Reviewer → Verify fix doesn't break other things
```

**For AI Provider Integration:**
```
1. Researcher → Study API patterns and authentication
2. System Architect → Design provider abstraction
3. Coder → Implement provider
4. Tester → Test edge cases (rate limits, errors, timeouts)
```

### Key Technical Skills Needed

- **Rust**: Async/await, trait objects, lifetimes, Arc/Mutex patterns
- **GTK4**: Widget lifecycle, signal handling, GLib main loop integration
- **libadwaita**: Modern GNOME patterns, responsive layouts
- **Tokio**: Runtime spawning, channels, async I/O
- **AI APIs**: REST clients, streaming responses, error handling

### Swarm Configuration Suggestion

For complex tasks like implementing Split Panes or SSH Manager:

```yaml
topology: hierarchical
coordinator: system-architect
workers:
  - coder (2 instances for parallel file work)
  - code-reviewer
  - deep-debugger (on standby)
memory: shared (use enhanced-memory MCP for context)
```

## Contact

Project repo: https://github.com/marc-shade/corgiterm

---

*Last updated: November 29, 2025*
