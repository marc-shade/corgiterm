# Split Pane Tree Structure and Operations

## Tree Structure Example

### Initial State (Single Pane)
```
Root
└── Terminal1
```

### After Horizontal Split
```
Root (Paned-H)
├── Terminal1
└── Terminal2 (focused)
```

### After Vertical Split on Terminal2
```
Root (Paned-H)
├── Terminal1
└── Split (Paned-V)
    ├── Terminal2
    └── Terminal3 (focused)
```

### After Another Horizontal Split on Terminal1
```
Root (Paned-H)
├── Split (Paned-H)
│   ├── Terminal1
│   └── Terminal4 (focused)
└── Split (Paned-V)
    ├── Terminal2
    └── Terminal3
```

## Close Operation Example

### Before Closing Terminal3
```
Root (Paned-H)
├── Terminal1
└── Split (Paned-V)      <- Parent
    ├── Terminal2        <- Sibling
    └── Terminal3        <- Target (focused)
```

### Step 1: Identify Parent and Sibling
- Parent: `Split (Paned-V)` node
- Sibling: `Terminal2`
- Target: `Terminal3` (the one being closed)

### Step 2: Promote Sibling
Replace parent's content with sibling's content:
```
Root (Paned-H)
├── Terminal1
└── Terminal2        <- Promoted (was child, now direct child of Root)
```

### Step 3: Update Widget Tree
GTK hierarchy is updated to match:
- Old `Paned-V` widget is removed from Root's Paned-H
- Terminal2's widget is added to Root's Paned-H

### Step 4: Update Focus
Focus is updated to Terminal2 (or first available terminal)

## Focus Cycling

With 4 panes in this structure:
```
Root (Paned-H)
├── Split (Paned-V)
│   ├── Terminal1      <- Index 0
│   └── Terminal2      <- Index 1
└── Split (Paned-V)
    ├── Terminal3      <- Index 2
    └── Terminal4      <- Index 3
```

### Focus Order (Depth-First Traversal)
1. Terminal1 (0)
2. Terminal2 (1)
3. Terminal3 (2)
4. Terminal4 (3)
5. Wraps to Terminal1 (0)

### Backward Cycling
4 → 3 → 2 → 1 → 4 (wraps)

## Algorithm Complexity

### Close Operation
- **Time**: O(n) where n is the number of panes (worst case: tree traversal)
- **Space**: O(h) where h is tree height (recursion stack)

### Focus Cycling
- **Time**: O(1) - Direct array indexing after initial collection
- **Space**: O(n) - Cached array of all pane references

### Pane List Refresh
- **Time**: O(n) - Single tree traversal
- **Space**: O(n) - Array of all terminal nodes

## Tree Invariants

1. **Root is never closed**: Close operations only work on non-root nodes
2. **Always at least one pane**: Can't close if only root terminal exists
3. **Binary tree**: Each split node has exactly 2 children
4. **Terminal leaves**: All leaf nodes are terminals, never splits
5. **Valid focus**: `focused_pane` always points to a valid terminal node

## Edge Cases Handled

1. **Single pane**: `close_focused()` returns `false`, no operation
2. **Closing root**: Detected by `Rc::ptr_eq`, operation aborted
3. **Empty tree**: Impossible - root is always present
4. **Focus on split node**: `find_first_terminal()` ensures terminal focus
5. **Wrap-around**: Modulo arithmetic handles boundary conditions

## Widget Lifecycle

### On Split
1. Create new `Paned` widget
2. Create new `TerminalView` widget
3. Move existing terminal to first child
4. Add new terminal to second child
5. Replace node's widget in parent

### On Close
1. Identify widgets to remove/keep
2. Remove closed terminal widget from GTK tree
3. Remove parent paned widget from GTK tree
4. Add sibling's widget to grandparent
5. GTK automatically handles cleanup of removed widgets

## Memory Safety

- **No leaks**: Widgets removed from GTK tree are automatically dropped
- **No dangling pointers**: `Rc<RefCell<>>` ensures reference counting
- **No double-free**: Rust ownership system prevents
- **Thread safety**: Not needed - GTK is single-threaded

## Performance Considerations

### Optimization: Pane Cache
Instead of traversing tree on every focus change:
- Cache all terminal nodes in `all_panes` vector
- Refresh only when tree structure changes (split/close)
- O(1) focus navigation vs O(n) tree traversal

### Memory Overhead
- Each pane: ~16 bytes (Rc pointer + RefCell)
- Cache vector: 16 bytes per pane
- Total overhead: ~32 bytes per pane (negligible)

### GTK Widget Updates
- Minimized: Only update affected subtree
- Batch operations: Widget changes before GTK refresh
- No redundant redraws: GTK handles efficiently
