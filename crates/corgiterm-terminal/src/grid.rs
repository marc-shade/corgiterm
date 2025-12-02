//! Terminal grid - the 2D array of cells
//!
//! The grid manages all terminal state:
//! - Cell contents
//! - Cursor position and style
//! - Scrollback buffer
//! - Damage tracking for efficient rendering

use crate::cell::{Cell, CellFlags, Color};

/// Cursor shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    /// Block cursor (filled rectangle)
    #[default]
    Block,
    /// Underline cursor
    Underline,
    /// Vertical bar cursor
    Beam,
    /// Hidden cursor
    Hidden,
}

/// Cursor state
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    /// Column position (0-indexed)
    pub col: usize,
    /// Row position (0-indexed)
    pub row: usize,
    /// Cursor shape
    pub shape: CursorShape,
    /// Whether cursor is visible
    pub visible: bool,
    /// Whether cursor is blinking
    pub blinking: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            shape: CursorShape::Block,
            visible: true,
            blinking: true,
        }
    }
}

/// A single row in the terminal grid
#[derive(Debug, Clone)]
pub struct GridRow {
    /// Cells in this row
    cells: Vec<Cell>,
    /// Whether this row has been modified (for damage tracking)
    dirty: bool,
    /// Whether this row is wrapped from the previous line
    wrapped: bool,
}

impl GridRow {
    /// Create a new row with the given width
    pub fn new(cols: usize) -> Self {
        Self {
            cells: vec![Cell::default(); cols],
            dirty: true,
            wrapped: false,
        }
    }

    /// Get cell at column
    pub fn cell(&self, col: usize) -> Option<&Cell> {
        self.cells.get(col)
    }

    /// Get mutable cell at column
    pub fn cell_mut(&mut self, col: usize) -> Option<&mut Cell> {
        self.dirty = true;
        self.cells.get_mut(col)
    }

    /// Set cell at column
    pub fn set_cell(&mut self, col: usize, cell: Cell) {
        if col < self.cells.len() {
            self.cells[col] = cell;
            self.dirty = true;
        }
    }

    /// Clear the row
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
        self.dirty = true;
        self.wrapped = false;
    }

    /// Clear cells from column to end
    pub fn clear_from(&mut self, col: usize) {
        for cell in self.cells.iter_mut().skip(col) {
            cell.reset();
        }
        self.dirty = true;
    }

    /// Clear cells from start to column
    pub fn clear_to(&mut self, col: usize) {
        for cell in self.cells.iter_mut().take(col + 1) {
            cell.reset();
        }
        self.dirty = true;
    }

    /// Resize the row
    pub fn resize(&mut self, cols: usize) {
        self.cells.resize(cols, Cell::default());
        self.dirty = true;
    }

    /// Get all cells
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Check if row is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get row width
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if row is empty
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

/// Current SGR (Select Graphic Rendition) attributes
/// These are applied to new characters as they're written
#[derive(Debug, Clone, Default)]
pub struct GraphicsState {
    pub fg: Color,
    pub bg: Color,
    pub flags: CellFlags,
    pub underline_color: Option<Color>,
}

/// The terminal grid
pub struct Grid {
    /// Rows in the visible area
    rows: Vec<GridRow>,
    /// Scrollback buffer
    scrollback: Vec<GridRow>,
    /// Maximum scrollback lines
    max_scrollback: usize,
    /// Grid width in columns
    cols: usize,
    /// Grid height in rows
    num_rows: usize,
    /// Cursor state
    cursor: Cursor,
    /// Saved cursor position (for DECSC/DECRC)
    saved_cursor: Option<Cursor>,
    /// Current graphics state
    graphics: GraphicsState,
    /// Saved graphics state
    saved_graphics: Option<GraphicsState>,
    /// Scroll region top
    scroll_top: usize,
    /// Scroll region bottom
    scroll_bottom: usize,
    /// Origin mode (DECOM) - reserved for future use
    #[allow(dead_code)]
    origin_mode: bool,
    /// Auto-wrap mode
    auto_wrap: bool,
    /// Insert mode - reserved for future use
    #[allow(dead_code)]
    insert_mode: bool,
    /// Alternate screen buffer
    alternate_screen: Option<Vec<GridRow>>,
}

impl Grid {
    /// Create a new grid with the given dimensions
    pub fn new(cols: usize, rows: usize) -> Self {
        let grid_rows = (0..rows).map(|_| GridRow::new(cols)).collect();
        Self {
            rows: grid_rows,
            scrollback: Vec::new(),
            max_scrollback: 10000,
            cols,
            num_rows: rows,
            cursor: Cursor::new(),
            saved_cursor: None,
            graphics: GraphicsState::default(),
            saved_graphics: None,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            origin_mode: false,
            auto_wrap: true,
            insert_mode: false,
            alternate_screen: None,
        }
    }

    /// Get cell at position
    pub fn cell(&self, col: usize, row: usize) -> Option<&Cell> {
        self.rows.get(row)?.cell(col)
    }

    /// Get mutable cell at position
    pub fn cell_mut(&mut self, col: usize, row: usize) -> Option<&mut Cell> {
        self.rows.get_mut(row)?.cell_mut(col)
    }

    /// Get a row
    pub fn row(&self, row: usize) -> Option<&GridRow> {
        self.rows.get(row)
    }

    /// Get mutable row
    pub fn row_mut(&mut self, row: usize) -> Option<&mut GridRow> {
        self.rows.get_mut(row)
    }

    /// Get all rows
    pub fn rows(&self) -> &[GridRow] {
        &self.rows
    }

    /// Get cursor
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Get mutable cursor
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// Get grid dimensions
    pub fn dims(&self) -> (usize, usize) {
        (self.cols, self.num_rows)
    }

    /// Write a character at cursor position
    pub fn write_char(&mut self, c: char) {
        let col = self.cursor.col;
        let row = self.cursor.row;

        // Handle wide characters
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);

        // Copy graphics state to avoid borrow issues
        let fg = self.graphics.fg;
        let bg = self.graphics.bg;
        let flags = self.graphics.flags;
        let underline_color = self.graphics.underline_color;

        if let Some(cell) = self.cell_mut(col, row) {
            cell.c = c;
            cell.fg = fg;
            cell.bg = bg;
            cell.flags = flags;
            cell.underline_color = underline_color;

            if char_width > 1 {
                cell.flags.insert(CellFlags::WIDE);
            }
        }

        // Mark next cell as spacer for wide characters
        if char_width > 1 && col + 1 < self.cols {
            if let Some(spacer) = self.cell_mut(col + 1, row) {
                spacer.c = ' ';
                spacer.flags = CellFlags::WIDE_SPACER;
            }
        }

        // Advance cursor
        self.cursor.col += char_width;
        if self.cursor.col >= self.cols && self.auto_wrap {
            self.cursor.col = 0;
            self.linefeed();
        }
    }

    /// Move to next line (with scrolling if at bottom)
    pub fn linefeed(&mut self) {
        if self.cursor.row >= self.scroll_bottom {
            self.scroll_up(1);
        } else {
            self.cursor.row += 1;
        }
    }

    /// Carriage return
    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }

    /// Backspace
    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    /// Tab
    pub fn tab(&mut self) {
        // Move to next tab stop (every 8 columns)
        let next_tab = ((self.cursor.col / 8) + 1) * 8;
        self.cursor.col = next_tab.min(self.cols - 1);
    }

    /// Scroll up by n lines
    pub fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            if self.scroll_top < self.rows.len() {
                let row = self.rows.remove(self.scroll_top);
                // Add to scrollback
                if self.scrollback.len() >= self.max_scrollback {
                    self.scrollback.remove(0);
                }
                self.scrollback.push(row);

                // Insert new row at bottom of scroll region
                let insert_pos = self.scroll_bottom.min(self.rows.len());
                self.rows.insert(insert_pos, GridRow::new(self.cols));
            }
        }
    }

    /// Scroll down by n lines
    pub fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            if self.scroll_bottom < self.rows.len() {
                self.rows.remove(self.scroll_bottom);
                self.rows.insert(self.scroll_top, GridRow::new(self.cols));
            }
        }
    }

    /// Clear the screen
    pub fn clear(&mut self) {
        for row in &mut self.rows {
            row.clear();
        }
        self.cursor.col = 0;
        self.cursor.row = 0;
    }

    /// Clear from cursor to end of screen
    pub fn clear_below(&mut self) {
        // Clear rest of current line
        if let Some(row) = self.rows.get_mut(self.cursor.row) {
            row.clear_from(self.cursor.col);
        }
        // Clear all rows below
        for row in self.rows.iter_mut().skip(self.cursor.row + 1) {
            row.clear();
        }
    }

    /// Clear from start of screen to cursor
    pub fn clear_above(&mut self) {
        // Clear start of current line
        if let Some(row) = self.rows.get_mut(self.cursor.row) {
            row.clear_to(self.cursor.col);
        }
        // Clear all rows above
        for row in self.rows.iter_mut().take(self.cursor.row) {
            row.clear();
        }
    }

    /// Clear entire line
    pub fn clear_line(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor.row) {
            row.clear();
        }
    }

    /// Clear from cursor to end of line
    pub fn clear_line_right(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor.row) {
            row.clear_from(self.cursor.col);
        }
    }

    /// Clear from start of line to cursor
    pub fn clear_line_left(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor.row) {
            row.clear_to(self.cursor.col);
        }
    }

    /// Move cursor to position
    pub fn move_cursor(&mut self, col: usize, row: usize) {
        self.cursor.col = col.min(self.cols.saturating_sub(1));
        self.cursor.row = row.min(self.num_rows.saturating_sub(1));
    }

    /// Move cursor relative
    pub fn move_cursor_relative(&mut self, dcol: isize, drow: isize) {
        let new_col = (self.cursor.col as isize + dcol).max(0) as usize;
        let new_row = (self.cursor.row as isize + drow).max(0) as usize;
        self.move_cursor(new_col, new_row);
    }

    /// Save cursor position
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor);
        self.saved_graphics = Some(self.graphics.clone());
    }

    /// Restore cursor position
    pub fn restore_cursor(&mut self) {
        if let Some(cursor) = self.saved_cursor {
            self.cursor = cursor;
        }
        if let Some(graphics) = self.saved_graphics.clone() {
            self.graphics = graphics;
        }
    }

    /// Resize the grid
    pub fn resize(&mut self, cols: usize, rows: usize) {
        // Resize existing rows
        for row in &mut self.rows {
            row.resize(cols);
        }

        // Add or remove rows
        while self.rows.len() < rows {
            self.rows.push(GridRow::new(cols));
        }
        while self.rows.len() > rows {
            // Move excess rows to scrollback
            let row = self.rows.remove(0);
            if self.scrollback.len() < self.max_scrollback {
                self.scrollback.push(row);
            }
        }

        self.cols = cols;
        self.num_rows = rows;
        self.scroll_bottom = rows.saturating_sub(1);

        // Clamp cursor
        self.cursor.col = self.cursor.col.min(cols.saturating_sub(1));
        self.cursor.row = self.cursor.row.min(rows.saturating_sub(1));
    }

    /// Get graphics state
    pub fn graphics(&self) -> &GraphicsState {
        &self.graphics
    }

    /// Get mutable graphics state
    pub fn graphics_mut(&mut self) -> &mut GraphicsState {
        &mut self.graphics
    }

    /// Reset graphics to default
    pub fn reset_graphics(&mut self) {
        self.graphics = GraphicsState::default();
    }

    /// Switch to alternate screen buffer
    pub fn enter_alternate_screen(&mut self) {
        if self.alternate_screen.is_none() {
            let main_screen = std::mem::replace(
                &mut self.rows,
                (0..self.num_rows)
                    .map(|_| GridRow::new(self.cols))
                    .collect(),
            );
            self.alternate_screen = Some(main_screen);
        }
    }

    /// Switch back to main screen buffer
    pub fn exit_alternate_screen(&mut self) {
        if let Some(main_screen) = self.alternate_screen.take() {
            self.rows = main_screen;
        }
    }

    /// Get dirty rows for rendering
    pub fn dirty_rows(&self) -> impl Iterator<Item = (usize, &GridRow)> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.is_dirty())
    }

    /// Clear all dirty flags
    pub fn clear_dirty(&mut self) {
        for row in &mut self.rows {
            row.clear_dirty();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(80, 24);
        assert_eq!(grid.dims(), (80, 24));
    }

    #[test]
    fn test_write_char() {
        let mut grid = Grid::new(80, 24);
        grid.write_char('H');
        grid.write_char('i');

        assert_eq!(grid.cell(0, 0).map(|c| c.c), Some('H'));
        assert_eq!(grid.cell(1, 0).map(|c| c.c), Some('i'));
        assert_eq!(grid.cursor().col, 2);
    }

    #[test]
    fn test_linefeed() {
        let mut grid = Grid::new(80, 24);
        grid.linefeed();
        assert_eq!(grid.cursor().row, 1);
    }

    #[test]
    fn test_scroll() {
        let mut grid = Grid::new(80, 3);
        grid.move_cursor(0, 2);
        grid.write_char('X');
        grid.linefeed();

        // X should have scrolled up
        assert_eq!(grid.cell(0, 1).map(|c| c.c), Some('X'));
    }
}
