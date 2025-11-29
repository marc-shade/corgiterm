# CorgiTerm Keyboard Shortcuts - Quick Reference

## Tab Management
| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| New terminal tab | `Ctrl+T` | `new_tab` |
| New document tab | `Ctrl+O` | `new_document_tab` |
| Close current tab | `Ctrl+W` | `close_tab` |
| Next tab | `Ctrl+Tab` | `next_tab` |
| Previous tab | `Ctrl+Shift+Tab` | `prev_tab` |
| Switch to tab 1-9 | `Ctrl+1` to `Ctrl+9` | `switch_to_tab_1` to `switch_to_tab_9` |

## Pane Management
| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Split horizontal (side by side) | `Ctrl+Shift+H` | `split_horizontal` |
| Split vertical (top/bottom) | `Ctrl+Shift+D` | `split_vertical` |
| Close focused pane | `Ctrl+Shift+W` | `close_pane` |
| Focus next pane | `Ctrl+Shift+]` | `focus_next_pane` |
| Focus previous pane | `Ctrl+Shift+[` | `focus_prev_pane` |

## UI Features
| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Toggle AI panel | `Ctrl+Shift+A` | `toggle_ai` |
| Quick tab switcher | `Ctrl+K` | `quick_switcher` |
| SSH manager | `Ctrl+S` | `ssh_manager` |
| Open file dialog | `Ctrl+Shift+O` | `open_file` |

## Application
| Action | Default Shortcut | Config Key |
|--------|-----------------|------------|
| Quit | `Ctrl+Q` | `quit` |

---

## Customization

Add to `~/.config/corgiterm/config.toml`:

```toml
[keybindings.shortcuts]
new_tab = "Ctrl+T"
toggle_ai = "Ctrl+Shift+A"
# ... customize as needed
```

See `docs/keyboard-shortcuts.md` for full documentation.
