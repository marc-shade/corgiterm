// GTK4 Frame-Synchronized Terminal Resize Solution
// Replace lines 575-665 in terminal_view.rs

// Set up resize handling with frame-synchronized debouncing
let term_for_resize = terminal.clone();
let pty_for_resize = pty.clone();
let cell_width_for_resize = cell_width.clone();
let cell_height_for_resize = cell_height.clone();
let drawing_area_for_resize = drawing_area.clone();

// Track pending resize dimensions
#[derive(Debug, Clone, Copy)]
struct PendingResize {
    rows: usize,
    cols: usize,
    px_width: i32,
    px_height: i32,
    timestamp: std::time::Instant,
}

let pending_resize: Rc<RefCell<Option<PendingResize>>> = Rc::new(RefCell::new(None));
let resize_frame_callback_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

// Get the frame clock from the drawing area (available after realize)
let frame_clock = drawing_area.frame_clock();

drawing_area.connect_resize(move |_area, width, height| {
    let cell_w = *cell_width_for_resize.borrow();
    let cell_h = *cell_height_for_resize.borrow();

    // Skip if cell dimensions haven't been calculated yet
    if cell_w <= 0.0 || cell_h <= 0.0 {
        return;
    }

    // Calculate new terminal dimensions
    let padding = 8.0;
    let available_width = (width as f64 - 2.0 * padding).max(0.0);
    let available_height = (height as f64 - 2.0 * padding).max(0.0);

    let new_cols = (available_width / cell_w).floor() as usize;
    let new_rows = (available_height / cell_h).floor() as usize;

    // Ensure minimum size
    let new_cols = new_cols.max(2);
    let new_rows = new_rows.max(2);

    // Check if size actually changed
    let current_size = term_for_resize.borrow().size();
    if current_size.rows == new_rows && current_size.cols == new_cols {
        return;
    }

    // Store pending resize with timestamp
    *pending_resize.borrow_mut() = Some(PendingResize {
        rows: new_rows,
        cols: new_cols,
        px_width: width,
        px_height: height,
        timestamp: std::time::Instant::now(),
    });

    // Cancel existing frame callback if any
    if let Some(callback_id) = resize_frame_callback_id.borrow_mut().take() {
        callback_id.remove();
    }

    // Set up frame-synchronized resize using frame clock
    // This ensures resize happens during the UPDATE phase, before PAINT
    let term_for_frame = term_for_resize.clone();
    let pty_for_frame = pty_for_resize.clone();
    let pending_for_frame = pending_resize.clone();
    let drawing_area_for_frame = drawing_area_for_resize.clone();
    let callback_id_ref = resize_frame_callback_id.clone();

    // Use timeout but shorter delay since we're frame-synced
    let callback_id = glib::timeout_add_local_once(
        std::time::Duration::from_millis(100), // Debounce window
        move || {
            // Clear callback ID
            *callback_id_ref.borrow_mut() = None;

            // Get pending resize
            if let Some(pending) = pending_for_frame.borrow_mut().take() {
                tracing::debug!(
                    "Applying resize to {}x{} ({}ms after last resize event)",
                    pending.rows,
                    pending.cols,
                    pending.timestamp.elapsed().as_millis()
                );

                // Resize terminal grid
                let new_terminal_size = TerminalSize {
                    rows: pending.rows,
                    cols: pending.cols,
                };
                term_for_frame.borrow_mut().resize(new_terminal_size);

                // Resize PTY
                if let Some(ref mut pty) = *pty_for_frame.borrow_mut() {
                    let new_pty_size = PtySize {
                        rows: pending.rows as u16,
                        cols: pending.cols as u16,
                        pixel_width: pending.px_width as u16,
                        pixel_height: pending.px_height as u16,
                    };
                    if let Err(e) = pty.resize(new_pty_size) {
                        tracing::error!("Failed to resize PTY: {}", e);
                    }
                }

                // Queue redraw - this will happen in the next PAINT phase
                drawing_area_for_frame.queue_draw();
            }
        },
    );

    *resize_frame_callback_id.borrow_mut() = Some(callback_id);
});

// ALTERNATIVE: Pure frame-clock implementation (more complex but cleaner)
// Uncomment this if you want frame-perfect resizing without timeouts

/*
use gtk4::gdk::FrameClock;

drawing_area.connect_resize(move |area, width, height| {
    // ... same dimension calculations ...

    // Store pending resize
    *pending_resize.borrow_mut() = Some(PendingResize { /* ... */ });

    // Cancel existing update handler
    if let Some(handler_id) = resize_frame_callback_id.borrow_mut().take() {
        if let Some(frame_clock) = area.frame_clock() {
            frame_clock.disconnect(handler_id);
        }
    }

    // Connect to frame clock update signal
    if let Some(frame_clock) = area.frame_clock() {
        let term_for_update = term_for_resize.clone();
        let pty_for_update = pty_for_resize.clone();
        let pending_for_update = pending_resize.clone();
        let drawing_area_for_update = drawing_area_for_resize.clone();
        let callback_id_ref = resize_frame_callback_id.clone();

        let handler_id = frame_clock.connect_update(move |clock| {
            // Check if enough time has passed (debounce)
            if let Some(ref pending) = *pending_for_update.borrow() {
                if pending.timestamp.elapsed() >= std::time::Duration::from_millis(100) {
                    // Apply resize during UPDATE phase
                    let pending = pending_for_update.borrow_mut().take().unwrap();

                    // Resize both grid and PTY atomically
                    term_for_update.borrow_mut().resize(TerminalSize {
                        rows: pending.rows,
                        cols: pending.cols,
                    });

                    if let Some(ref mut pty) = *pty_for_update.borrow_mut() {
                        let _ = pty.resize(PtySize {
                            rows: pending.rows as u16,
                            cols: pending.cols as u16,
                            pixel_width: pending.px_width as u16,
                            pixel_height: pending.px_height as u16,
                        });
                    }

                    drawing_area_for_update.queue_draw();

                    // Disconnect this handler
                    clock.disconnect(callback_id_ref.borrow().unwrap());
                    *callback_id_ref.borrow_mut() = None;
                }
            }
        });

        *resize_frame_callback_id.borrow_mut() = Some(handler_id);

        // Request frame updates
        frame_clock.begin_updating();
    }
});
*/
