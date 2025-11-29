# Keyboard Shortcuts Configuration

CorgiTerm allows you to customize all keyboard shortcuts via the configuration file.

## Configuration Location

Add shortcut customizations to `~/.config/corgiterm/config.toml` under the `[keybindings.shortcuts]` section.

## Shortcut Format

Shortcuts are specified as strings with modifiers and keys separated by `+`:

```toml
[keybindings.shortcuts]
new_tab = "Ctrl+T"
toggle_ai = "Ctrl+Shift+A"
quick_switcher = "Ctrl+K"
```

### Supported Modifiers

- `Ctrl` or `Control`
- `Shift`
- `Alt` or `Meta`
- `Super`, `Win`, or `Cmd` (Windows/Command key)

### Supported Keys

**Letters**: `A-Z` (case-insensitive)

**Numbers**: `0-9`

**Function Keys**: `F1-F12`

**Special Keys**:
- `Tab`
- `Space`
- `Enter` or `Return`
- `Escape` or `Esc`
- `Backspace`
- `Delete` or `Del`
- `Insert` or `Ins`
- `Home`
- `End`
- `PageUp` or `PgUp`
- `PageDown` or `PgDn`

**Arrow Keys**: `Up`, `Down`, `Left`, `Right`

**Symbols**:
- `[` or `BracketLeft`
- `]` or `BracketRight`
- `+` or `Plus`
- `-` or `Minus`
- `=` or `Equal`
- `/` or `Slash`
- `\` or `Backslash`
- `,` or `Comma`
- `.` or `Period`
- `;` or `Semicolon`
- `'` or `Apostrophe`
- `` ` `` or `Grave`

## Available Shortcuts

### Tab Management

```toml
[keybindings.shortcuts]
# Create new terminal tab (default: Ctrl+T)
new_tab = "Ctrl+T"

# Close current tab (default: Ctrl+W)
close_tab = "Ctrl+W"

# Switch to next tab (default: Ctrl+Tab)
next_tab = "Ctrl+Tab"

# Switch to previous tab (default: Ctrl+Shift+Tab)
prev_tab = "Ctrl+Shift+Tab"

# Create new document tab (default: Ctrl+O)
new_document_tab = "Ctrl+O"
```

### Tab Switching

```toml
# Switch to specific tabs 1-9 (default: Ctrl+1 through Ctrl+9)
switch_to_tab_1 = "Ctrl+1"
switch_to_tab_2 = "Ctrl+2"
switch_to_tab_3 = "Ctrl+3"
switch_to_tab_4 = "Ctrl+4"
switch_to_tab_5 = "Ctrl+5"
switch_to_tab_6 = "Ctrl+6"
switch_to_tab_7 = "Ctrl+7"
switch_to_tab_8 = "Ctrl+8"
switch_to_tab_9 = "Ctrl+9"
```

### Pane Management

```toml
# Split pane horizontally - side by side (default: Ctrl+Shift+H)
split_horizontal = "Ctrl+Shift+H"

# Split pane vertically - top/bottom (default: Ctrl+Shift+D)
split_vertical = "Ctrl+Shift+D"

# Close focused pane (default: Ctrl+Shift+W)
close_pane = "Ctrl+Shift+W"

# Focus next pane (default: Ctrl+Shift+])
focus_next_pane = "Ctrl+Shift+]"

# Focus previous pane (default: Ctrl+Shift+[)
focus_prev_pane = "Ctrl+Shift+["
```

### UI Features

```toml
# Toggle AI panel (default: Ctrl+Shift+A)
toggle_ai = "Ctrl+Shift+A"

# Quick tab switcher (default: Ctrl+K)
quick_switcher = "Ctrl+K"

# SSH connection manager (default: Ctrl+S)
ssh_manager = "Ctrl+S"

# Open file dialog (default: Ctrl+Shift+O)
open_file = "Ctrl+Shift+O"
```

### Application

```toml
# Quit application (default: Ctrl+Q)
quit = "Ctrl+Q"
```

## Disabling Shortcuts

To disable a shortcut, set it to an empty string or remove it from the config:

```toml
[keybindings.shortcuts]
# Disable the quick switcher
quick_switcher = ""
```

## Example Configuration

Here's a complete example with Emacs-style bindings:

```toml
[keybindings.shortcuts]
# Tab management
new_tab = "Ctrl+X+T"
close_tab = "Ctrl+X+K"
next_tab = "Ctrl+X+Right"
prev_tab = "Ctrl+X+Left"

# Pane management
split_horizontal = "Ctrl+X+3"
split_vertical = "Ctrl+X+2"
close_pane = "Ctrl+X+0"
focus_next_pane = "Ctrl+X+O"

# UI features
toggle_ai = "Alt+A"
quick_switcher = "Alt+X"
quit = "Ctrl+X+Ctrl+C"
```

## Example Configuration (macOS-style)

```toml
[keybindings.shortcuts]
# Use Cmd instead of Ctrl on macOS
new_tab = "Super+T"
close_tab = "Super+W"
quit = "Super+Q"
quick_switcher = "Super+K"
toggle_ai = "Super+Shift+A"
```

## Troubleshooting

### Shortcut Not Working

1. Check the configuration file for syntax errors:
   ```bash
   cat ~/.config/corgiterm/config.toml
   ```

2. Check the logs for parsing errors:
   ```bash
   journalctl -f | grep corgiterm
   ```

3. Ensure the shortcut doesn't conflict with system or GTK shortcuts

### Reverting to Defaults

To revert all shortcuts to defaults, remove the `[keybindings.shortcuts]` section from your config file or delete the entire section.

## Hot Reloading

Changes to keyboard shortcuts require restarting CorgiTerm. Configuration hot-reloading for shortcuts is planned for a future release.

## Technical Details

- Shortcuts are parsed at application startup
- Invalid shortcuts are logged as warnings and ignored
- If parsing fails, the default shortcut is used
- Shortcuts use GTK4's event controller system
- Case-insensitive key matching for letters
