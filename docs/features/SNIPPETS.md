# Command Snippets Library

Save and reuse common commands with CorgiTerm's built-in snippets library.

## Overview

The Snippets feature allows you to:
- **Save** frequently used commands for quick access
- **Search** and filter snippets by name, description, or tags
- **Insert** commands into the terminal with a single click
- **Track usage** to see your most-used commands
- **Organize** with tags and categories
- **Quick access** via keyboard shortcuts (Ctrl+Shift+P)

## Features

### Data Model

Each snippet contains:
- **Name**: Short, descriptive title
- **Command**: The actual command to execute
- **Description**: What the command does
- **Tags**: Comma-separated tags for organization
- **Usage tracking**: Automatically tracks how often you use each snippet
- **Timestamps**: Creation time and last used time

### Storage

Snippets are stored in JSON format at:
```
~/.config/corgiterm/snippets.json
```

The file is automatically created on first use and persists between sessions.

### User Interface

#### Snippets Library (Ctrl+Shift+S)

Full-featured library management:
- **Search bar**: Filter snippets by name, description, command, or tags
- **Sort options**:
  - By name (alphabetical)
  - By usage (most used first)
  - By recency (recently used first)
- **Action buttons** for each snippet:
  - **Insert**: Copy command to active terminal
  - **Edit**: Modify snippet details
  - **Delete**: Remove snippet (with confirmation)
- **Add button**: Create new snippets

#### Quick Insert Dialog (Ctrl+Shift+P)

Fast command palette-style access:
- Type to fuzzy search snippets
- Shows top 10 most relevant results
- Displays usage count for each snippet
- Press Enter or click to insert
- Escape to cancel

### Creating Snippets

#### From the Library
1. Click the **+** button in the toolbar
2. Fill in:
   - Name (required)
   - Command (required)
   - Description (optional but recommended)
   - Tags (comma-separated, optional)
3. Click **Add**

#### Programmatically
```rust
use corgiterm_config::{Snippet, SnippetsManager};

let snippet = Snippet::new(
    "Git Status".to_string(),
    "git status -sb".to_string(),
    "Show git status in short format".to_string(),
    vec!["git".to_string(), "status".to_string()],
);

if let Some(manager) = app::snippets_manager() {
    manager.read().add(snippet)?;
}
```

## Usage Tracking

The library automatically tracks:
- **Use count**: Incremented each time you insert a snippet
- **Last used**: Timestamp of most recent use

This data powers:
- Sort by usage (find your most-used commands)
- Sort by recency (find recently accessed snippets)
- Smart suggestions in Quick Insert

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+S` | Open Snippets Library |
| `Ctrl+Shift+P` | Quick Insert (command palette) |
| `Escape` | Close dialogs |
| `Enter` | Insert selected snippet (in Quick Insert) |

## Example Snippets

Here are some useful snippets to get started:

### Git Commands
```json
{
  "name": "Git Status",
  "command": "git status -sb",
  "description": "Show git status in short format with branch info",
  "tags": ["git", "status"]
}
```

```json
{
  "name": "Git Log Pretty",
  "command": "git log --oneline --graph --decorate --all",
  "description": "Beautiful git log with graph",
  "tags": ["git", "log", "visualization"]
}
```

### System Administration
```json
{
  "name": "Find Large Files",
  "command": "find . -type f -size +100M -exec ls -lh {} \\;",
  "description": "Find files larger than 100MB",
  "tags": ["find", "disk", "cleanup"]
}
```

```json
{
  "name": "Memory Usage Top 10",
  "command": "ps aux --sort=-%mem | head -n 11",
  "description": "Show top 10 memory-consuming processes",
  "tags": ["memory", "performance", "monitoring"]
}
```

### Docker
```json
{
  "name": "Docker Clean",
  "command": "docker system prune -af --volumes",
  "description": "Clean up all unused Docker resources",
  "tags": ["docker", "cleanup"]
}
```

### Development
```json
{
  "name": "Cargo Watch Test",
  "command": "cargo watch -x test",
  "description": "Continuously run tests on file changes",
  "tags": ["rust", "cargo", "testing"]
}
```

```json
{
  "name": "npm Update All",
  "command": "npm update && npm outdated",
  "description": "Update all npm packages and show outdated ones",
  "tags": ["npm", "update", "nodejs"]
}
```

## Demo Data

To try out the snippets feature, copy the example file:

```bash
cp examples/snippets-demo.json ~/.config/corgiterm/snippets.json
```

This includes 10 commonly used commands across git, Docker, npm, and system administration.

## Technical Details

### Architecture

**Config Layer** (`corgiterm-config`):
- `Snippet` struct: Data model with usage tracking
- `SnippetsConfig`: Collection management
- `SnippetsManager`: File I/O and CRUD operations

**UI Layer** (`corgiterm-ui/snippets.rs`):
- `show_snippets_dialog()`: Full library UI
- `show_quick_insert_dialog()`: Fast command palette
- `show_snippet_editor_dialog()`: Create/edit UI
- Global initialization via `init_snippets()`

### Data Flow

1. **Initialization**: `SnippetsManager` loads from `~/.config/corgiterm/snippets.json`
2. **User Action**: Click, search, or keyboard shortcut triggers UI
3. **Display**: UI fetches snippets via manager, applies filters/sorting
4. **Insert**: Click → record usage → return command string → caller inserts into terminal
5. **Save**: All changes (add/edit/delete/usage) automatically persist to JSON

### File Format

```json
{
  "snippets": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Example",
      "command": "echo 'Hello'",
      "description": "Print hello",
      "tags": ["example", "demo"],
      "created_at": 1732895400,
      "last_used": 1732900000,
      "use_count": 5
    }
  ]
}
```

### UUID Generation

Each snippet gets a unique UUID v4 identifier for reliable tracking across edits and ensuring no conflicts.

## Future Enhancements

Potential improvements for future versions:

- **Variables/Placeholders**: Support `{{placeholder}}` syntax for parameterized commands
- **Import/Export**: Share snippet collections with others
- **Sync**: Cloud sync across devices
- **Categories**: Hierarchical organization beyond tags
- **Templates**: Pre-built snippet packs for common workflows
- **Snippet Runner**: Execute snippets directly without inserting
- **Smart Suggestions**: AI-powered snippet recommendations based on context

## Integration Points

The snippets feature integrates with:
- **Terminal View**: Insert commands into active terminal
- **AI Panel**: Suggest snippets based on natural language queries
- **Session Management**: Save project-specific snippets
- **Safe Mode**: Preview snippet commands before execution

## Troubleshooting

### Snippets not saving
- Check file permissions on `~/.config/corgiterm/`
- Look for errors in logs: `~/.config/corgiterm/corgiterm.log`

### Snippets file corrupted
1. Backup current file: `cp ~/.config/corgiterm/snippets.json ~/.config/corgiterm/snippets.json.bak`
2. Start fresh: `rm ~/.config/corgiterm/snippets.json`
3. Restart CorgiTerm to create new file

### Can't find a snippet
- Check search query (searches name, description, command, and tags)
- Try sorting by usage or recency
- Verify snippet exists in `~/.config/corgiterm/snippets.json`

## Development

### Adding snippets programmatically

```rust
use corgiterm_config::Snippet;

// Create snippet
let snippet = Snippet::new(
    "My Command".into(),
    "echo 'test'".into(),
    "Test command".into(),
    vec!["test".into()],
);

// Add to library
if let Some(mgr) = crate::app::snippets_manager() {
    mgr.read().add(snippet).unwrap();
}
```

### Accessing snippets

```rust
// Search
if let Some(mgr) = crate::app::snippets_manager() {
    let results = mgr.read().search("git");
    for snippet in results {
        println!("{}: {}", snippet.name, snippet.command);
    }
}

// Get by usage
if let Some(mgr) = crate::app::snippets_manager() {
    let popular = mgr.read().by_usage();
    // Returns Vec<Snippet> sorted by use_count
}
```

## Credits

Inspired by:
- VS Code Command Palette
- IntelliJ Live Templates
- Warp Terminal Workflows
- Fish shell abbreviations

Built with GTK4 and libadwaita for native Linux desktop integration.
