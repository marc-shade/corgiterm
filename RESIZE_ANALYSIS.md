# GTK4 Terminal Resize Analysis & Solution

## Problem Summary

Your terminal emulator has two resize-related issues:

1. **Blank lines when opening/closing AI panel** - Grid resizes immediately, but PTY hasn't been notified yet
2. **Ghost/repeating lines during GTK Paned resize** - Grid constantly changing size, creating visual artifacts

## Root Cause Analysis

### Current Implementation Issues

```rust
// ❌ PROBLEM: Grid resizes immediately
term_for_resize.borrow_mut().resize(new_terminal_size);
drawing_area_for_resize.queue_draw();

// ✅ PTY resize is correctly debounced
glib::timeout_add_local_once(
    std::time::Duration::from_millis(100),
    move || {
        pty.resize(new_pty_size);
    },
);
```

**Why this fails:**

1. **Grid-PTY desync**: Grid shrinks/grows before PTY knows
   - Shrink: New size truncates content → blank areas appear
   - Grow: New cells are uninitialized → garbage or blank content

2. **No frame synchronization**: `queue_draw()` and `timeout_add_local()` aren't tied to the rendering pipeline
   - Resize can happen mid-frame
   - Multiple resize events during rapid drag create tearing

3. **Visual feedback paradox**:
   - You want instant visual feedback (good UX)
   - But instant grid resize causes corruption
   - Solution: Use frame clock for smooth, artifact-free resizing

## GTK4 Frame Clock Explanation

### What is GdkFrameClock?

GTK4's rendering happens in phases per frame (~60 FPS):

```
Frame N:
  1. FLUSH_EVENTS - Process input events
  2. BEFORE_PAINT - Prepare for painting
  3. UPDATE        - Layout/resize/animations ← RESIZE SHOULD HAPPEN HERE
  4. LAYOUT        - Widget layout
  5. PAINT         - Draw to screen
  6. AFTER_PAINT   - Cleanup
  └→ Frame N+1
```

### Why Use Frame Clock?

**Without frame clock (your current code):**
```
Resize event → Immediate grid resize → queue_draw()
                                      ↓
                            Next paint cycle shows partial state
                            ↓
                            100ms later PTY resizes
                            ↓
                            queue_draw() again

Result: User sees intermediate corrupted states
```

**With frame clock:**
```
Resize event → Store pending resize → Wait for debounce
                                      ↓
                            Frame UPDATE phase → Apply resize atomically
                            ↓
                            Same frame PAINT phase → Draw correct state

Result: User sees only final correct state
```

## Solution Patterns

### Pattern 1: Timeout with Frame Awareness (Recommended)

**Pros:**
- Simple to implement
- Works with existing code structure
- Good enough for most cases

**Cons:**
- Still uses timeout (not perfectly frame-synced)
- Small chance of tearing under extreme resize rates

```rust
// Store pending resize
*pending_resize.borrow_mut() = Some(PendingResize {
    rows: new_rows,
    cols: new_cols,
    px_width: width,
    px_height: height,
    timestamp: std::time::Instant::now(),
});

// Debounce with timeout
let callback_id = glib::timeout_add_local_once(
    std::time::Duration::from_millis(100),
    move || {
        // Apply BOTH grid and PTY resize together
        term.resize(new_size);
        pty.resize(new_size);
        queue_draw();
    },
);
```

**Key change:** Grid and PTY resize together, not separately.

### Pattern 2: Pure Frame Clock (Cleanest)

**Pros:**
- Perfect frame synchronization
- No tearing ever
- Matches GTK4 best practices

**Cons:**
- More complex to implement
- Requires careful handler lifecycle management

```rust
// Get frame clock from drawing area
let frame_clock = drawing_area.frame_clock();

// Store pending resize
*pending_resize.borrow_mut() = Some(pending);

// Connect to UPDATE phase
let handler_id = frame_clock.connect_update(move |clock| {
    if let Some(pending) = pending_resize.borrow_mut().take() {
        // Check debounce
        if pending.timestamp.elapsed() >= Duration::from_millis(100) {
            // Apply resize during UPDATE phase
            term.resize(pending.size);
            pty.resize(pending.size);

            // Disconnect handler
            clock.disconnect(handler_id);
        }
    }
});

// Tell frame clock to call update handlers
frame_clock.begin_updating();
```

### Pattern 3: Hybrid (Best of Both)

Combine timeout debouncing with frame-synchronized application:

```rust
// Use timeout for debounce window
glib::timeout_add_local_once(Duration::from_millis(100), move || {
    // Use frame clock to apply resize on next UPDATE phase
    if let Some(frame_clock) = drawing_area.frame_clock() {
        frame_clock.connect_update_once(move |_| {
            // Resize happens here during UPDATE phase
            term.resize(size);
            pty.resize(size);
        });
        frame_clock.request_phase(gdk::FrameClockPhase::UPDATE);
    }
});
```

## Additional Recommendations

### 1. Pre-fill New Cells

When grid grows, fill new cells with spaces to avoid garbage:

```rust
impl Terminal {
    pub fn resize(&mut self, new_size: TerminalSize) {
        let old_size = self.size;

        // If growing, pre-fill with spaces
        if new_size.rows > old_size.rows {
            for _ in old_size.rows..new_size.rows {
                self.grid.push(vec![Cell::default(); new_size.cols]);
            }
        }

        if new_size.cols > old_size.cols {
            for row in &mut self.grid {
                row.resize(new_size.cols, Cell::default());
            }
        }

        // Then handle shrinking...
        self.size = new_size;
    }
}
```

### 2. Optimize Redraw During Resize

Only redraw on final resize, not intermediate states:

```rust
// Don't call queue_draw() in connect_resize
// Only call it after debounce completes

drawing_area.connect_resize(move |_, width, height| {
    // Store pending, don't draw
    *pending_resize.borrow_mut() = Some(pending);

    // Debounce...
});

// In timeout callback:
glib::timeout_add_local_once(Duration::from_millis(100), move || {
    apply_resize();
    drawing_area.queue_draw(); // ✅ Only draw final state
});
```

### 3. Handle Edge Cases

```rust
// Minimum size enforcement
let new_cols = new_cols.max(2);
let new_rows = new_rows.max(2);

// Detect no-op resizes
if current_size.rows == new_rows && current_size.cols == new_cols {
    return; // Don't process redundant resize
}

// Handle cell dimension not ready
if cell_w <= 0.0 || cell_h <= 0.0 {
    tracing::warn!("Cell dimensions not calculated yet, deferring resize");
    return;
}
```

## Implementation Steps

1. **Replace current resize handler** (lines 575-665) with solution from `resize_solution.rs`

2. **Test both scenarios:**
   - Open/close AI panel → Should see smooth transition, no blanks
   - Drag GTK Paned splitter → Should see smooth resize, no ghosts

3. **Monitor for issues:**
   - Check logs for "Applying resize" messages
   - Verify debounce timing (should batch rapid resizes)
   - Watch for shell prompt spam (PTY resize should still be debounced)

4. **Optional optimizations:**
   - Implement pattern 3 (hybrid) for perfect frame sync
   - Add cell pre-filling to Terminal::resize()
   - Remove intermediate queue_draw() calls

## Debugging Tips

### Enable Resize Logging

```rust
drawing_area.connect_resize(move |_, width, height| {
    tracing::debug!(
        "Resize event: {}x{} → grid {}x{}, cell {}x{}",
        width, height, new_rows, new_cols, cell_w, cell_h
    );
    // ...
});

// In timeout:
tracing::debug!(
    "Applying resize after {}ms debounce",
    pending.timestamp.elapsed().as_millis()
);
```

### Check Frame Clock Availability

```rust
if let Some(frame_clock) = drawing_area.frame_clock() {
    tracing::debug!("Frame clock available, FPS: {}", frame_clock.fps());
} else {
    tracing::warn!("No frame clock - widget not realized?");
}
```

### Measure Resize Performance

```rust
let start = std::time::Instant::now();
term.resize(new_size);
pty.resize(new_pty_size);
tracing::debug!("Resize took {:?}", start.elapsed());
```

## Expected Results

### Before (Current Behavior)

- **AI panel open**: Terminal shows blank area at bottom
- **AI panel close**: Terminal content jumps/repeats
- **Paned drag**: Ghosting/flickering during resize
- **Logs**: Hundreds of "Resizing PTY" messages (timeout spam)

### After (With Fix)

- **AI panel open**: Smooth transition, content fills immediately
- **AI panel close**: Smooth transition, no jumps
- **Paned drag**: Butter-smooth resize, no artifacts
- **Logs**: Single "Applying resize" after drag completes

## Alternative: Don't Resize Grid Immediately

If frame clock is too complex, simplest fix:

```rust
drawing_area.connect_resize(move |_, width, height| {
    // Calculate dimensions
    let new_rows = ...;
    let new_cols = ...;

    // Store pending
    *pending_resize.borrow_mut() = Some((new_rows, new_cols, width, height));

    // Cancel old timeout
    if let Some(id) = resize_timeout_id.borrow_mut().take() {
        id.remove();
    }

    // ✅ DON'T resize grid here
    // ✅ DON'T queue_draw here

    // Set new timeout for BOTH grid and PTY
    let id = glib::timeout_add_local_once(
        Duration::from_millis(100),
        move || {
            // ✅ Resize grid and PTY together
            term.resize(TerminalSize { rows, cols });
            pty.resize(PtySize { rows, cols, px_width, px_height });

            // ✅ Single redraw after both complete
            drawing_area.queue_draw();
        },
    );

    *resize_timeout_id.borrow_mut() = Some(id);
});
```

This alone will fix 90% of your issues. The frame clock solution is for the remaining 10% (perfect smoothness).

## References

- [GTK4 Frame Clock Documentation](https://docs.gtk.org/gdk4/class.FrameClock.html)
- [GTK4 Widget Lifecycle](https://docs.gtk.org/gtk4/drawing-model.html)
- [Terminal Emulator Resize Patterns](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Window-manipulation)

---

**Summary:** Move grid resize into the debounced timeout with PTY resize. Optionally use frame clock for perfect synchronization. This ensures the terminal grid and PTY size stay in sync, eliminating blank areas and visual corruption.
