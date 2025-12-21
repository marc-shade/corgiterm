//! Terminal rendering view with PTY integration

use gtk4::gio::{Menu, SimpleAction};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box, DrawingArea, Entry, EventControllerKey, EventControllerMotion,
    EventControllerScroll, EventControllerScrollFlags, GestureClick, GestureDrag, Label,
    Orientation, PopoverMenu, Revealer, RevealerTransitionType, Scrollbar,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::config_manager;
use corgiterm_config::themes::ThemeManager;
use corgiterm_core::{
    terminal::Cell, HintDetector, HintModeState, Pty, PtySize, Terminal, TerminalSize,
};
use std::path::Path;

/// URL regex pattern
static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://[^\s<>\[\]{}|\\^`\x00-\x1f\x7f]+").unwrap());

/// Detected URL with position info
#[derive(Debug, Clone)]
struct DetectedUrl {
    url: String,
    row: usize,
    start_col: usize,
    end_col: usize,
}

/// Search state
#[derive(Debug, Clone, Default)]
struct SearchState {
    /// Is search active?
    active: bool,
    /// Search query
    query: String,
    /// Matching positions (row, start_col, end_col)
    matches: Vec<(usize, usize, usize)>,
    /// Current match index
    current_match: usize,
}

/// Selection state for text selection
#[derive(Debug, Clone, Copy, Default)]
struct Selection {
    /// Is selection active?
    active: bool,
    /// Start position (row, col)
    start: (usize, usize),
    /// End position (row, col)
    end: (usize, usize),
}

/// Default ANSI colors (fallback if theme not loaded)
const DEFAULT_COLORS: [(f64, f64, f64); 16] = [
    // Standard colors 0-7
    (0.118, 0.106, 0.086), // 0: Black (background)
    (0.800, 0.341, 0.322), // 1: Red
    (0.573, 0.706, 0.447), // 2: Green
    (0.898, 0.659, 0.294), // 3: Yellow
    (0.467, 0.573, 0.702), // 4: Blue
    (0.694, 0.494, 0.627), // 5: Magenta
    (0.529, 0.675, 0.686), // 6: Cyan
    (0.910, 0.859, 0.769), // 7: White (foreground)
    // Bright colors 8-15
    (0.392, 0.373, 0.345), // 8: Bright Black
    (0.898, 0.498, 0.467), // 9: Bright Red
    (0.714, 0.820, 0.596), // 10: Bright Green
    (0.949, 0.792, 0.478), // 11: Bright Yellow
    (0.627, 0.718, 0.831), // 12: Bright Blue
    (0.824, 0.651, 0.776), // 13: Bright Magenta
    (0.671, 0.808, 0.816), // 14: Bright Cyan
    (0.969, 0.945, 0.910), // 15: Bright White
];

/// Convert hex color string to RGB tuple
fn hex_to_rgb(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(hex.get(0..2).unwrap_or("00"), 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(hex.get(2..4).unwrap_or("00"), 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(hex.get(4..6).unwrap_or("00"), 16).unwrap_or(0) as f64 / 255.0;
    (r, g, b)
}

/// Load colors from the current theme
fn load_theme_colors() -> [(f64, f64, f64); 16] {
    // Try to load from config's theme
    if let Some(config_manager) = config_manager() {
        let config = config_manager.read();
        let theme_name = &config.config().appearance.theme;

        let theme_manager = ThemeManager::new();
        if let Some(theme) = theme_manager.get(theme_name) {
            let colors = &theme.colors;
            // Index 0 = background, Index 7 = foreground (used for default colors)
            // Other indices = ANSI colors
            return [
                hex_to_rgb(&colors.background),     // 0: Background
                hex_to_rgb(&colors.red),            // 1: Red
                hex_to_rgb(&colors.green),          // 2: Green
                hex_to_rgb(&colors.yellow),         // 3: Yellow
                hex_to_rgb(&colors.blue),           // 4: Blue
                hex_to_rgb(&colors.magenta),        // 5: Magenta
                hex_to_rgb(&colors.cyan),           // 6: Cyan
                hex_to_rgb(&colors.foreground),     // 7: Foreground
                hex_to_rgb(&colors.bright_black),   // 8: Bright Black
                hex_to_rgb(&colors.bright_red),     // 9: Bright Red
                hex_to_rgb(&colors.bright_green),   // 10: Bright Green
                hex_to_rgb(&colors.bright_yellow),  // 11: Bright Yellow
                hex_to_rgb(&colors.bright_blue),    // 12: Bright Blue
                hex_to_rgb(&colors.bright_magenta), // 13: Bright Magenta
                hex_to_rgb(&colors.bright_cyan),    // 14: Bright Cyan
                hex_to_rgb(&colors.bright_white),   // 15: Bright White
            ];
        }
    }

    // Fallback to default colors
    DEFAULT_COLORS
}

/// Terminal view widget with PTY
#[allow(dead_code)]
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
    /// Text selection state
    selection: Rc<RefCell<Selection>>,
    /// Current hover position (row, col)
    hover_pos: Rc<RefCell<Option<(usize, usize)>>>,
    /// Detected URLs in current view
    detected_urls: Rc<RefCell<Vec<DetectedUrl>>>,
    /// Search state
    search_state: Rc<RefCell<SearchState>>,
    /// Terminal event receiver
    event_rx: Rc<crossbeam_channel::Receiver<corgiterm_core::terminal::TerminalEvent>>,
    /// Cursor visible state (for blinking)
    cursor_visible: Rc<RefCell<bool>>,
    /// Visual bell flash state
    bell_flash: Rc<RefCell<bool>>,
    /// Theme colors (loaded from config)
    colors: Rc<RefCell<[(f64, f64, f64); 16]>>,
    /// PTY size - used to clip rendering during resize to prevent ghost lines
    /// When grid size != PTY size, we're in a resize transition
    pty_cols: Rc<RefCell<usize>>,
    /// Hint mode state for URL/path hints (foot-style keyboard navigation)
    hint_mode: Rc<RefCell<HintModeState>>,
    /// Hint detector for scanning terminal buffer
    hint_detector: Rc<HintDetector>,
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
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let terminal = Rc::new(RefCell::new(Terminal::new(
            TerminalSize { rows: 24, cols: 80 },
            event_tx,
        )));

        // Apply scrollback setting from config
        if let Some(config_manager) = crate::app::config_manager() {
            let scrollback = config_manager.read().config().terminal.scrollback_lines;
            terminal.borrow_mut().set_max_scrollback(scrollback);
        }

        let event_rx = Rc::new(event_rx);

        // Create PTY and spawn shell
        let (shell, term) = crate::app::config_manager()
            .map(|cm| {
                let config = cm.read().config();
                (config.general.shell.clone(), config.terminal.term.clone())
            })
            .unwrap_or_else(|| {
                (
                    std::env::var("SHELL").unwrap_or("/bin/bash".to_string()),
                    "xterm-256color".to_string(),
                )
            });
        let pty = Rc::new(RefCell::new(None));
        {
            match Pty::spawn(Some(&shell), PtySize::default(), working_dir, Some(&term)) {
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

        // Selection state
        let selection = Rc::new(RefCell::new(Selection::default()));

        // Hover position for URL highlighting
        let hover_pos: Rc<RefCell<Option<(usize, usize)>>> = Rc::new(RefCell::new(None));

        // Detected URLs cache
        let detected_urls: Rc<RefCell<Vec<DetectedUrl>>> = Rc::new(RefCell::new(Vec::new()));

        // Search state
        let search_state: Rc<RefCell<SearchState>> = Rc::new(RefCell::new(SearchState::default()));

        // Cursor blink state
        let cursor_visible: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));

        // Visual bell flash state
        let bell_flash: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

        // Load initial colors from theme
        let colors: Rc<RefCell<[(f64, f64, f64); 16]>> = Rc::new(RefCell::new(load_theme_colors()));

        // PTY column count - used to clip rendering during resize transitions
        // Initialize to default PTY size (80 columns)
        let pty_cols: Rc<RefCell<usize>> = Rc::new(RefCell::new(80));

        // Hint mode state for URL/path keyboard navigation (foot-style)
        let hint_mode: Rc<RefCell<HintModeState>> = Rc::new(RefCell::new(HintModeState::new()));
        let hint_detector: Rc<HintDetector> = Rc::new(HintDetector::new());

        // Set up drawing callback with Pango for text rendering
        let term_for_draw = terminal.clone();
        let cell_width_for_draw = cell_width.clone();
        let cell_height_for_draw = cell_height.clone();
        let scroll_offset_for_draw = scroll_offset.clone();
        let selection_for_draw = selection.clone();
        let hover_pos_for_draw = hover_pos.clone();
        let detected_urls_for_draw = detected_urls.clone();
        let search_state_for_draw = search_state.clone();
        let cursor_visible_for_draw = cursor_visible.clone();
        let bell_flash_for_draw = bell_flash.clone();
        let pty_cols_for_draw = pty_cols.clone();
        let colors_for_draw = colors.clone();
        let hint_mode_for_draw = hint_mode.clone();
        drawing_area.set_draw_func(move |area, cr, _width, _height| {
            // Use cached theme colors (updated via reload_theme_colors())
            let current_colors = *colors_for_draw.borrow();
            let term = term_for_draw.borrow();
            let grid = term.grid();
            let scrollback = term.scrollback();
            let cursor = term.cursor();
            let offset = *scroll_offset_for_draw.borrow();

            // Get Pango context and create layout
            let pango_context = area.pango_context();

            // Configure font from config or use default
            let (font_family, font_size) =
                if let Some(config_manager) = crate::app::config_manager() {
                    let config = config_manager.read().config();
                    (
                        config.appearance.font_family.clone(),
                        config.appearance.font_size,
                    )
                } else {
                    ("Source Code Pro".to_string(), 11.0)
                };
            let font_string = format!("{} {}", font_family, font_size as u32);
            let font_desc = pango::FontDescription::from_string(&font_string);

            // Get font metrics for cell sizing
            let metrics = pango_context.metrics(Some(&font_desc), None);
            let cell_w = (metrics.approximate_char_width() as f64 / pango::SCALE as f64).ceil();
            let cell_h =
                ((metrics.ascent() + metrics.descent()) as f64 / pango::SCALE as f64).ceil();
            let ascent = metrics.ascent() as f64 / pango::SCALE as f64;

            // Store cell dimensions for resize calculations
            *cell_width_for_draw.borrow_mut() = cell_w;
            *cell_height_for_draw.borrow_mut() = cell_h;

            // Padding
            let padding = 8.0;

            // Background
            let (bg_r, bg_g, bg_b) = current_colors[0];
            cr.set_source_rgb(bg_r, bg_g, bg_b);
            cr.paint().ok();

            // Create layout for text
            let layout = pango::Layout::new(&pango_context);
            layout.set_font_description(Some(&font_desc));

            // Default foreground
            let (fg_r, fg_g, fg_b) = current_colors[7];
            cr.set_source_rgb(fg_r, fg_g, fg_b);

            // Helper to draw a cell
            let draw_cell = |cr: &cairo::Context,
                             layout: &pango::Layout,
                             cell: &Cell,
                             x: f64,
                             y: f64,
                             font_desc: &pango::FontDescription| {
                // Handle inverse and hidden attributes
                let cell_fg = if cell.attrs.inverse {
                    cell.bg // Swap fg/bg for inverse
                } else {
                    cell.fg
                };
                let cell_bg_draw = if cell.attrs.inverse {
                    cell.fg // Swap fg/bg for inverse
                } else {
                    cell.bg
                };

                if cell_bg_draw[0] != 30 || cell_bg_draw[1] != 27 || cell_bg_draw[2] != 22 {
                    cr.set_source_rgba(
                        cell_bg_draw[0] as f64 / 255.0,
                        cell_bg_draw[1] as f64 / 255.0,
                        cell_bg_draw[2] as f64 / 255.0,
                        cell_bg_draw[3] as f64 / 255.0,
                    );
                    cr.rectangle(x, y, cell_w, cell_h);
                    cr.fill().ok();
                }

                if !cell.content.is_empty() && !cell.attrs.hidden {
                    let fg = cell_fg;
                    let (r, g, b) = (
                        fg[0] as f64 / 255.0,
                        fg[1] as f64 / 255.0,
                        fg[2] as f64 / 255.0,
                    );

                    // Apply color modifiers
                    if cell.attrs.bold {
                        cr.set_source_rgb(
                            (r * 1.2).min(1.0),
                            (g * 1.2).min(1.0),
                            (b * 1.2).min(1.0),
                        );
                    } else if cell.attrs.dim {
                        cr.set_source_rgb(r * 0.6, g * 0.6, b * 0.6);
                    } else {
                        cr.set_source_rgb(r, g, b);
                    }

                    // Apply font styles (italic, bold weight)
                    let mut styled_font = font_desc.clone();
                    if cell.attrs.italic {
                        styled_font.set_style(pango::Style::Italic);
                    }
                    if cell.attrs.bold {
                        styled_font.set_weight(pango::Weight::Bold);
                    }
                    layout.set_font_description(Some(&styled_font));

                    layout.set_text(&cell.content);
                    let text_y = y + (cell_h - ascent) / 2.0;
                    cr.move_to(x, text_y);
                    pangocairo::functions::show_layout(cr, layout);

                    // Draw underline
                    if cell.attrs.underline {
                        cr.set_line_width(1.0);
                        let underline_y = y + cell_h - 2.0;
                        cr.move_to(x, underline_y);
                        cr.line_to(x + cell_w, underline_y);
                        cr.stroke().ok();
                    }

                    // Draw strikethrough
                    if cell.attrs.strikethrough {
                        cr.set_line_width(1.0);
                        let strike_y = y + cell_h / 2.0;
                        cr.move_to(x, strike_y);
                        cr.line_to(x + cell_w, strike_y);
                        cr.stroke().ok();
                    }

                    // Reset font description for next cell
                    layout.set_font_description(Some(font_desc));
                }
            };

            // Calculate visible lines based on scroll offset
            // offset=0 means we show the current grid (bottom)
            // offset>0 means we show some scrollback
            let visible_rows = grid.len();
            let scrollback_len = scrollback.len();
            let max_offset = scrollback_len;
            let effective_offset = offset.min(max_offset);

            // Detect URLs in visible grid content
            let mut urls = Vec::new();
            for (row_idx, row) in grid.iter().enumerate() {
                // Build line text
                let line_text: String = row
                    .iter()
                    .map(|c| {
                        if c.content.is_empty() {
                            ' '
                        } else {
                            c.content.chars().next().unwrap_or(' ')
                        }
                    })
                    .collect();

                // Find URLs in this line
                for mat in URL_REGEX.find_iter(&line_text) {
                    urls.push(DetectedUrl {
                        url: mat.as_str().to_string(),
                        row: row_idx,
                        start_col: mat.start(),
                        end_col: mat.end() - 1,
                    });
                }
            }
            *detected_urls_for_draw.borrow_mut() = urls.clone();

            // Get current hover position
            let hover = *hover_pos_for_draw.borrow();

            // Get search state
            let search = search_state_for_draw.borrow();

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

                    // Get actual row index for selection checking
                    let actual_row = if source == "grid" { idx } else { screen_row };
                    let sel = selection_for_draw.borrow();

                    // Clip columns to PTY size to prevent ghost lines during resize
                    // When grid is larger than PTY, don't draw the extra columns
                    let max_cols = *pty_cols_for_draw.borrow();

                    for (col_idx, cell) in row.iter().enumerate() {
                        // Skip columns beyond PTY size to prevent stale content display
                        if col_idx >= max_cols {
                            break;
                        }
                        let x = padding + (col_idx as f64 * cell_w);

                        // Check if this cell is selected (only for grid content)
                        if source == "grid" && is_cell_selected(actual_row, col_idx, &sel) {
                            // Draw selection highlight background
                            cr.set_source_rgba(0.467, 0.573, 0.702, 0.5); // Blue with transparency
                            cr.rectangle(x, y, cell_w, cell_h);
                            cr.fill().ok();
                        }

                        // Check if this cell is part of a search match
                        if source == "grid" && search.active {
                            for (match_idx, (match_row, match_start, match_end)) in
                                search.matches.iter().enumerate()
                            {
                                if actual_row == *match_row
                                    && col_idx >= *match_start
                                    && col_idx < *match_end
                                {
                                    // Different color for current match vs other matches
                                    if match_idx == search.current_match {
                                        // Current match: bright orange/yellow
                                        cr.set_source_rgba(0.949, 0.792, 0.478, 0.7);
                                    } else {
                                        // Other matches: dimmer yellow
                                        cr.set_source_rgba(0.898, 0.659, 0.294, 0.4);
                                    }
                                    cr.rectangle(x, y, cell_w, cell_h);
                                    cr.fill().ok();
                                    break;
                                }
                            }
                        }

                        // Check if this cell is part of a URL that's being hovered
                        let is_url_hovered = if source == "grid" {
                            if let Some((hover_row, hover_col)) = hover {
                                urls.iter().any(|url| {
                                    url.row == actual_row
                                        && col_idx >= url.start_col
                                        && col_idx <= url.end_col
                                        && hover_row == url.row
                                        && hover_col >= url.start_col
                                        && hover_col <= url.end_col
                                })
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        draw_cell(cr, &layout, cell, x, y, &font_desc);

                        // Draw underline for hovered URLs
                        if is_url_hovered {
                            cr.set_source_rgba(0.467, 0.573, 0.702, 0.9); // Blue for links
                            let underline_y = y + cell_h - 2.0;
                            cr.move_to(x, underline_y);
                            cr.line_to(x + cell_w, underline_y);
                            cr.set_line_width(1.0);
                            cr.stroke().ok();
                        }
                    }
                }
            }

            // Only draw cursor if not scrolled up (offset == 0) and cursor is visible (for blink)
            let cursor_is_visible = *cursor_visible_for_draw.borrow();
            let (cursor_blink_enabled, _blink_rate) = crate::app::config_manager()
                .map(|cm| {
                    let config = cm.read().config();
                    (
                        config.appearance.cursor_blink,
                        config.appearance.cursor_blink_rate,
                    )
                })
                .unwrap_or((true, 530));

            // Show cursor if: not scrolled, and (blink disabled OR cursor is in visible phase)
            if offset == 0 && (!cursor_blink_enabled || cursor_is_visible) {
                let cursor_x = padding + (cursor.1 as f64 * cell_w);
                let cursor_y = padding + (cursor.0 as f64 * cell_h);

                // Get cursor style from config
                let cursor_style = crate::app::config_manager()
                    .map(|cm| cm.read().config().appearance.cursor_style)
                    .unwrap_or(corgiterm_config::CursorStyle::Block);

                let (accent_r, accent_g, accent_b) = current_colors[3];
                cr.set_source_rgba(accent_r, accent_g, accent_b, 0.8);

                match cursor_style {
                    corgiterm_config::CursorStyle::Block => {
                        cr.rectangle(cursor_x, cursor_y, cell_w, cell_h);
                        cr.fill().ok();
                        // Draw character in inverse color
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
                    corgiterm_config::CursorStyle::Underline => {
                        let line_height = 2.0;
                        cr.rectangle(
                            cursor_x,
                            cursor_y + cell_h - line_height,
                            cell_w,
                            line_height,
                        );
                        cr.fill().ok();
                    }
                    corgiterm_config::CursorStyle::Bar => {
                        let bar_width = 2.0;
                        cr.rectangle(cursor_x, cursor_y, bar_width, cell_h);
                        cr.fill().ok();
                    }
                    corgiterm_config::CursorStyle::Hollow => {
                        cr.set_line_width(1.0);
                        cr.rectangle(cursor_x + 0.5, cursor_y + 0.5, cell_w - 1.0, cell_h - 1.0);
                        cr.stroke().ok();
                    }
                }
            }

            // Draw hint mode overlay (foot-style URL hints)
            let hint_state = hint_mode_for_draw.borrow();
            if hint_state.active {
                // Dim the background slightly to highlight hints
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
                cr.paint().ok();

                // Draw each hint with its label
                for hint in &hint_state.hints {
                    // Calculate hint position
                    let hint_x = padding + (hint.col_start as f64 * cell_w);
                    let hint_y = padding + (hint.row as f64 * cell_h);
                    let hint_width = ((hint.col_end - hint.col_start) as f64 * cell_w).max(cell_w);

                    // Draw highlight box behind the hinted text
                    cr.set_source_rgba(0.467, 0.573, 0.702, 0.3); // Blue tint
                    cr.rectangle(hint_x, hint_y, hint_width, cell_h);
                    cr.fill().ok();

                    // Draw underline
                    cr.set_source_rgba(0.467, 0.573, 0.702, 0.9);
                    cr.set_line_width(2.0);
                    cr.move_to(hint_x, hint_y + cell_h - 1.0);
                    cr.line_to(hint_x + hint_width, hint_y + cell_h - 1.0);
                    cr.stroke().ok();

                    // Draw label badge (like "a", "b", "aa", etc.)
                    let label = &hint.label;

                    // Label background (rounded rectangle)
                    let badge_width = (label.len() as f64 * cell_w * 0.7).max(cell_w);
                    let badge_height = cell_h * 0.9;
                    let badge_x = hint_x - 2.0;
                    let badge_y = hint_y + (cell_h - badge_height) / 2.0;

                    // Orange/yellow badge background
                    cr.set_source_rgba(0.898, 0.659, 0.294, 0.95);
                    cr.rectangle(badge_x, badge_y, badge_width, badge_height);
                    cr.fill().ok();

                    // Badge border
                    cr.set_source_rgba(0.7, 0.5, 0.2, 1.0);
                    cr.set_line_width(1.0);
                    cr.rectangle(badge_x, badge_y, badge_width, badge_height);
                    cr.stroke().ok();

                    // Label text (dark on light badge)
                    cr.set_source_rgb(0.1, 0.1, 0.1);
                    layout.set_text(label);
                    let text_x = badge_x + (badge_width - cell_w * label.len() as f64 * 0.6) / 2.0;
                    let text_y = badge_y + (badge_height - ascent) / 2.0;
                    cr.move_to(text_x, text_y);
                    pangocairo::functions::show_layout(cr, &layout);
                }

                // Show input buffer if partially typed
                if !hint_state.input_buffer.is_empty() {
                    // Draw status bar at bottom showing current input
                    let status_y = padding + (visible_rows as f64 * cell_h) + 4.0;
                    cr.set_source_rgba(0.2, 0.2, 0.2, 0.9);
                    cr.rectangle(padding, status_y, cell_w * 20.0, cell_h);
                    cr.fill().ok();

                    cr.set_source_rgb(0.9, 0.9, 0.9);
                    layout.set_text(&format!("Hint: {}", hint_state.input_buffer));
                    cr.move_to(padding + 4.0, status_y + (cell_h - ascent) / 2.0);
                    pangocairo::functions::show_layout(cr, &layout);
                }
            }
            drop(hint_state);

            // Draw visual bell flash overlay
            if *bell_flash_for_draw.borrow() {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.3);
                cr.paint().ok();
            }
        });

        // Set up resize handling with debounced PTY resize to prevent ghost lines
        // Strategy: Resize grid IMMEDIATELY (no dead space) but debounce PTY resize
        // Ghost lines happen when multiple SIGWINCH signals cause shell to redraw repeatedly
        // Solution: Only send ONE SIGWINCH after resize events stabilize
        let term_for_resize = terminal.clone();
        let pty_for_resize = pty.clone();
        let cell_width_for_resize = cell_width.clone();
        let cell_height_for_resize = cell_height.clone();
        let _drawing_area_for_resize = drawing_area.clone(); // Kept for potential future use
        let pty_cols_for_resize = pty_cols.clone();

        // Track pending PTY resize (grid resizes immediately, PTY is debounced)
        let pending_pty_resize: Rc<RefCell<Option<(usize, usize, i32, i32)>>> =
            Rc::new(RefCell::new(None));
        let pty_resize_timeout_id: Rc<RefCell<Option<glib::SourceId>>> =
            Rc::new(RefCell::new(None));

        drawing_area.connect_resize(move |area, width, height| {
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

            // IMMEDIATELY resize terminal grid for stable display
            // This ensures the grid matches the drawing area at all times
            let new_terminal_size = TerminalSize {
                rows: new_rows,
                cols: new_cols,
            };
            term_for_resize.borrow_mut().resize(new_terminal_size);

            // Update pty_cols immediately for rendering clipping
            *pty_cols_for_resize.borrow_mut() = new_cols;

            // Queue redraw immediately after grid resize
            area.queue_draw();

            // Store pending PTY resize dimensions (PTY resize is debounced to avoid SIGWINCH spam)
            *pending_pty_resize.borrow_mut() = Some((new_rows, new_cols, width, height));

            // Cancel any existing resize timeout
            if let Some(source_id) = pty_resize_timeout_id.borrow_mut().take() {
                source_id.remove();
            }

            // Clone references for the timeout closure
            let pty_for_timeout = pty_for_resize.clone();
            let pending_for_timeout = pending_pty_resize.clone();
            let timeout_id_ref = pty_resize_timeout_id.clone();

            // 100ms debounce for PTY resize only - prevents SIGWINCH storm to shell
            let source_id =
                glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    // Clear the timeout ID
                    *timeout_id_ref.borrow_mut() = None;

                    // Get the final pending dimensions for PTY
                    if let Some((rows, cols, px_width, px_height)) =
                        pending_for_timeout.borrow_mut().take()
                    {
                        tracing::debug!("Applying PTY resize to {}x{} (debounced)", rows, cols);

                        // Resize PTY (sends SIGWINCH to shell)
                        if let Some(ref mut pty) = *pty_for_timeout.borrow_mut() {
                            let new_pty_size = PtySize {
                                rows: rows as u16,
                                cols: cols as u16,
                                pixel_width: px_width as u16,
                                pixel_height: px_height as u16,
                            };
                            if let Err(e) = pty.resize(new_pty_size) {
                                tracing::error!("Failed to resize PTY: {}", e);
                            }
                        }
                    }
                });

            *pty_resize_timeout_id.borrow_mut() = Some(source_id);
        });

        // Set up keyboard input
        let key_controller = EventControllerKey::new();
        let pty_for_input = pty.clone();
        let drawing_area_for_clipboard = drawing_area.clone();
        let terminal_for_copy = terminal.clone();
        let terminal_for_key = terminal.clone();
        let scroll_offset_for_key = scroll_offset.clone();
        let hint_mode_for_key = hint_mode.clone();
        let hint_detector_for_key = hint_detector.clone();
        let drawing_area_for_hint = drawing_area.clone();
        key_controller.connect_key_pressed(move |_, key, _keycode, modifier| {
            use gtk4::gdk::{Key, ModifierType};

            let ctrl = modifier.contains(ModifierType::CONTROL_MASK);
            let shift = modifier.contains(ModifierType::SHIFT_MASK);

            // Handle hint mode keyboard interaction
            {
                let mut hint_state = hint_mode_for_key.borrow_mut();

                // If hint mode is active, handle hint input
                if hint_state.active {
                    // Escape exits hint mode
                    if matches!(key, Key::Escape) {
                        hint_state.deactivate();
                        drawing_area_for_hint.queue_draw();
                        return glib::Propagation::Stop;
                    }

                    // Letter keys for hint selection
                    if let Some(c) = key.to_unicode() {
                        if c.is_ascii_lowercase() {
                            if let Some(hint) = hint_state.handle_key(c) {
                                // Hint selected! Execute the action
                                let text = hint.text.clone();
                                let hint_type = hint.hint_type.clone();

                                // Drop borrow before action
                                drop(hint_state);

                                // Execute action based on hint type
                                match hint_type {
                                    corgiterm_core::HintType::Url => {
                                        // Open URL in browser
                                        if let Err(e) = open::that(&text) {
                                            tracing::warn!("Failed to open URL {}: {}", text, e);
                                        } else {
                                            tracing::info!("Opened URL: {}", text);
                                        }
                                    }
                                    corgiterm_core::HintType::FilePath => {
                                        // Copy path to clipboard (user can paste or cd)
                                        let clipboard = drawing_area_for_hint.clipboard();
                                        clipboard.set_text(&text);
                                        tracing::info!("Copied path to clipboard: {}", text);
                                    }
                                    corgiterm_core::HintType::Email
                                    | corgiterm_core::HintType::IpAddress
                                    | corgiterm_core::HintType::GitHash
                                    | corgiterm_core::HintType::Port => {
                                        // Copy to clipboard
                                        let clipboard = drawing_area_for_hint.clipboard();
                                        clipboard.set_text(&text);
                                        tracing::info!("Copied to clipboard: {}", text);
                                    }
                                }

                                drawing_area_for_hint.queue_draw();
                                return glib::Propagation::Stop;
                            }
                            drawing_area_for_hint.queue_draw();
                            return glib::Propagation::Stop;
                        }
                    }

                    // Any other key exits hint mode
                    hint_state.deactivate();
                    drawing_area_for_hint.queue_draw();
                    // Don't return - let the key pass through to normal handling
                }
            }

            // Ctrl+Shift+U activates hint mode (foot-style URL hints)
            if ctrl && shift && matches!(key, Key::U | Key::u) {
                let term = terminal_for_key.borrow();
                let grid = term.grid();

                // Collect visible lines as strings
                let lines: Vec<String> = grid
                    .iter()
                    .map(|row| row.iter().map(|cell| cell.content.as_str()).collect())
                    .collect();

                // Scan for hints
                let hints = hint_detector_for_key.scan(&lines);

                if !hints.is_empty() {
                    tracing::info!("Hint mode activated: {} hints found", hints.len());
                    hint_mode_for_key.borrow_mut().activate(hints);
                    drawing_area_for_hint.queue_draw();
                } else {
                    tracing::debug!("No hints found in terminal buffer");
                }

                return glib::Propagation::Stop;
            }

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

            // Check for Ctrl+Shift+A (select all)
            if ctrl && shift && matches!(key, Key::A | Key::a) {
                // Copy all content (scrollback + visible) to clipboard
                let clipboard = drawing_area_for_clipboard.clipboard();
                let term = terminal_for_copy.borrow();

                // Collect scrollback lines first
                let mut lines = Vec::new();
                for row in term.scrollback() {
                    let mut line = String::new();
                    for cell in row.iter() {
                        line.push_str(&cell.content);
                    }
                    lines.push(line.trim_end().to_string());
                }

                // Then visible grid
                for row in term.grid().iter() {
                    let mut line = String::new();
                    for cell in row.iter() {
                        line.push_str(&cell.content);
                    }
                    lines.push(line.trim_end().to_string());
                }

                // Join lines and set clipboard
                let text = lines.join("\n");
                clipboard.set_text(&text);
                tracing::info!("Copied {} lines to clipboard (select all)", lines.len());

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

            // Check for zoom shortcuts (Ctrl+Plus/Minus/0)
            if ctrl && !shift {
                // Ctrl+Plus or Ctrl+= (zoom in)
                if matches!(key, Key::plus | Key::equal | Key::KP_Add) {
                    if let Some(config_manager) = crate::app::config_manager() {
                        let current_size = config_manager.read().config().appearance.font_size;
                        let new_size = (current_size + 1.0).min(24.0);
                        config_manager.read().update(|config| {
                            config.appearance.font_size = new_size;
                        });
                        let _ = config_manager.read().save();
                        drawing_area_for_clipboard.queue_draw();
                        tracing::info!("Zoom in: font size {}", new_size);
                    }
                    return glib::Propagation::Stop;
                }
                // Ctrl+Minus (zoom out)
                if matches!(key, Key::minus | Key::KP_Subtract) {
                    if let Some(config_manager) = crate::app::config_manager() {
                        let current_size = config_manager.read().config().appearance.font_size;
                        let new_size = (current_size - 1.0).max(8.0);
                        config_manager.read().update(|config| {
                            config.appearance.font_size = new_size;
                        });
                        let _ = config_manager.read().save();
                        drawing_area_for_clipboard.queue_draw();
                        tracing::info!("Zoom out: font size {}", new_size);
                    }
                    return glib::Propagation::Stop;
                }
                // Ctrl+0 (reset zoom)
                if matches!(key, Key::_0 | Key::KP_0) {
                    if let Some(config_manager) = crate::app::config_manager() {
                        config_manager.read().update(|config| {
                            config.appearance.font_size = 11.0; // Default size
                        });
                        let _ = config_manager.read().save();
                        drawing_area_for_clipboard.queue_draw();
                        tracing::info!("Zoom reset: font size 11");
                    }
                    return glib::Propagation::Stop;
                }
            }

            if let Some(ref pty) = *pty_for_input.borrow() {
                // Convert GDK key to bytes
                let bytes = key_to_bytes(key, modifier);
                if !bytes.is_empty() {
                    // Scroll to bottom on keystroke if enabled
                    let scroll_on_keystroke = crate::app::config_manager()
                        .map(|cm| cm.read().config().terminal.scroll_on_keystroke)
                        .unwrap_or(true);
                    if scroll_on_keystroke {
                        *scroll_offset_for_key.borrow_mut() = 0;
                    }
                    let _ = pty.write(&bytes);
                }
            }
            glib::Propagation::Stop
        });
        drawing_area.add_controller(key_controller);

        // Add motion controller for URL hover tracking
        let motion_controller = EventControllerMotion::new();
        let hover_pos_for_motion = hover_pos.clone();
        let cell_width_for_motion = cell_width.clone();
        let cell_height_for_motion = cell_height.clone();
        let drawing_area_for_motion = drawing_area.clone();

        motion_controller.connect_motion(move |_, x, y| {
            let cell_w = *cell_width_for_motion.borrow();
            let cell_h = *cell_height_for_motion.borrow();
            let padding = 8.0;

            let col = ((x - padding) / cell_w).max(0.0) as usize;
            let row = ((y - padding) / cell_h).max(0.0) as usize;

            let old_pos = *hover_pos_for_motion.borrow();
            let new_pos = Some((row, col));

            if old_pos != new_pos {
                *hover_pos_for_motion.borrow_mut() = new_pos;
                drawing_area_for_motion.queue_draw();
            }
        });

        motion_controller.connect_leave(move |_| {
            // hover_pos cleared on leave handled below
        });

        let hover_pos_for_leave = hover_pos.clone();
        let drawing_area_for_leave = drawing_area.clone();
        motion_controller.connect_leave(move |_| {
            *hover_pos_for_leave.borrow_mut() = None;
            drawing_area_for_leave.queue_draw();
        });

        drawing_area.add_controller(motion_controller);

        // Add click handler to grab focus and handle URL clicks
        let click_gesture = GestureClick::new();
        let drawing_area_for_focus = drawing_area.clone();
        let cell_width_for_click = cell_width.clone();
        let cell_height_for_click = cell_height.clone();
        let detected_urls_for_click = detected_urls.clone();

        click_gesture.connect_pressed(move |gesture, _n_press, x, y| {
            drawing_area_for_focus.grab_focus();

            // Check if Ctrl is held for URL clicking
            if let Some(event) = gesture.current_event() {
                let modifier = event.modifier_state();
                if modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                    let cell_w = *cell_width_for_click.borrow();
                    let cell_h = *cell_height_for_click.borrow();
                    let padding = 8.0;

                    let col = ((x - padding) / cell_w).max(0.0) as usize;
                    let row = ((y - padding) / cell_h).max(0.0) as usize;

                    // Find URL at this position
                    let urls = detected_urls_for_click.borrow();
                    if let Some(url) = urls
                        .iter()
                        .find(|u| u.row == row && col >= u.start_col && col <= u.end_col)
                    {
                        // Open URL in default browser using xdg-open
                        tracing::info!("Opening URL: {}", url.url);
                        let url_str = url.url.clone();
                        std::thread::spawn(move || {
                            let _ = std::process::Command::new("xdg-open").arg(&url_str).spawn();
                        });
                    }
                }
            }
        });
        drawing_area.add_controller(click_gesture);

        // Right-click context menu
        let right_click_gesture = GestureClick::new();
        right_click_gesture.set_button(3); // Right mouse button
        let drawing_area_for_context = drawing_area.clone();
        let terminal_for_context = terminal.clone();
        let pty_for_context = pty.clone();
        let container_for_context = container.clone();

        right_click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
            // Create context menu
            let menu = Menu::new();
            menu.append(Some("Copy"), Some("term.copy"));
            menu.append(Some("Paste"), Some("term.paste"));
            menu.append(Some("Select All"), Some("term.select-all"));
            menu.append(Some("Find..."), Some("term.find"));

            // Create popover menu
            let popover = PopoverMenu::from_model(Some(&menu));
            popover.set_parent(&drawing_area_for_context);
            popover.set_has_arrow(true);
            popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));

            // Add actions to the drawing area's action group
            let action_group = gtk4::gio::SimpleActionGroup::new();

            // Copy action
            let copy_action = SimpleAction::new("copy", None);
            let terminal_copy = terminal_for_context.clone();
            let da_copy = drawing_area_for_context.clone();
            copy_action.connect_activate(move |_, _| {
                let clipboard = da_copy.clipboard();
                let term = terminal_copy.borrow();
                let grid = term.grid();

                let mut lines = Vec::new();
                for row in grid.iter() {
                    let mut line = String::new();
                    for cell in row.iter() {
                        line.push_str(&cell.content);
                    }
                    lines.push(line.trim_end().to_string());
                }
                let text = lines.join("\n");
                clipboard.set_text(&text);
            });
            action_group.add_action(&copy_action);

            // Paste action
            let paste_action = SimpleAction::new("paste", None);
            let pty_paste = pty_for_context.clone();
            let da_paste = drawing_area_for_context.clone();
            paste_action.connect_activate(move |_, _| {
                let clipboard = da_paste.clipboard();
                let pty_clone = pty_paste.clone();
                clipboard.read_text_async(None::<&gtk4::gio::Cancellable>, move |result| {
                    if let Ok(Some(text)) = result {
                        if let Some(ref pty) = *pty_clone.borrow() {
                            let _ = pty.write(text.as_bytes());
                        }
                    }
                });
            });
            action_group.add_action(&paste_action);

            // Select All action
            let select_all_action = SimpleAction::new("select-all", None);
            let terminal_select = terminal_for_context.clone();
            let da_select = drawing_area_for_context.clone();
            select_all_action.connect_activate(move |_, _| {
                let clipboard = da_select.clipboard();
                let term = terminal_select.borrow();

                let mut lines = Vec::new();
                for row in term.scrollback() {
                    let mut line = String::new();
                    for cell in row.iter() {
                        line.push_str(&cell.content);
                    }
                    lines.push(line.trim_end().to_string());
                }
                for row in term.grid().iter() {
                    let mut line = String::new();
                    for cell in row.iter() {
                        line.push_str(&cell.content);
                    }
                    lines.push(line.trim_end().to_string());
                }
                let text = lines.join("\n");
                clipboard.set_text(&text);
                tracing::info!("Copied {} lines to clipboard", lines.len());
            });
            action_group.add_action(&select_all_action);

            // Find action
            let find_action = SimpleAction::new("find", None);
            let container_find = container_for_context.clone();
            find_action.connect_activate(move |_, _| {
                // Find and show the search revealer
                let mut child = container_find.first_child();
                while let Some(widget) = child {
                    if let Some(revealer) = widget.downcast_ref::<Revealer>() {
                        revealer.set_reveal_child(true);
                        if let Some(entry) = revealer
                            .child()
                            .and_then(|c| c.first_child())
                            .and_then(|c| c.first_child())
                        {
                            if let Some(entry) = entry.downcast_ref::<Entry>() {
                                entry.grab_focus();
                            }
                        }
                        break;
                    }
                    child = widget.next_sibling();
                }
            });
            action_group.add_action(&find_action);

            drawing_area_for_context.insert_action_group("term", Some(&action_group));

            popover.popup();
        });
        drawing_area.add_controller(right_click_gesture);

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

        // Set up PTY reading with background thread + channel (non-blocking)
        // The old approach used fcntl with pty.master_fd() which returns -1 (portable-pty
        // doesn't expose the fd), causing blocking reads that froze the GTK main loop.
        // This new approach spawns a background thread for blocking reads and uses a
        // channel to send data to the GTK main loop.
        let term_for_read = terminal.clone();
        let drawing_area_clone = drawing_area.clone();
        let scroll_offset_for_reset = scroll_offset.clone();

        // Create channel for PTY data (background thread -> main loop)
        let (pty_data_tx, pty_data_rx) = crossbeam_channel::unbounded::<Vec<u8>>();
        let pty_data_rx = Rc::new(pty_data_rx);

        // Spawn background thread for PTY reading
        if let Some(pty_ref) = pty.borrow().as_ref() {
            // Clone the Pty's reader Arc for the background thread
            let pty_reader = pty_ref.reader_clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    // This read is blocking, but it's in a background thread so it's fine
                    match pty_reader.lock() {
                        Ok(mut reader) => {
                            match reader.read(&mut buf) {
                                Ok(0) => {
                                    // EOF - PTY closed
                                    tracing::debug!("PTY reader: EOF");
                                    break;
                                }
                                Ok(n) => {
                                    // Send data to main loop via channel
                                    if pty_data_tx.send(buf[..n].to_vec()).is_err() {
                                        // Receiver dropped, exit thread
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::debug!("PTY read error: {} (may be normal on close)", e);
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            tracing::error!("PTY reader lock poisoned");
                            break;
                        }
                    }
                }
                tracing::debug!("PTY reader thread exiting");
            });
        }

        // Poll channel for PTY data in GTK main loop (non-blocking)
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            // Try to receive data from background thread (non-blocking)
            while let Ok(data) = pty_data_rx.try_recv() {
                // Use safe_process for automatic crash recovery
                let health = term_for_read.borrow_mut().safe_process(&data);

                // Log health status changes for debugging
                match health {
                    corgiterm_core::TerminalHealth::Degraded => {
                        tracing::warn!("Terminal entered degraded mode - attempting recovery");
                    }
                    corgiterm_core::TerminalHealth::Recovered => {
                        tracing::info!("Terminal recovered from error state");
                    }
                    corgiterm_core::TerminalHealth::NeedsReset => {
                        tracing::error!("Terminal needs manual reset");
                    }
                    _ => {}
                }

                // Check scroll_on_output setting - if enabled, scroll to bottom
                let scroll_on_output = crate::app::config_manager()
                    .map(|cm| cm.read().config().terminal.scroll_on_output)
                    .unwrap_or(false);
                if scroll_on_output {
                    *scroll_offset_for_reset.borrow_mut() = 0;
                }
                drawing_area_clone.queue_draw();
            }
            glib::ControlFlow::Continue
        });

        // Set up cursor blink timer
        let cursor_visible_for_blink = cursor_visible.clone();
        let drawing_area_for_blink = drawing_area.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(530), move || {
            // Check if cursor blink is enabled
            let (blink_enabled, reduce_motion) = crate::app::config_manager()
                .map(|cm| {
                    let config = cm.read().config();
                    (
                        config.appearance.cursor_blink,
                        config.accessibility.reduce_motion,
                    )
                })
                .unwrap_or((true, false));

            // Don't blink if disabled or reduce_motion is enabled
            if blink_enabled && !reduce_motion {
                let mut visible = cursor_visible_for_blink.borrow_mut();
                *visible = !*visible;
                drawing_area_for_blink.queue_draw();
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

        // Create search bar with Revealer
        let search_bar = Box::new(Orientation::Horizontal, 8);
        search_bar.set_margin_start(8);
        search_bar.set_margin_end(8);
        search_bar.set_margin_top(4);
        search_bar.set_margin_bottom(4);
        search_bar.add_css_class("search-bar");

        let search_entry = Entry::new();
        search_entry.set_placeholder_text(Some("Search..."));
        search_entry.set_hexpand(true);
        search_bar.append(&search_entry);

        let match_count_label = Label::new(Some("0/0"));
        match_count_label.add_css_class("dim-label");
        search_bar.append(&match_count_label);

        let search_revealer = Revealer::new();
        search_revealer.set_transition_type(RevealerTransitionType::SlideDown);
        search_revealer.set_transition_duration(150);
        search_revealer.set_child(Some(&search_bar));
        search_revealer.set_reveal_child(false);

        container.append(&search_revealer);

        // Connect search entry to perform search
        let search_state_for_entry = search_state.clone();
        let terminal_for_search = terminal.clone();
        let drawing_area_for_search = drawing_area.clone();
        let match_label_for_entry = match_count_label.clone();
        search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let mut state = search_state_for_entry.borrow_mut();
            state.query = query.clone();
            state.matches.clear();
            state.current_match = 0;

            if query.is_empty() {
                state.active = false;
                match_label_for_entry.set_text("0/0");
            } else {
                state.active = true;

                // Search through terminal content
                let term = terminal_for_search.borrow();
                let grid = term.grid();

                for (row_idx, row) in grid.iter().enumerate() {
                    // Build line text
                    let line_text: String = row
                        .iter()
                        .map(|c| {
                            if c.content.is_empty() {
                                ' '
                            } else {
                                c.content.chars().next().unwrap_or(' ')
                            }
                        })
                        .collect();

                    // Find all matches in this line (case-insensitive)
                    let lower_line = line_text.to_lowercase();
                    let lower_query = query.to_lowercase();
                    let mut start = 0;
                    while let Some(pos) = lower_line[start..].find(&lower_query) {
                        let actual_pos = start + pos;
                        state
                            .matches
                            .push((row_idx, actual_pos, actual_pos + query.len()));
                        start = actual_pos + 1;
                    }
                }

                // Update label
                if state.matches.is_empty() {
                    match_label_for_entry.set_text("0/0");
                } else {
                    match_label_for_entry.set_text(&format!("1/{}", state.matches.len()));
                }
            }

            drawing_area_for_search.queue_draw();
        });

        // Handle Enter to go to next match, Shift+Enter for previous
        let search_state_for_nav = search_state.clone();
        let drawing_area_for_nav = drawing_area.clone();
        let match_label_for_nav = match_count_label.clone();
        search_entry.connect_activate(move |_| {
            let mut state = search_state_for_nav.borrow_mut();
            if !state.matches.is_empty() {
                state.current_match = (state.current_match + 1) % state.matches.len();
                match_label_for_nav.set_text(&format!(
                    "{}/{}",
                    state.current_match + 1,
                    state.matches.len()
                ));
            }
            drop(state);
            drawing_area_for_nav.queue_draw();
        });

        // Handle Escape to close search bar
        let search_revealer_for_escape = search_revealer.clone();
        let search_state_for_escape = search_state.clone();
        let drawing_area_for_escape = drawing_area.clone();
        let search_key_controller = EventControllerKey::new();
        search_key_controller.connect_key_pressed(move |_, key, _, _| {
            use gtk4::gdk::Key;
            if matches!(key, Key::Escape) {
                search_revealer_for_escape.set_reveal_child(false);
                let mut state = search_state_for_escape.borrow_mut();
                state.active = false;
                state.matches.clear();
                drop(state);
                drawing_area_for_escape.queue_draw();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        search_entry.add_controller(search_key_controller);

        // Wrap revealer in Rc for access from key handler
        let search_revealer_rc = Rc::new(search_revealer.clone());
        let search_entry_rc = Rc::new(search_entry.clone());

        // Add Ctrl+Shift+F handler to container
        let search_key_controller2 = EventControllerKey::new();
        let search_revealer_for_toggle = search_revealer_rc.clone();
        let search_entry_for_focus = search_entry_rc.clone();
        search_key_controller2.connect_key_pressed(move |_, key, _, modifier| {
            use gtk4::gdk::{Key, ModifierType};

            let ctrl = modifier.contains(ModifierType::CONTROL_MASK);
            let shift = modifier.contains(ModifierType::SHIFT_MASK);

            if ctrl && shift && matches!(key, Key::F | Key::f) {
                let is_revealed = search_revealer_for_toggle.reveals_child();
                search_revealer_for_toggle.set_reveal_child(!is_revealed);
                if !is_revealed {
                    search_entry_for_focus.grab_focus();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        container.add_controller(search_key_controller2);

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

        // Add gesture drag for text selection
        let drag_gesture = GestureDrag::new();
        drag_gesture.set_button(1); // Left mouse button

        let selection_for_drag_start = selection.clone();
        let cell_width_for_drag = cell_width.clone();
        let cell_height_for_drag = cell_height.clone();
        let drawing_area_for_drag = drawing_area.clone();

        drag_gesture.connect_drag_begin(move |_, start_x, start_y| {
            let cell_w = *cell_width_for_drag.borrow();
            let cell_h = *cell_height_for_drag.borrow();
            let padding = 8.0;

            // Convert pixel coords to cell coords
            let col = ((start_x - padding) / cell_w).max(0.0) as usize;
            let row = ((start_y - padding) / cell_h).max(0.0) as usize;

            let mut sel = selection_for_drag_start.borrow_mut();
            sel.active = true;
            sel.start = (row, col);
            sel.end = (row, col);

            drawing_area_for_drag.queue_draw();
        });

        let selection_for_drag_update = selection.clone();
        let cell_width_for_update = cell_width.clone();
        let cell_height_for_update = cell_height.clone();
        let drawing_area_for_update = drawing_area.clone();

        drag_gesture.connect_drag_update(move |gesture, offset_x, offset_y| {
            if let Some((start_x, start_y)) = gesture.start_point() {
                let cell_w = *cell_width_for_update.borrow();
                let cell_h = *cell_height_for_update.borrow();
                let padding = 8.0;

                let end_x = start_x + offset_x;
                let end_y = start_y + offset_y;

                let col = ((end_x - padding) / cell_w).max(0.0) as usize;
                let row = ((end_y - padding) / cell_h).max(0.0) as usize;

                let mut sel = selection_for_drag_update.borrow_mut();
                sel.end = (row, col);

                drawing_area_for_update.queue_draw();
            }
        });

        let selection_for_drag_end = selection.clone();
        let terminal_for_copy_sel = terminal.clone();
        let drawing_area_for_end = drawing_area.clone();

        drag_gesture.connect_drag_end(move |_, _, _| {
            let sel = selection_for_drag_end.borrow();

            // Check if copy_on_select is enabled
            let copy_on_select = crate::app::config_manager()
                .map(|cm| cm.read().config().terminal.copy_on_select)
                .unwrap_or(false);

            // Copy selected text to clipboard if selection is valid and copy_on_select is enabled
            if copy_on_select && sel.active && sel.start != sel.end {
                let term = terminal_for_copy_sel.borrow();
                let grid = term.grid();

                let (start_row, start_col, end_row, end_col) =
                    normalize_selection(sel.start.0, sel.start.1, sel.end.0, sel.end.1);

                let mut text = String::new();
                for row in start_row..=end_row {
                    if row >= grid.len() {
                        break;
                    }

                    let col_start = if row == start_row { start_col } else { 0 };
                    let col_end = if row == end_row {
                        end_col + 1
                    } else {
                        grid[row].len()
                    };

                    for col in col_start..col_end.min(grid[row].len()) {
                        text.push_str(&grid[row][col].content);
                    }

                    if row < end_row {
                        text.push('\n');
                    }
                }

                // Set clipboard
                let clipboard = drawing_area_for_end.clipboard();
                clipboard.set_text(text.trim_end());
                tracing::debug!("Copied selection to clipboard (copy_on_select)");
            }
        });

        drawing_area.add_controller(drag_gesture);

        // Load theme colors
        let colors = Rc::new(RefCell::new(load_theme_colors()));

        Self {
            container,
            drawing_area,
            terminal,
            pty,
            cell_width,
            cell_height,
            scroll_offset,
            scrollbar_adj,
            selection,
            hover_pos,
            detected_urls,
            search_state,
            event_rx,
            cursor_visible,
            bell_flash,
            colors,
            pty_cols,
            hint_mode,
            hint_detector,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Queue a redraw
    pub fn queue_draw(&self) {
        self.drawing_area.queue_draw();
    }

    /// Get the terminal event receiver for listening to title changes, bells, etc.
    pub fn event_receiver(
        &self,
    ) -> Rc<crossbeam_channel::Receiver<corgiterm_core::terminal::TerminalEvent>> {
        self.event_rx.clone()
    }

    /// Trigger a visual bell flash
    pub fn trigger_visual_bell(&self) {
        let bell_flash = self.bell_flash.clone();
        let drawing_area = self.drawing_area.clone();

        // Set flash on
        *bell_flash.borrow_mut() = true;
        drawing_area.queue_draw();

        // Turn off after 100ms
        glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            *bell_flash.borrow_mut() = false;
            drawing_area.queue_draw();
        });
    }

    /// Get reference to bell flash state (for external bell triggering)
    pub fn bell_flash_ref(&self) -> Rc<RefCell<bool>> {
        self.bell_flash.clone()
    }

    /// Get reference to drawing area (for external redraw)
    pub fn drawing_area_ref(&self) -> DrawingArea {
        self.drawing_area.clone()
    }

    /// Send a command to the terminal (appends newline)
    pub fn send_command(&self, command: &str) {
        if let Some(ref pty) = *self.pty.borrow() {
            // Record command for AI learning
            let directory = self
                .working_directory()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| {
                    std::env::current_dir()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                });
            crate::app::record_command(command.to_string(), directory, None);

            // Send command with newline
            let cmd_with_newline = format!("{}\n", command);
            if let Err(e) = pty.write(cmd_with_newline.as_bytes()) {
                tracing::error!("Failed to send command to PTY: {}", e);
            } else {
                tracing::debug!("Sent command: {}", command);
            }
        }
    }

    /// Send raw bytes to the terminal (no newline added)
    pub fn send_bytes(&self, bytes: &[u8]) {
        if let Some(ref pty) = *self.pty.borrow() {
            if let Err(e) = pty.write(bytes) {
                tracing::error!("Failed to send bytes to PTY: {}", e);
            }
        }
    }

    /// Get current working directory from terminal (if available)
    pub fn working_directory(&self) -> Option<std::path::PathBuf> {
        if let Some(ref pty) = *self.pty.borrow() {
            // Try to get foreground process group first (the actual running command)
            // If that fails, fall back to the shell PID
            let pid = pty.foreground_pid().unwrap_or_else(|| pty.pid());

            // Read /proc/<pid>/cwd symlink
            let proc_cwd = format!("/proc/{}/cwd", pid);
            match std::fs::read_link(&proc_cwd) {
                Ok(path) => Some(path),
                Err(e) => {
                    tracing::debug!("Failed to read {}: {}", proc_cwd, e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Get the current directory name for display (just the last component)
    pub fn current_directory_name(&self) -> String {
        self.working_directory()
            .and_then(|path| {
                // Try to get just the directory name
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "Terminal".to_string())
    }

    /// Get visible lines as strings for thumbnail rendering
    /// Returns up to `max_lines` from the terminal grid
    pub fn get_visible_lines(&self, max_lines: usize) -> Vec<String> {
        let terminal = self.terminal.borrow();
        let grid = terminal.grid();

        let mut lines = Vec::new();
        let start = if grid.len() > max_lines {
            grid.len() - max_lines
        } else {
            0
        };

        for row in grid.iter().skip(start).take(max_lines) {
            let mut line = String::new();
            for cell in row.iter() {
                if cell.content.is_empty() {
                    line.push(' ');
                } else {
                    line.push_str(&cell.content);
                }
            }
            lines.push(line.trim_end().to_string());
        }
        lines
    }

    /// Get terminal dimensions (columns, rows)
    pub fn terminal_size(&self) -> (usize, usize) {
        let terminal = self.terminal.borrow();
        let size = terminal.size();
        (size.cols, size.rows)
    }
}

impl Default for TerminalView {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize selection to ensure start is before end
fn normalize_selection(
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
) -> (usize, usize, usize, usize) {
    if start_row < end_row || (start_row == end_row && start_col <= end_col) {
        (start_row, start_col, end_row, end_col)
    } else {
        (end_row, end_col, start_row, start_col)
    }
}

/// Check if a cell is within the selection
fn is_cell_selected(row: usize, col: usize, sel: &Selection) -> bool {
    if !sel.active {
        return false;
    }

    let (start_row, start_col, end_row, end_col) =
        normalize_selection(sel.start.0, sel.start.1, sel.end.0, sel.end.1);

    if row < start_row || row > end_row {
        return false;
    }

    if row == start_row && row == end_row {
        // Single line selection
        col >= start_col && col <= end_col
    } else if row == start_row {
        // First line of multi-line selection
        col >= start_col
    } else if row == end_row {
        // Last line of multi-line selection
        col <= end_col
    } else {
        // Middle lines are fully selected
        true
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
