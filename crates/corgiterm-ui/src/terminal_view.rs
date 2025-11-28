//! Terminal rendering view with PTY integration

use gtk4::prelude::*;
use gtk4::glib;
use gtk4::{Adjustment, Box, DrawingArea, EventControllerKey, EventControllerScroll, EventControllerScrollFlags, GestureClick, Orientation, Scrollbar};
use std::cell::RefCell;
use std::rc::Rc;

use corgiterm_core::{Pty, PtySize, Terminal, TerminalSize, terminal::Cell};
use std::path::Path;

/// ANSI colors (standard 16 + 256 extended)
const COLORS: [(f64, f64, f64); 16] = [
    // Standard colors 0-7
    (0.118, 0.106, 0.086),  // 0: Black (background)
    (0.800, 0.341, 0.322),  // 1: Red
    (0.573, 0.706, 0.447),  // 2: Green
    (0.898, 0.659, 0.294),  // 3: Yellow
    (0.467, 0.573, 0.702),  // 4: Blue
    (0.694, 0.494, 0.627),  // 5: Magenta
    (0.529, 0.675, 0.686),  // 6: Cyan
    (0.910, 0.859, 0.769),  // 7: White (foreground)
    // Bright colors 8-15
    (0.392, 0.373, 0.345),  // 8: Bright Black
    (0.898, 0.498, 0.467),  // 9: Bright Red
    (0.714, 0.820, 0.596),  // 10: Bright Green
    (0.949, 0.792, 0.478),  // 11: Bright Yellow
    (0.627, 0.718, 0.831),  // 12: Bright Blue
    (0.824, 0.651, 0.776),  // 13: Bright Magenta
    (0.671, 0.808, 0.816),  // 14: Bright Cyan
    (0.969, 0.945, 0.910),  // 15: Bright White
];

/// Terminal view widget with PTY
pub struct TerminalView {
    container: Box,
    drawing_area: DrawingArea,
    terminal: Rc<RefCell<Terminal>>,
    pty: Rc<RefCell<Option<Pty>>>,
    /// Cell dimensions for resize calculations
    cell_width: Rc<RefCell<f64>>,
    cell_height: Rc<RefCell<f64>>,
    /// Scroll offset (0 = at bottom, positive = scrolled up into history)
    scroll_offset: Rc<RefCell<usize>>,
    /// Scrollbar adjustment
    scrollbar_adj: Adjustment,
}

impl TerminalView {
    pub fn new() -> Self {
        Self::with_working_dir(None)
    }

    pub fn with_working_dir(working_dir: Option<&Path>) -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("terminal-view");

        // Create drawing area for terminal content
        let drawing_area = DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        drawing_area.set_can_focus(true);
        drawing_area.set_focusable(true);

        // Create terminal emulator
        let (event_tx, _event_rx) = crossbeam_channel::unbounded();
        let terminal = Rc::new(RefCell::new(Terminal::new(
            TerminalSize { rows: 24, cols: 80 },
            event_tx,
        )));

        // Create PTY and spawn shell
        let pty = Rc::new(RefCell::new(None));
        {
            match Pty::spawn(None, PtySize::default(), working_dir) {
                Ok(p) => {
                    *pty.borrow_mut() = Some(p);
                    tracing::info!("PTY spawned successfully");
                }
                Err(e) => {
                    tracing::error!("Failed to spawn PTY: {}", e);
                }
            }
        }

        // Cell dimensions storage (initialized with defaults)
        let cell_width = Rc::new(RefCell::new(10.0));
        let cell_height = Rc::new(RefCell::new(20.0));

        // Scroll offset (0 = at bottom/current view)
        let scroll_offset = Rc::new(RefCell::new(0usize));

        // Set up drawing callback with Pango for text rendering
        let term_for_draw = terminal.clone();
        let cell_width_for_draw = cell_width.clone();
        let cell_height_for_draw = cell_height.clone();
        let scroll_offset_for_draw = scroll_offset.clone();
        drawing_area.set_draw_func(move |area, cr, _width, _height| {
            let term = term_for_draw.borrow();
            let grid = term.grid();
            let scrollback = term.scrollback();
            let cursor = term.cursor();
            let offset = *scroll_offset_for_draw.borrow();

            // Get Pango context and create layout
            let pango_context = area.pango_context();

            // Configure font
            let font_desc = pango::FontDescription::from_string("JetBrains Mono 12");

            // Get font metrics for cell sizing
            let metrics = pango_context.metrics(Some(&font_desc), None);
            let cell_w = (metrics.approximate_char_width() as f64 / pango::SCALE as f64).ceil();
            let cell_h = ((metrics.ascent() + metrics.descent()) as f64 / pango::SCALE as f64).ceil();
            let ascent = metrics.ascent() as f64 / pango::SCALE as f64;

            // Store cell dimensions for resize calculations
            *cell_width_for_draw.borrow_mut() = cell_w;
            *cell_height_for_draw.borrow_mut() = cell_h;

            // Padding
            let padding = 8.0;

            // Background
            let (bg_r, bg_g, bg_b) = COLORS[0];
            cr.set_source_rgb(bg_r, bg_g, bg_b);
            cr.paint().ok();

            // Create layout for text
            let layout = pango::Layout::new(&pango_context);
            layout.set_font_description(Some(&font_desc));

            // Default foreground
            let (fg_r, fg_g, fg_b) = COLORS[7];
            cr.set_source_rgb(fg_r, fg_g, fg_b);

            // Helper to draw a cell
            let draw_cell = |cr: &cairo::Context, layout: &pango::Layout, cell: &Cell, x: f64, y: f64| {
                // Draw cell background if not default
                let cell_bg = cell.bg;
                if cell_bg[0] != 30 || cell_bg[1] != 27 || cell_bg[2] != 22 {
                    cr.set_source_rgba(
                        cell_bg[0] as f64 / 255.0,
                        cell_bg[1] as f64 / 255.0,
                        cell_bg[2] as f64 / 255.0,
                        cell_bg[3] as f64 / 255.0,
                    );
                    cr.rectangle(x, y, cell_w, cell_h);
                    cr.fill().ok();
                }

                if !cell.content.is_empty() {
                    let fg = cell.fg;
                    let (r, g, b) = (
                        fg[0] as f64 / 255.0,
                        fg[1] as f64 / 255.0,
                        fg[2] as f64 / 255.0,
                    );

                    if cell.attrs.bold {
                        cr.set_source_rgb((r * 1.2).min(1.0), (g * 1.2).min(1.0), (b * 1.2).min(1.0));
                    } else if cell.attrs.dim {
                        cr.set_source_rgb(r * 0.6, g * 0.6, b * 0.6);
                    } else {
                        cr.set_source_rgb(r, g, b);
                    }

                    layout.set_text(&cell.content);
                    cr.move_to(x, y + (cell_h - ascent) / 2.0);
                    pangocairo::functions::show_layout(cr, layout);
                }
            };

            // Calculate visible lines based on scroll offset
            // offset=0 means we show the current grid (bottom)
            // offset>0 means we show some scrollback
            let visible_rows = grid.len();
            let scrollback_len = scrollback.len();
            let max_offset = scrollback_len;
            let effective_offset = offset.min(max_offset);

            for screen_row in 0..visible_rows {
                // Calculate which line to show
                // If offset=0, show grid[screen_row]
                // If offset>0, show from scrollback (older lines)
                let source_line = if effective_offset > 0 {
                    // How many lines from scrollback to show
                    let scrollback_lines_to_show = effective_offset.min(visible_rows);

                    if screen_row < scrollback_lines_to_show {
                        // This row comes from scrollback
                        let scrollback_idx = scrollback_len - effective_offset + screen_row;
                        if scrollback_idx < scrollback_len {
                            Some(("scrollback", scrollback_idx))
                        } else {
                            None
                        }
                    } else {
                        // This row comes from grid
                        let grid_row = screen_row - scrollback_lines_to_show;
                        if grid_row < grid.len() {
                            Some(("grid", grid_row))
                        } else {
                            None
                        }
                    }
                } else {
                    Some(("grid", screen_row))
                };

                let y = padding + (screen_row as f64 * cell_h);

                if let Some((source, idx)) = source_line {
                    let row = match source {
                        "scrollback" => &scrollback[idx],
                        "grid" => &grid[idx],
                        _ => continue,
                    };
                    for (col_idx, cell) in row.iter().enumerate() {
                        let x = padding + (col_idx as f64 * cell_w);
                        draw_cell(cr, &layout, cell, x, y);
                    }
                }
            }

            // Only draw cursor if not scrolled up (offset == 0)
            if offset == 0 {
                let cursor_x = padding + (cursor.1 as f64 * cell_w);
                let cursor_y = padding + (cursor.0 as f64 * cell_h);

                let (accent_r, accent_g, accent_b) = COLORS[3];
                cr.set_source_rgba(accent_r, accent_g, accent_b, 0.8);
                cr.rectangle(cursor_x, cursor_y, cell_w, cell_h);
                cr.fill().ok();

                if cursor.0 < grid.len() && cursor.1 < grid[cursor.0].len() {
                    let cell = &grid[cursor.0][cursor.1];
                    if !cell.content.is_empty() {
                        cr.set_source_rgb(bg_r, bg_g, bg_b);
                        layout.set_text(&cell.content);
                        cr.move_to(cursor_x, cursor_y + (cell_h - ascent) / 2.0);
                        pangocairo::functions::show_layout(cr, &layout);
                    }
                }
            }
        });

        // Set up resize handling
        let term_for_resize = terminal.clone();
        let pty_for_resize = pty.clone();
        let cell_width_for_resize = cell_width.clone();
        let cell_height_for_resize = cell_height.clone();
        let drawing_area_for_resize = drawing_area.clone();

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

            tracing::debug!("Resizing terminal to {}x{}", new_rows, new_cols);

            // Resize terminal emulator
            let new_terminal_size = TerminalSize {
                rows: new_rows,
                cols: new_cols,
            };
            term_for_resize.borrow_mut().resize(new_terminal_size);

            // Resize PTY
            if let Some(ref mut pty) = *pty_for_resize.borrow_mut() {
                let new_pty_size = PtySize {
                    rows: new_rows as u16,
                    cols: new_cols as u16,
                    pixel_width: width as u16,
                    pixel_height: height as u16,
                };
                if let Err(e) = pty.resize(new_pty_size) {
                    tracing::error!("Failed to resize PTY: {}", e);
                }
            }

            // Queue redraw
            drawing_area_for_resize.queue_draw();
        });

        // Set up keyboard input
        let key_controller = EventControllerKey::new();
        let pty_for_input = pty.clone();
        let drawing_area_for_clipboard = drawing_area.clone();
        let terminal_for_copy = terminal.clone();
        key_controller.connect_key_pressed(move |_, key, _keycode, modifier| {
            use gtk4::gdk::{Key, ModifierType};

            let ctrl = modifier.contains(ModifierType::CONTROL_MASK);
            let shift = modifier.contains(ModifierType::SHIFT_MASK);

            // Check for Ctrl+Shift+C (copy)
            if ctrl && shift && matches!(key, Key::C | Key::c) {
                // Copy visible terminal content to clipboard
                let clipboard = drawing_area_for_clipboard.clipboard();
                let term = terminal_for_copy.borrow();
                let grid = term.grid();

                // Collect all visible lines
                let mut lines = Vec::new();
                for row in grid.iter() {
                    let mut line = String::new();
                    for cell in row.iter() {
                        line.push_str(&cell.content);
                    }
                    // Trim trailing whitespace from each line
                    lines.push(line.trim_end().to_string());
                }

                // Join lines and set clipboard
                let text = lines.join("\n");
                clipboard.set_text(&text);

                return glib::Propagation::Stop;
            }

            // Check for Ctrl+Shift+V (paste)
            if ctrl && shift && matches!(key, Key::V | Key::v) {
                // Handle paste from clipboard
                let clipboard = drawing_area_for_clipboard.clipboard();
                let pty_clone = pty_for_input.clone();

                clipboard.read_text_async(None::<&gtk4::gio::Cancellable>, move |result| {
                    if let Ok(Some(text)) = result {
                        if let Some(ref pty) = *pty_clone.borrow() {
                            // Write clipboard text to PTY
                            let _ = pty.write(text.as_bytes());
                        }
                    }
                });

                return glib::Propagation::Stop;
            }

            if let Some(ref pty) = *pty_for_input.borrow() {
                // Convert GDK key to bytes
                let bytes = key_to_bytes(key, modifier);
                if !bytes.is_empty() {
                    let _ = pty.write(&bytes);
                }
            }
            glib::Propagation::Stop
        });
        drawing_area.add_controller(key_controller);

        // Add click handler to grab focus when clicked
        let click_gesture = GestureClick::new();
        let drawing_area_for_focus = drawing_area.clone();
        click_gesture.connect_pressed(move |_, _n_press, _x, _y| {
            drawing_area_for_focus.grab_focus();
        });
        drawing_area.add_controller(click_gesture);

        // Add mouse wheel scroll for scrollback
        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        let scroll_offset_for_wheel = scroll_offset.clone();
        let terminal_for_scroll = terminal.clone();
        let drawing_area_for_scroll = drawing_area.clone();
        scroll_controller.connect_scroll(move |_, _dx, dy| {
            let term = terminal_for_scroll.borrow();
            let scrollback_len = term.scrollback().len();
            drop(term);

            let mut offset = scroll_offset_for_wheel.borrow_mut();

            // dy > 0 means scroll down (toward bottom/newer), dy < 0 means scroll up (toward top/older)
            if dy < 0.0 {
                // Scroll up into history
                *offset = (*offset + 3).min(scrollback_len);
            } else if dy > 0.0 {
                // Scroll down toward current
                *offset = offset.saturating_sub(3);
            }

            drawing_area_for_scroll.queue_draw();
            glib::Propagation::Stop
        });
        drawing_area.add_controller(scroll_controller);

        // Set up PTY reading with glib timeout
        let term_for_read = terminal.clone();
        let pty_for_read = pty.clone();
        let drawing_area_clone = drawing_area.clone();
        let scroll_offset_for_reset = scroll_offset.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            if let Some(ref pty) = *pty_for_read.borrow() {
                let mut buf = [0u8; 4096];
                // Set non-blocking read
                unsafe {
                    let flags = libc::fcntl(pty.master_fd(), libc::F_GETFL);
                    libc::fcntl(pty.master_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK);
                }
                match pty.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        term_for_read.borrow_mut().process(&buf[..n]);
                        // Reset scroll to bottom when new output arrives
                        *scroll_offset_for_reset.borrow_mut() = 0;
                        drawing_area_clone.queue_draw();
                    }
                    _ => {}
                }
            }
            glib::ControlFlow::Continue
        });

        // Create scrollbar with adjustment
        // Adjustment: value, lower, upper, step_increment, page_increment, page_size
        // We use inverted values: 0 = bottom (current), upper = max scrollback
        let scrollbar_adj = Adjustment::new(0.0, 0.0, 1.0, 1.0, 10.0, 1.0);
        let scrollbar = Scrollbar::new(Orientation::Vertical, Some(&scrollbar_adj));
        scrollbar.set_vexpand(true);

        // Connect scrollbar to scroll_offset
        let scroll_offset_for_adj = scroll_offset.clone();
        let drawing_area_for_adj = drawing_area.clone();
        scrollbar_adj.connect_value_changed(move |adj| {
            let value = adj.value() as usize;
            let upper = adj.upper() as usize;
            // Invert: scrollbar at top (max value) = scrolled up into history
            // scrollbar at bottom (0) = at current content
            *scroll_offset_for_adj.borrow_mut() = upper.saturating_sub(value);
            drawing_area_for_adj.queue_draw();
        });

        // Create horizontal box: drawing area + scrollbar
        let content_box = Box::new(Orientation::Horizontal, 0);
        content_box.set_vexpand(true);
        content_box.set_hexpand(true);
        drawing_area.set_hexpand(true);
        content_box.append(&drawing_area);
        content_box.append(&scrollbar);
        container.append(&content_box);

        // Focus the drawing area
        drawing_area.grab_focus();

        Self {
            container,
            drawing_area,
            terminal,
            pty,
            cell_width,
            cell_height,
            scroll_offset,
            scrollbar_adj,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Queue a redraw
    pub fn queue_draw(&self) {
        self.drawing_area.queue_draw();
    }
}

impl Default for TerminalView {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert GDK key press to terminal bytes
fn key_to_bytes(key: gtk4::gdk::Key, modifier: gtk4::gdk::ModifierType) -> Vec<u8> {
    use gtk4::gdk::Key;
    use gtk4::gdk::ModifierType;

    let ctrl = modifier.contains(ModifierType::CONTROL_MASK);

    match key {
        Key::Return | Key::KP_Enter => vec![b'\r'],
        Key::BackSpace => vec![0x7f],
        Key::Tab => vec![b'\t'],
        Key::Escape => vec![0x1b],
        Key::Up => vec![0x1b, b'[', b'A'],
        Key::Down => vec![0x1b, b'[', b'B'],
        Key::Right => vec![0x1b, b'[', b'C'],
        Key::Left => vec![0x1b, b'[', b'D'],
        Key::Home => vec![0x1b, b'[', b'H'],
        Key::End => vec![0x1b, b'[', b'F'],
        Key::Page_Up => vec![0x1b, b'[', b'5', b'~'],
        Key::Page_Down => vec![0x1b, b'[', b'6', b'~'],
        Key::Insert => vec![0x1b, b'[', b'2', b'~'],
        Key::Delete => vec![0x1b, b'[', b'3', b'~'],
        Key::F1 => vec![0x1b, b'O', b'P'],
        Key::F2 => vec![0x1b, b'O', b'Q'],
        Key::F3 => vec![0x1b, b'O', b'R'],
        Key::F4 => vec![0x1b, b'O', b'S'],
        Key::F5 => vec![0x1b, b'[', b'1', b'5', b'~'],
        Key::F6 => vec![0x1b, b'[', b'1', b'7', b'~'],
        Key::F7 => vec![0x1b, b'[', b'1', b'8', b'~'],
        Key::F8 => vec![0x1b, b'[', b'1', b'9', b'~'],
        Key::F9 => vec![0x1b, b'[', b'2', b'0', b'~'],
        Key::F10 => vec![0x1b, b'[', b'2', b'1', b'~'],
        Key::F11 => vec![0x1b, b'[', b'2', b'3', b'~'],
        Key::F12 => vec![0x1b, b'[', b'2', b'4', b'~'],
        _ => {
            if let Some(c) = key.to_unicode() {
                if ctrl && c.is_ascii_alphabetic() {
                    // Ctrl+letter: send control character
                    vec![(c.to_ascii_lowercase() as u8) - b'a' + 1]
                } else {
                    c.to_string().into_bytes()
                }
            } else {
                vec![]
            }
        }
    }
}
