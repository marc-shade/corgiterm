# Keyboard Shortcuts Implementation Summary

## Overview

Implemented configurable keyboard shortcuts for CorgiTerm, allowing users to customize all keyboard bindings via the configuration file.

## Changes Made

### 1. Configuration Schema (`corgiterm-config/src/lib.rs`)

Added `ShortcutsConfig` struct with all configurable shortcuts:
- Tab management (new, close, next, prev)
- Tab switching (1-9)
- Pane management (split, close, focus)
- UI features (AI toggle, quick switcher, SSH manager, file open)
- Application (quit)

All shortcuts are `Option<String>` to allow disabling and provide defaults.

### 2. Shortcut Parser (`corgiterm-config/src/shortcuts.rs`)

New module providing:
- `parse_shortcut()`: Parses strings like "Ctrl+Shift+A" into GTK modifiers and keys
- `ParsedShortcut`: Structured representation of a shortcut
- `matches_shortcut()`: Checks if a key event matches a parsed shortcut

**Supported modifiers:**
- Ctrl/Control
- Shift
- Alt/Meta
- Super/Win/Cmd

**Supported keys:**
- Letters (A-Z, case-insensitive)
- Numbers (0-9)
- Function keys (F1-F12)
- Special keys (Tab, Enter, Escape, etc.)
- Arrow keys
- Common symbols

### 3. Keyboard Shortcuts Manager (`corgiterm-ui/src/keyboard.rs`)

New module providing:
- `ShortcutAction`: Enum of all available actions
- `KeyboardShortcuts`: Manager that loads from config and provides matching

**Features:**
- Loads shortcuts from configuration at startup
- Validates and parses all shortcuts
- Logs warnings for invalid shortcuts
- Falls back to defaults for missing/invalid shortcuts
- Provides clean API for checking key matches

### 4. Window Integration (`corgiterm-ui/src/window.rs`)

Refactored keyboard handling:
- Loads `KeyboardShortcuts` from configuration at startup
- Replaced hardcoded key matching with `shortcuts.matches()` calls
- Cleaner, more maintainable code structure
- All actions now configurable

**Before:**
```rust
if ctrl && !shift {
    match key {
        Key::t | Key::T => { /* new tab */ }
        // ... many more cases
    }
}
```

**After:**
```rust
if shortcuts.matches(ShortcutAction::NewTab, key, modifier) {
    tabs.add_terminal_tab("Terminal", None);
    return gtk4::glib::Propagation::Stop;
}
```

### 5. Documentation

Created comprehensive documentation:
- `docs/keyboard-shortcuts.md`: User guide with examples
- `docs/examples/keyboard-shortcuts.toml`: Example configuration with all shortcuts

## Configuration Example

```toml
[keybindings.shortcuts]
# Tab management
new_tab = "Ctrl+T"
close_tab = "Ctrl+W"
next_tab = "Ctrl+Tab"
prev_tab = "Ctrl+Shift+Tab"

# Pane management
split_horizontal = "Ctrl+Shift+H"
split_vertical = "Ctrl+Shift+D"

# UI features
toggle_ai = "Ctrl+Shift+A"
quick_switcher = "Ctrl+K"
ssh_manager = "Ctrl+S"

# Application
quit = "Ctrl+Q"

# Disable a shortcut
# quick_switcher = ""
```

## Default Shortcuts

All default shortcuts match the original hardcoded values:
- **Ctrl+T**: New tab
- **Ctrl+W**: Close tab
- **Ctrl+Shift+A**: Toggle AI panel
- **Ctrl+K**: Quick switcher
- **Ctrl+S**: SSH manager
- **Ctrl+Shift+H/D**: Split panes
- **Ctrl+Shift+W**: Close pane
- **Ctrl+Shift+]**: Focus next pane
- **Ctrl+Shift+[**: Focus previous pane
- **Ctrl+1-9**: Switch to tab 1-9
- **Ctrl+O**: New document tab
- **Ctrl+Shift+O**: Open file
- **Ctrl+Q**: Quit

## Technical Details

### Architecture

```
Config File (TOML)
    ↓
ShortcutsConfig (parsed by figment/toml)
    ↓
KeyboardShortcuts::from_config()
    ↓
ParsedShortcut (GTK ModifierType + Key)
    ↓
matches() checks at runtime
```

### Key Features

1. **Type Safety**: Shortcuts are validated at startup, not at runtime
2. **Graceful Degradation**: Invalid shortcuts log warnings and use defaults
3. **Case Insensitive**: Letter keys match both uppercase and lowercase
4. **Special Handling**: Tab key correctly handles ISO_Left_Tab for Shift+Tab
5. **Extensible**: Easy to add new shortcuts by extending `ShortcutAction` enum

### Error Handling

- Invalid modifier → Error logged, shortcut ignored, default used
- Invalid key → Error logged, shortcut ignored, default used
- Empty shortcut string → Shortcut disabled
- Missing shortcut in config → Default used
- Malformed TOML → Entire config parsing fails (handled by figment)

## Testing

Manual testing checklist:
- [ ] All default shortcuts work
- [ ] Custom shortcuts in config work
- [ ] Disabled shortcuts don't trigger
- [ ] Invalid shortcuts log warnings
- [ ] Case-insensitive matching works
- [ ] Tab switching (Ctrl+1-9) works
- [ ] Pane operations work
- [ ] UI toggles work

## Future Enhancements

Potential improvements:
1. **Hot Reload**: Reload shortcuts without restart (config manager supports this)
2. **Conflict Detection**: Warn about duplicate shortcuts
3. **UI Editor**: Visual shortcut editor in preferences
4. **Context-Aware**: Different shortcuts per mode (terminal vs document)
5. **Sequences**: Support multi-key sequences (like Emacs Ctrl+X Ctrl+S)
6. **Mouse Bindings**: Extend to mouse button combinations

## Migration Notes

No migration needed! The implementation:
- Preserves all existing default shortcuts
- Only affects users who add `[keybindings.shortcuts]` to their config
- Backwards compatible with existing configurations

## Files Modified

- `crates/corgiterm-config/src/lib.rs` - Added ShortcutsConfig
- `crates/corgiterm-config/src/shortcuts.rs` - New parser module
- `crates/corgiterm-config/Cargo.toml` - Added gtk4 dependency
- `crates/corgiterm-ui/src/keyboard.rs` - New shortcuts manager
- `crates/corgiterm-ui/src/lib.rs` - Exposed keyboard module
- `crates/corgiterm-ui/src/window.rs` - Integrated configurable shortcuts
- `docs/keyboard-shortcuts.md` - User documentation
- `docs/examples/keyboard-shortcuts.toml` - Example configuration

## Build Status

✅ Compiles successfully with no errors
⚠️ Some unrelated warnings in other modules (dead code, unused variables)

```bash
cargo build --workspace
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.18s
```
