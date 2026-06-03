# Keyboard Shortcuts

CorgiTerm keyboard shortcuts can be edited from **Preferences > Keybindings**. The shortcut help window uses the same configuration, so the shortcuts shown in the UI match the shortcuts the app will handle.

Advanced users can also edit `~/.config/corgiterm/config.toml` under `[keybindings.shortcuts]`.

## Shortcut Format

Shortcuts use modifiers plus one key, separated by `+`:

```toml
[keybindings.shortcuts]
new_tab = "Ctrl+T"
toggle_ai = "Ctrl+Shift+A"
quick_switcher = "Ctrl+K"
```

Multi-step key sequences such as `Ctrl+X+T` are not supported. Use one final key after any modifiers.

### Supported Modifiers

- `Ctrl` or `Control`
- `Shift`
- `Alt` or `Meta`
- `Super`, `Win`, or `Cmd`

### Supported Keys

- Letters: `A-Z`, case-insensitive
- Numbers: `0-9`
- Function keys: `F1-F12`
- Special keys: `Tab`, `Space`, `Enter`, `Return`, `Escape`, `Esc`, `Backspace`, `Delete`, `Del`, `Insert`, `Ins`, `Home`, `End`, `PageUp`, `PgUp`, `PageDown`, `PgDn`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Symbols: `[`, `]`, `BracketLeft`, `BracketRight`, `+`, `Plus`, `-`, `Minus`, `=`, `Equal`, `/`, `Slash`, `\`, `Backslash`, `,`, `Comma`, `.`, `Period`, `;`, `Semicolon`, `'`, `Apostrophe`, `` ` ``, `Grave`

## Defaults

### Tabs

| Action | Default | Config key |
|---|---|---|
| New terminal tab | `Ctrl+T` | `new_tab` |
| Close current tab | `Ctrl+W` | `close_tab` |
| Next tab | `Ctrl+Tab` | `next_tab` |
| Previous tab | `Ctrl+Shift+Tab` | `prev_tab` |
| New document tab | `Ctrl+O` | `new_document_tab` |
| Switch to tab 1 | `Ctrl+1` | `switch_to_tab_1` |
| Switch to tab 2 | `Ctrl+2` | `switch_to_tab_2` |
| Switch to tab 3 | `Ctrl+3` | `switch_to_tab_3` |
| Switch to tab 4 | `Ctrl+4` | `switch_to_tab_4` |
| Switch to tab 5 | `Ctrl+5` | `switch_to_tab_5` |
| Switch to tab 6 | `Ctrl+6` | `switch_to_tab_6` |
| Switch to tab 7 | `Ctrl+7` | `switch_to_tab_7` |
| Switch to tab 8 | `Ctrl+8` | `switch_to_tab_8` |
| Switch to tab 9 | `Ctrl+9` | `switch_to_tab_9` |

### Panes

| Action | Default | Config key |
|---|---|---|
| Split horizontal | `Ctrl+Shift+H` | `split_horizontal` |
| Split vertical | `Ctrl+Shift+D` | `split_vertical` |
| Close focused pane | `Ctrl+Shift+W` | `close_pane` |
| Focus next pane | `Ctrl+Shift+]` | `focus_next_pane` |
| Focus previous pane | `Ctrl+Shift+[` | `focus_prev_pane` |

### Terminal

| Action | Default | Config key |
|---|---|---|
| Copy | `Ctrl+Shift+C` | `copy` |
| Paste | `Ctrl+Shift+V` | `paste` |
| Select all | `Ctrl+Shift+A` | `select_all` |
| Find in terminal | `Ctrl+Shift+F` | `find_terminal` |
| Activate hints | `Ctrl+Shift+U` | `activate_hints` |
| Zoom in | `Ctrl+Plus` | `zoom_in` |
| Zoom out | `Ctrl+Minus` | `zoom_out` |
| Reset zoom | `Ctrl+0` | `reset_zoom` |

### Tools And UI

| Action | Default | Config key |
|---|---|---|
| Toggle AI panel | `Ctrl+Shift+A` | `toggle_ai` |
| Toggle sidebar | `Ctrl+Shift+B` | `toggle_sidebar` |
| Quick switcher | `Ctrl+K` | `quick_switcher` |
| SSH manager | `Ctrl+S` | `ssh_manager` |
| Snippets | `Ctrl+Shift+S` | `snippets` |
| ASCII art | `Ctrl+Shift+G` | `ascii_art` |
| Open file | `Ctrl+Shift+O` | `open_file` |
| History search | `Ctrl+R` | `history_search` |

### Application

| Action | Default | Config key |
|---|---|---|
| Quit | `Ctrl+Q` | `quit` |

## Disabling Shortcuts

In Preferences, clear the shortcut field and apply it. In the config file, set the shortcut to an empty string:

```toml
[keybindings.shortcuts]
quick_switcher = ""
```

## Resetting Shortcuts

Use **Preferences > Keybindings > Reset All Shortcuts** to restore defaults. You can also remove the `[keybindings.shortcuts]` section from `config.toml`.

## Live Behavior

- Changes made in Preferences apply immediately and are saved to disk.
- The keyboard shortcuts help window reads the current configuration each time it opens.
- Manual edits to `config.toml` are loaded on the next app launch.

## Troubleshooting

1. Use the Preferences page first; invalid shortcut labels are marked before saving.
2. If editing `config.toml`, confirm the shortcut is under `[keybindings.shortcuts]`.
3. Check whether a macOS, GTK, or terminal shell shortcut is intercepting the same key.
4. Check logs for shortcut parsing warnings if a manually edited shortcut does not work.
