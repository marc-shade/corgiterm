
![corgiterm](https://github.com/user-attachments/assets/a444f023-d409-4434-911e-03f40089f81c)

# CorgiTerm

**The terminal that teaches you as you go.**

For people who know their way around a computer but find the command line intimidating. CorgiTerm bridges the gap between powerful CLI tools and the friendly, visual interfaces you're used to.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4.x-green.svg)](https://gtk.org/)

---

## The Problem: Staring at a Blank Canvas

You've heard that the command line is powerful. You've seen developers type cryptic commands that somehow do amazing things. But when you open a terminal, you're greeted by... nothing. A blinking cursor. A blank canvas.

**Where do you even start?**

- "What if I break something?"
- "I don't know what to type"
- "How do I even know what's possible?"

CorgiTerm solves this.

---

## How CorgiTerm Makes CLI Accessible

### 1. Safe Mode: Review Before You Run

Every command is previewed before execution. See exactly what will happen, with risk assessment and undo suggestions.

```
+------------------------------------------------------------------+
|  Safe Mode Preview                                               |
+------------------------------------------------------------------+
|                                                                  |
|  Command: rm -rf ./node_modules                                  |
|                                                                  |
|  CAUTION - This will permanently delete files                    |
|                                                                  |
|  What it does:                                                   |
|  - Recursively removes the 'node_modules' directory              |
|  - Files will be permanently deleted (not moved to trash)        |
|                                                                  |
|  Safer alternative: trash-put ./node_modules (recoverable)       |
|  To restore: npm install                                         |
|                                                                  |
|  [ Execute ]  [ Cancel ]                                         |
+------------------------------------------------------------------+
```

**No more "oh no, what did I just do?"** Safe Mode catches dangerous operations before they happen, explains what commands will do, and suggests safer alternatives.

### 2. Just Ask: Type What You Want, Not What You Know

Don't know the command? Just describe what you want in plain English:

| You type... | CorgiTerm translates to... |
|-------------|---------------------------|
| "show files bigger than 1GB" | `find . -size +1G -type f` |
| "what's using port 3000" | `lsof -i :3000` |
| "count lines in all python files" | `find . -name "*.py" \| xargs wc -l` |
| "compress this folder" | `tar -czvf folder.tar.gz folder/` |

The AI shows you the command, explains it, and you decide whether to run it. **You learn the actual commands by seeing what your natural language becomes.**

### 3. AI That Learns Your Style

CorgiTerm observes how you work and adapts:

- **Remembers your preferences**: Uses `exa` if you prefer it over `ls`
- **Suggests based on patterns**: Knows you usually run `git status` after `git add`
- **Gets smarter over time**: More relevant suggestions the more you use it

### 4. Snippets Library: Your Personal Command Cookbook

Save commands you'll use again with variables for the parts that change:

```bash
# Docker: Run container with port mapping
docker run -d -p {{host_port}}:{{container_port}} --name {{name}} {{image}}

# Git: Create feature branch
git checkout -b feature/{{branch_name}} && git push -u origin feature/{{branch_name}}

# SSH: Connect to server
ssh {{user|username}}@{{host:192.168.1.100}} -p {{port:22}}
```

Never google "how to do X in the terminal" twice. Build your personal library and reuse commands with `Ctrl+Shift+S`.

---

## What Makes CorgiTerm Different

| Feature | CorgiTerm | Warp | iTerm2 | GNOME Terminal |
|---------|-----------|------|--------|----------------|
| Safe Mode (preview before run) | **Yes** | No | No | No |
| Natural language commands | **Yes** | Yes | No | No |
| Learns your preferences | **Yes** | Limited | No | No |
| Works offline (local AI) | **Yes** | No | N/A | N/A |
| No account required | **Yes** | No | Yes | Yes |
| Open source | **Yes** | No | Yes | Yes |
| Snippets with variables | **Yes** | Workflows | No | No |
| AI explains errors | **Yes** | Yes | No | No |

### Why Not Just Use Warp?

Warp is great, but:
- **Requires login** to use AI features
- **Closed source** - you can't see what it's doing
- **Cloud-dependent** - AI features don't work offline
- **macOS only** (Linux in beta)

CorgiTerm is **open source, runs AI locally with Ollama, requires no account**, and works on Linux and macOS today.

---

## Getting Started

### Install (Linux)

```bash
# Fedora
sudo dnf install gtk4-devel libadwaita-devel lua-devel

# Ubuntu/Debian
sudo apt install libgtk-4-dev libadwaita-1-dev liblua5.4-dev

# Build and run
git clone https://github.com/marc-shade/corgiterm
cd corgiterm
cargo build --release
./target/release/corgiterm
```

### Install (macOS)

```bash
# Install dependencies via Homebrew
brew install gtk4 libadwaita lua@5.4

# Set up pkg-config path for Lua
export PKG_CONFIG_PATH="/opt/homebrew/opt/lua@5.4/lib/pkgconfig:$PKG_CONFIG_PATH"

# Build and run
git clone https://github.com/marc-shade/corgiterm
cd corgiterm
cargo build --release
./target/release/corgiterm
```

### First Steps

1. **Open CorgiTerm** - Safe Mode is on by default
2. **Press `Ctrl+Shift+A`** to open the AI panel
3. **Type naturally** in the Command tab: "show me what's in this folder"
4. **Review** the suggested command and click Execute
5. **Save useful commands** to Snippets with `Ctrl+Shift+S`

---

## The AI Panel: Three Ways to Get Help

Press `Ctrl+Shift+A` to open the AI assistant:

| Tab | What it does | When to use it |
|-----|--------------|----------------|
| **Chat** | Have a conversation about anything terminal-related | "How do I set up SSH keys?" |
| **Explain** | Paste a command or error to understand it | "What does `chmod 755` mean?" |
| **Command** | Describe what you want, get the command | "find all PDFs modified today" |

### AI Providers (Your Choice, Zero Config)

CorgiTerm auto-detects what's available:

1. **Claude CLI / Gemini CLI** - If you have these installed, they just work
2. **Ollama (local)** - Run AI on your own machine, no internet needed
3. **API keys** - Claude, OpenAI, or Gemini if you prefer

**No account required. No data sent anywhere by default.**

---

## Features for Power Users

Already comfortable with the terminal? CorgiTerm has you covered:

- **GPU-accelerated rendering** - 144fps smooth scrolling
- **Split panes** - `Ctrl+Shift+H` (horizontal) / `Ctrl+Shift+D` (vertical)
- **SSH Manager** - Visual saved connections (`Ctrl+Shift+M`)
- **Quick Switcher** - VS Code-style tab switching (`Ctrl+K`)
- **URL/Path hints** - Keyboard-driven link navigation
- **500+ settings** - Customize everything
- **WASM + Lua plugins** - Extend functionality

---

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New Tab | `Ctrl+T` |
| Close Tab | `Ctrl+W` |
| **Toggle AI Panel** | `Ctrl+Shift+A` |
| **Snippets Library** | `Ctrl+Shift+S` |
| Quick Switcher | `Ctrl+K` |
| SSH Manager | `Ctrl+Shift+M` |
| Split Horizontal | `Ctrl+Shift+H` |
| Split Vertical | `Ctrl+Shift+D` |
| Search in Terminal | `Ctrl+Shift+F` |
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` |

---

## Privacy First

- **No login required** - Start using immediately
- **No telemetry** - We don't track you
- **Local-first AI** - Run Ollama for completely offline AI
- **Open source** - Audit the code yourself
- **Your data stays yours** - Command history never leaves your machine

---

## Roadmap

### Done
- [x] Safe Mode with risk assessment
- [x] Natural language to commands
- [x] AI Chat/Explain/Command modes
- [x] Multiple AI providers (Claude, Gemini, Ollama, OpenAI)
- [x] Snippets library with variables
- [x] SSH Manager
- [x] Split panes
- [x] Theme creator
- [x] Session recording and playback
- [x] WASM and Lua plugin system
- [x] macOS support (Apple Silicon & Intel)
- [x] Cross-platform PTY (via portable-pty)
- [x] GPU-accelerated rendering (wgpu/glyphon)

### Coming Soon
- [ ] Windows support (ConPTY integration)
- [ ] Plugin marketplace / repository
- [ ] Collaborative terminals (pair programming)
- [ ] Tmux/screen session integration
- [ ] AI-powered command history search
- [ ] Custom keybinding profiles

---

## Built With

- [Rust](https://www.rust-lang.org/) - Fast and safe
- [GTK4](https://gtk.org/) + [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) - Modern cross-platform UI
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty) - Cross-platform PTY (Linux, macOS, Windows)
- [wgpu](https://wgpu.rs/) + [glyphon](https://github.com/grovesNL/glyphon) - GPU-accelerated text rendering

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Development setup
cargo install cargo-watch
cargo watch -x run

# Run tests
cargo test --workspace
```

---

## License

Dual-licensed under MIT and Apache 2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

---

**CorgiTerm** - *The terminal that teaches you as you go.*
