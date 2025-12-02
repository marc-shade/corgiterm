//! Terminal emulation using VTE
//!
//! Provides VT100/xterm compatible terminal emulation with modern features.
//! Includes health monitoring and crash recovery for maximum stability.

use vte::{Params, Parser, Perform};

/// Health status of the terminal - inspired by foot terminal's stability focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalHealth {
    /// Terminal is functioning normally
    Healthy,
    /// Terminal recovered from an error
    Recovered,
    /// Terminal is in degraded mode (some features may be limited)
    Degraded,
    /// Terminal needs manual reset to recover
    NeedsReset,
}

/// Terminal dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub rows: usize,
    pub cols: usize,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { rows: 24, cols: 80 }
    }
}

/// Events emitted by the terminal
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Terminal title changed
    TitleChanged(String),
    /// Bell/alert
    Bell,
    /// Clipboard operation
    Clipboard(ClipboardAction),
    /// Color scheme changed
    ColorChanged,
    /// Cursor position changed
    CursorMoved { row: usize, col: usize },
    /// Screen content changed
    Redraw,
}

/// Clipboard actions from terminal
#[derive(Debug, Clone)]
pub enum ClipboardAction {
    Copy(String),
    Paste,
}

/// Default foreground color (warm white)
pub const DEFAULT_FG: [u8; 4] = [232, 219, 196, 255];
/// Default background color (dark brown)
pub const DEFAULT_BG: [u8; 4] = [30, 27, 22, 255];

/// A single cell in the terminal grid
#[derive(Debug, Clone)]
pub struct Cell {
    /// The character (may be multi-codepoint for emoji)
    pub content: String,
    /// Foreground color (RGBA)
    pub fg: [u8; 4],
    /// Background color (RGBA)
    pub bg: [u8; 4],
    /// Cell attributes
    pub attrs: CellAttributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: String::new(),
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            attrs: CellAttributes::default(),
        }
    }
}

/// Cell display attributes
#[derive(Debug, Clone, Copy, Default)]
pub struct CellAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub blink: bool,
}

/// ANSI 16-color palette (Corgi Dark theme)
const ANSI_COLORS: [[u8; 4]; 16] = [
    [30, 27, 22, 255],    // 0: Black (background)
    [204, 87, 82, 255],   // 1: Red
    [146, 180, 114, 255], // 2: Green
    [229, 168, 75, 255],  // 3: Yellow
    [119, 146, 179, 255], // 4: Blue
    [177, 126, 160, 255], // 5: Magenta
    [135, 172, 175, 255], // 6: Cyan
    [232, 219, 196, 255], // 7: White (foreground)
    [100, 95, 88, 255],   // 8: Bright Black
    [229, 127, 119, 255], // 9: Bright Red
    [182, 209, 152, 255], // 10: Bright Green
    [242, 202, 122, 255], // 11: Bright Yellow
    [160, 183, 212, 255], // 12: Bright Blue
    [210, 166, 198, 255], // 13: Bright Magenta
    [171, 206, 208, 255], // 14: Bright Cyan
    [247, 241, 232, 255], // 15: Bright White
];

/// Terminal state (separate from parser to avoid borrow issues)
struct TerminalState {
    /// Terminal grid (rows of cells)
    grid: Vec<Vec<Cell>>,
    /// Current size
    size: TerminalSize,
    /// Cursor position
    cursor: (usize, usize),
    /// Terminal title
    title: String,
    /// Scrollback buffer
    scrollback: Vec<Vec<Cell>>,
    /// Maximum scrollback lines
    max_scrollback: usize,
    /// Pending events
    pending_events: Vec<TerminalEvent>,
    /// Current text attributes
    current_attrs: CellAttributes,
    /// Current foreground color
    current_fg: [u8; 4],
    /// Current background color
    current_bg: [u8; 4],
    /// Health status for stability monitoring
    health: TerminalHealth,
    /// Error count for recovery decisions
    error_count: u32,
    /// Maximum errors before forced reset
    max_errors: u32,
}

impl TerminalState {
    fn new(size: TerminalSize) -> Self {
        let grid = vec![vec![Cell::default(); size.cols]; size.rows];
        Self {
            grid,
            size,
            cursor: (0, 0),
            title: String::new(),
            scrollback: Vec::new(),
            max_scrollback: 10000,
            pending_events: Vec::new(),
            current_attrs: CellAttributes::default(),
            current_fg: DEFAULT_FG,
            current_bg: DEFAULT_BG,
            health: TerminalHealth::Healthy,
            error_count: 0,
            max_errors: 10, // Reset after 10 consecutive errors
        }
    }

    /// Validate terminal state is consistent
    fn validate_state(&self) -> bool {
        // Check cursor is within bounds
        if self.cursor.0 >= self.size.rows || self.cursor.1 >= self.size.cols {
            return false;
        }

        // Check grid dimensions
        if self.grid.len() != self.size.rows {
            return false;
        }

        for row in &self.grid {
            if row.len() != self.size.cols {
                return false;
            }
        }

        true
    }

    /// Soft reset - reset graphics state but preserve content
    /// Similar to DECSTR (Soft Terminal Reset)
    fn soft_reset(&mut self) {
        self.current_attrs = CellAttributes::default();
        self.current_fg = DEFAULT_FG;
        self.current_bg = DEFAULT_BG;

        // Clamp cursor to valid bounds
        self.cursor.0 = self.cursor.0.min(self.size.rows.saturating_sub(1));
        self.cursor.1 = self.cursor.1.min(self.size.cols.saturating_sub(1));
    }

    /// Hard reset - clear everything and start fresh
    /// Similar to RIS (Reset to Initial State)
    fn hard_reset(&mut self) {
        self.grid = vec![vec![Cell::default(); self.size.cols]; self.size.rows];
        self.cursor = (0, 0);
        self.current_attrs = CellAttributes::default();
        self.current_fg = DEFAULT_FG;
        self.current_bg = DEFAULT_BG;
        self.pending_events.clear();
        self.error_count = 0;
        self.health = TerminalHealth::Healthy;
    }

    fn put_char(&mut self, c: char) {
        if self.cursor.1 >= self.size.cols {
            self.newline();
        }

        if self.cursor.0 < self.size.rows && self.cursor.1 < self.size.cols {
            let cell = &mut self.grid[self.cursor.0][self.cursor.1];
            cell.content = c.to_string();
            cell.fg = self.current_fg;
            cell.bg = self.current_bg;
            cell.attrs = self.current_attrs;
            self.cursor.1 += 1;
        }
    }

    /// Convert 256-color index to RGBA
    fn color_from_256(idx: u8) -> [u8; 4] {
        if idx < 16 {
            ANSI_COLORS[idx as usize]
        } else if idx < 232 {
            // 216 color cube (6x6x6)
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let to_255 = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            [to_255(r), to_255(g), to_255(b), 255]
        } else {
            // 24 grayscale levels
            let level = 8 + (idx - 232) * 10;
            [level, level, level, 255]
        }
    }

    /// Apply SGR (Select Graphic Rendition) parameters
    fn apply_sgr(&mut self, params: &[u16]) {
        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => {
                    // Reset
                    self.current_attrs = CellAttributes::default();
                    self.current_fg = DEFAULT_FG;
                    self.current_bg = DEFAULT_BG;
                }
                1 => self.current_attrs.bold = true,
                2 => self.current_attrs.dim = true,
                3 => self.current_attrs.italic = true,
                4 => self.current_attrs.underline = true,
                5 | 6 => self.current_attrs.blink = true,
                7 => self.current_attrs.inverse = true,
                8 => self.current_attrs.hidden = true,
                9 => self.current_attrs.strikethrough = true,
                21 | 22 => {
                    self.current_attrs.bold = false;
                    self.current_attrs.dim = false;
                }
                23 => self.current_attrs.italic = false,
                24 => self.current_attrs.underline = false,
                25 => self.current_attrs.blink = false,
                27 => self.current_attrs.inverse = false,
                28 => self.current_attrs.hidden = false,
                29 => self.current_attrs.strikethrough = false,
                // Foreground colors 30-37
                30..=37 => {
                    self.current_fg = ANSI_COLORS[(params[i] - 30) as usize];
                }
                // Default foreground
                39 => self.current_fg = DEFAULT_FG,
                // Background colors 40-47
                40..=47 => {
                    self.current_bg = ANSI_COLORS[(params[i] - 40) as usize];
                }
                // Default background
                49 => self.current_bg = DEFAULT_BG,
                // Bright foreground colors 90-97
                90..=97 => {
                    self.current_fg = ANSI_COLORS[(params[i] - 90 + 8) as usize];
                }
                // Bright background colors 100-107
                100..=107 => {
                    self.current_bg = ANSI_COLORS[(params[i] - 100 + 8) as usize];
                }
                // Extended foreground (256 or RGB)
                38 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        // 256 color mode: 38;5;N
                        self.current_fg = Self::color_from_256(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        // RGB mode: 38;2;R;G;B
                        self.current_fg = [
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                            255,
                        ];
                        i += 4;
                    }
                }
                // Extended background (256 or RGB)
                48 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        // 256 color mode: 48;5;N
                        self.current_bg = Self::color_from_256(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        // RGB mode: 48;2;R;G;B
                        self.current_bg = [
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                            255,
                        ];
                        i += 4;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn newline(&mut self) {
        self.cursor.1 = 0;
        if self.cursor.0 + 1 >= self.size.rows {
            self.scroll_up();
        } else {
            self.cursor.0 += 1;
        }
    }

    fn scroll_up(&mut self) {
        if !self.grid.is_empty() {
            let line = self.grid.remove(0);
            self.scrollback.push(line);

            while self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }

            self.grid.push(vec![Cell::default(); self.size.cols]);
        }
    }
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
            }
            0x09 => {
                let next_tab = (self.cursor.1 / 8 + 1) * 8;
                self.cursor.1 = next_tab.min(self.size.cols - 1);
            }
            0x0A..=0x0C => {
                self.newline();
            }
            0x0D => {
                self.cursor.1 = 0;
            }
            0x07 => {
                self.pending_events.push(TerminalEvent::Bell);
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() >= 2 {
            match params[0] {
                b"0" | b"2" => {
                    if let Ok(title) = std::str::from_utf8(params[1]) {
                        self.title = title.to_string();
                        self.pending_events
                            .push(TerminalEvent::TitleChanged(title.to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        let params: Vec<u16> = params.iter().map(|p| p[0]).collect();

        match c {
            'A' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor.0 = self.cursor.0.saturating_sub(n);
            }
            'B' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor.0 = (self.cursor.0 + n).min(self.size.rows - 1);
            }
            'C' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor.1 = (self.cursor.1 + n).min(self.size.cols - 1);
            }
            'D' => {
                let n = params.first().copied().unwrap_or(1) as usize;
                self.cursor.1 = self.cursor.1.saturating_sub(n);
            }
            'H' | 'f' => {
                let row = params.first().copied().unwrap_or(1).saturating_sub(1) as usize;
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1) as usize;
                self.cursor = (row.min(self.size.rows - 1), col.min(self.size.cols - 1));
            }
            'J' => {
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        for col in self.cursor.1..self.size.cols {
                            self.grid[self.cursor.0][col] = Cell::default();
                        }
                        for row in (self.cursor.0 + 1)..self.size.rows {
                            for col in 0..self.size.cols {
                                self.grid[row][col] = Cell::default();
                            }
                        }
                    }
                    1 => {
                        for row in 0..self.cursor.0 {
                            for col in 0..self.size.cols {
                                self.grid[row][col] = Cell::default();
                            }
                        }
                        for col in 0..=self.cursor.1 {
                            self.grid[self.cursor.0][col] = Cell::default();
                        }
                    }
                    2 | 3 => {
                        for row in 0..self.size.rows {
                            for col in 0..self.size.cols {
                                self.grid[row][col] = Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = params.first().copied().unwrap_or(0);
                let row = self.cursor.0;
                match mode {
                    0 => {
                        for col in self.cursor.1..self.size.cols {
                            self.grid[row][col] = Cell::default();
                        }
                    }
                    1 => {
                        for col in 0..=self.cursor.1 {
                            self.grid[row][col] = Cell::default();
                        }
                    }
                    2 => {
                        for col in 0..self.size.cols {
                            self.grid[row][col] = Cell::default();
                        }
                    }
                    _ => {}
                }
            }
            // SGR - Select Graphic Rendition (colors and attributes)
            'm' => {
                let params_vec: Vec<u16> = if params.is_empty() {
                    vec![0] // Default to reset
                } else {
                    params.clone()
                };
                self.apply_sgr(&params_vec);
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

/// Terminal state and emulation
pub struct Terminal {
    /// Internal state
    state: TerminalState,
    /// VTE parser
    parser: Parser,
    /// Event sender
    events: crossbeam_channel::Sender<TerminalEvent>,
}

impl Terminal {
    /// Create a new terminal with the given size
    pub fn new(size: TerminalSize, events: crossbeam_channel::Sender<TerminalEvent>) -> Self {
        Self {
            state: TerminalState::new(size),
            parser: Parser::new(),
            events,
        }
    }

    /// Process input bytes from PTY
    pub fn process(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.parser.advance(&mut self.state, *byte);
        }

        // Send any pending events
        for event in self.state.pending_events.drain(..) {
            let _ = self.events.send(event);
        }

        let _ = self.events.send(TerminalEvent::Redraw);
    }

    /// Get current size
    pub fn size(&self) -> TerminalSize {
        self.state.size
    }

    /// Resize terminal
    ///
    /// Resize that preserves content by reflowing text (wrapping) and handling scrollback.
    /// Implements Alacritty-like behavior: cursor stays with content, excess lines move to scrollback.
    pub fn resize(&mut self, size: TerminalSize) {
        let old_cols = self.state.size.cols;
        let new_cols = size.cols;
        let new_rows = size.rows;

        // If size hasn't changed, do nothing
        if old_cols == new_cols && self.state.size.rows == new_rows {
            return;
        }

        let old_cursor = self.state.cursor;
        let mut new_cursor = old_cursor;
        let mut new_grid = Vec::new();

        // 1. Horizontal Reflow
        // Rebuild the grid by wrapping lines that are too long
        for (r, mut row) in self.state.grid.drain(..).enumerate() {
            // If shrinking width, we might need to split this row
            if new_cols < old_cols {
                let mut chunk_start = 0;
                let row_len = row.len();

                while chunk_start < row_len {
                    let chunk_end = (chunk_start + new_cols).min(row_len);
                    let chunk_len = chunk_end - chunk_start;

                    // Extract chunk
                    let mut chunk: Vec<Cell> = row.drain(0..chunk_len).collect();

                    // Check if this chunk should be kept
                    // Keep if:
                    // 1. It's the first chunk (base line)
                    // 2. It has non-empty content
                    // 3. The cursor is inside it
                    let is_base_line = chunk_start == 0;
                    let has_content = chunk
                        .iter()
                        .any(|c| !c.content.is_empty() && c.content != " ");
                    let cursor_in_chunk = r == old_cursor.0
                        && old_cursor.1 >= chunk_start
                        && old_cursor.1 < chunk_end;

                    if is_base_line || has_content || cursor_in_chunk {
                        // Pad chunk to new width if necessary
                        chunk.resize(new_cols, Cell::default());

                        // Update cursor if it's in this chunk
                        if cursor_in_chunk {
                            new_cursor.0 = new_grid.len();
                            new_cursor.1 = old_cursor.1 - chunk_start;
                        }

                        new_grid.push(chunk);
                    }

                    chunk_start += chunk_len;

                    // Optimization: if remaining row is empty and no cursor, stop processing this row
                    // (This prevents generating tons of empty wrapped lines)
                    let remaining_has_content = row
                        .iter()
                        .any(|c| !c.content.is_empty() && c.content != " ");
                    let cursor_in_remaining = r == old_cursor.0 && old_cursor.1 >= chunk_start;

                    if !remaining_has_content && !cursor_in_remaining {
                        break;
                    }
                }
            } else {
                // Expanding or same width: just resize the row
                row.resize(new_cols, Cell::default());
                if r == old_cursor.0 {
                    new_cursor.0 = new_grid.len();
                }
                new_grid.push(row);
            }
        }

        self.state.grid = new_grid;
        self.state.cursor = new_cursor;

        // 2. Vertical Resize (Handle Scrollback)
        let current_rows = self.state.grid.len();

        if current_rows > new_rows {
            // Too many rows (due to wrap or shrink). Move top to scrollback.
            let excess = current_rows - new_rows;
            for _ in 0..excess {
                if !self.state.grid.is_empty() {
                    let line = self.state.grid.remove(0);
                    self.state.scrollback.push(line);
                }
            }
            // Trim scrollback
            while self.state.scrollback.len() > self.state.max_scrollback {
                self.state.scrollback.remove(0);
            }

            // Adjust cursor: It effectively moved up 'excess' times relative to viewport
            self.state.cursor.0 = self.state.cursor.0.saturating_sub(excess);
        } else if current_rows < new_rows {
            // Too few rows. Pull from scrollback.
            let needed = new_rows - current_rows;
            let available = self.state.scrollback.len();
            let to_pull = needed.min(available);

            // Pull from scrollback (pop from end = most recent)
            // We need to insert them at the TOP of the grid
            // Note: These pulled lines might need horizontal resize too!
            for _ in 0..to_pull {
                if let Some(mut line) = self.state.scrollback.pop() {
                    line.resize(new_cols, Cell::default());
                    self.state.grid.insert(0, line);
                }
            }

            // If we pulled lines, the cursor (which tracks grid content) moves DOWN
            self.state.cursor.0 += to_pull;

            // If still need rows, append blank ones
            let remaining = needed - to_pull;
            for _ in 0..remaining {
                self.state.grid.push(vec![Cell::default(); new_cols]);
            }
        }

        self.state.size = size;

        // Final Clamp
        self.state.cursor.0 = self.state.cursor.0.min(new_rows.saturating_sub(1));
        self.state.cursor.1 = self.state.cursor.1.min(new_cols.saturating_sub(1));
    }

    /// Clear all content in the visible grid (not scrollback).
    /// Useful before resize to remove stale content that would cause ghost lines.
    pub fn clear_screen(&mut self) {
        let cols = self.state.size.cols;
        for row in &mut self.state.grid {
            *row = vec![Cell::default(); cols];
        }
        // Reset cursor to top-left
        self.state.cursor = (0, 0);
    }

    /// Get cell at position
    pub fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.state.grid.get(row).and_then(|r| r.get(col))
    }

    /// Get cursor position
    pub fn cursor(&self) -> (usize, usize) {
        self.state.cursor
    }

    /// Get terminal title
    pub fn title(&self) -> &str {
        &self.state.title
    }

    /// Get grid for rendering
    pub fn grid(&self) -> &[Vec<Cell>] {
        &self.state.grid
    }

    /// Get scrollback buffer
    pub fn scrollback(&self) -> &[Vec<Cell>] {
        &self.state.scrollback
    }

    /// Set maximum scrollback lines
    pub fn set_max_scrollback(&mut self, lines: usize) {
        self.state.max_scrollback = lines;
        // Trim if needed
        while self.state.scrollback.len() > self.state.max_scrollback {
            self.state.scrollback.remove(0);
        }
    }

    // ==================== Health Monitoring & Crash Recovery ====================
    // Inspired by foot terminal's stability focus

    /// Process bytes with automatic recovery on errors
    /// Returns the health status after processing
    pub fn safe_process(&mut self, bytes: &[u8]) -> TerminalHealth {
        // First validate state
        if !self.state.validate_state() {
            self.state.error_count += 1;

            if self.state.error_count >= self.state.max_errors {
                // Too many errors, perform hard reset
                self.hard_reset();
                self.state.health = TerminalHealth::Recovered;
            } else {
                // Try soft reset first
                self.soft_reset();
                self.state.health = TerminalHealth::Degraded;
            }
        }

        // Process bytes (VTE parser handles invalid sequences gracefully)
        self.process(bytes);

        // Clear error count on successful processing
        if self.state.error_count > 0 {
            self.state.error_count = 0;
            if self.state.health == TerminalHealth::Degraded {
                self.state.health = TerminalHealth::Recovered;
            }
        }

        self.state.health
    }

    /// Get current health status
    pub fn health(&self) -> TerminalHealth {
        self.state.health
    }

    /// Check if terminal needs user intervention
    pub fn needs_reset(&self) -> bool {
        self.state.health == TerminalHealth::NeedsReset
    }

    /// Get error count (for diagnostics)
    pub fn error_count(&self) -> u32 {
        self.state.error_count
    }

    /// Soft reset - reset graphics state but preserve content
    /// Similar to DECSTR (Soft Terminal Reset)
    pub fn soft_reset(&mut self) {
        self.state.soft_reset();
    }

    /// Hard reset - clear everything and start fresh
    /// Similar to RIS (Reset to Initial State)
    pub fn hard_reset(&mut self) {
        self.state.hard_reset();
        self.parser = Parser::new();
    }

    /// Validate terminal state is consistent
    pub fn validate_state(&self) -> bool {
        self.state.validate_state()
    }

    /// Mark terminal as needing attention (e.g., after repeated PTY errors)
    pub fn set_degraded(&mut self) {
        self.state.health = TerminalHealth::Degraded;
    }

    /// Force mark terminal as needing reset (severe error)
    pub fn set_needs_reset(&mut self) {
        self.state.health = TerminalHealth::NeedsReset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let term = Terminal::new(TerminalSize::default(), tx);
        assert_eq!(term.size().rows, 24);
        assert_eq!(term.size().cols, 80);
    }

    #[test]
    fn test_process_text() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let mut term = Terminal::new(TerminalSize::default(), tx);
        term.process(b"Hello");
        assert_eq!(term.state.grid[0][0].content, "H");
        assert_eq!(term.state.grid[0][4].content, "o");
    }

    #[test]
    fn test_health_monitoring() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let term = Terminal::new(TerminalSize::default(), tx);
        assert_eq!(term.health(), TerminalHealth::Healthy);
        assert!(!term.needs_reset());
        assert_eq!(term.error_count(), 0);
    }

    #[test]
    fn test_safe_process() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let mut term = Terminal::new(TerminalSize::default(), tx);

        // Safe process should return Healthy on normal input
        let health = term.safe_process(b"Normal text");
        assert!(health == TerminalHealth::Healthy || health == TerminalHealth::Recovered);
    }

    #[test]
    fn test_soft_reset() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let mut term = Terminal::new(TerminalSize::default(), tx);

        // Write some text with colors
        term.process(b"\x1b[31mRed text");

        // Soft reset should preserve content but reset graphics
        term.soft_reset();

        // Text should still be there
        assert_eq!(term.state.grid[0][0].content, "R");
    }

    #[test]
    fn test_hard_reset() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let mut term = Terminal::new(TerminalSize::default(), tx);

        // Write some text
        term.process(b"Hello, World!");

        // Hard reset should clear everything
        term.hard_reset();

        // Grid should be empty
        assert_eq!(term.state.grid[0][0].content, "");
        assert_eq!(term.cursor(), (0, 0));
        assert_eq!(term.health(), TerminalHealth::Healthy);
    }

    #[test]
    fn test_validate_state() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let term = Terminal::new(TerminalSize::default(), tx);
        assert!(term.validate_state());
    }

    #[test]
    fn test_degraded_mode() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let mut term = Terminal::new(TerminalSize::default(), tx);

        term.set_degraded();
        assert_eq!(term.health(), TerminalHealth::Degraded);

        term.set_needs_reset();
        assert_eq!(term.health(), TerminalHealth::NeedsReset);
        assert!(term.needs_reset());
    }
}
