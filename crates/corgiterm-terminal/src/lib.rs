//! # CorgiTerm Terminal Backend
//!
//! High-performance, Wayland-native terminal emulation engine.
//! Inspired by foot and Alacritty for maximum stability and speed.
//!
//! ## Architecture
//!
//! - **Grid**: 2D cell grid with efficient damage tracking
//! - **Parser**: VTE-based escape sequence parsing (same as Alacritty)
//! - **Renderer**: GPU-accelerated text rendering via wgpu/glyphon
//!
//! ## Design Goals
//!
//! 1. Zero panics - all errors are handled gracefully
//! 2. Minimal allocations in hot paths
//! 3. Damage tracking for efficient redraw
//! 4. Full Unicode/emoji support
//!
//! ```text
//!    ∩＿∩
//!   (・ω・)  Stability First!
//!   /　 つ   Making terminals rock-solid
//! ```

pub mod cell;
pub mod grid;
pub mod parser;
pub mod renderer;
pub mod selection;
pub mod ansi;

pub use cell::{Cell, CellFlags, Color, Rgb};
pub use grid::{Grid, GridRow, Cursor, CursorShape, GraphicsState};
pub use parser::TerminalParser;
pub use renderer::{GpuRenderer, RenderStats, RendererConfig};
pub use selection::{Selection, SelectionRange, SelectionType, Point};
pub use ansi::{AnsiPalette, AnsiColor};

use thiserror::Error;

/// Terminal backend errors - all recoverable, no panics
#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("Renderer initialization failed: {0}")]
    RendererInit(String),

    #[error("Font loading failed: {0}")]
    FontLoad(String),

    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Parse error at position {position}: {message}")]
    Parse { position: usize, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal state invalid: {0}")]
    InvalidState(String),

    #[error("Recovery failed after {attempts} attempts: {reason}")]
    RecoveryFailed { attempts: u32, reason: String },
}

/// Health status of the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalHealth {
    /// Terminal is functioning normally
    Healthy,
    /// Terminal recovered from an error
    Recovered,
    /// Terminal is in degraded mode (some features disabled)
    Degraded,
    /// Terminal needs reset to recover
    NeedsReset,
}

pub type Result<T> = std::result::Result<T, TerminalError>;

/// Terminal size in cells and pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    /// Width in cells
    pub cols: u16,
    /// Height in cells
    pub rows: u16,
    /// Cell width in pixels
    pub cell_width: u16,
    /// Cell height in pixels
    pub cell_height: u16,
}

impl TerminalSize {
    pub fn new(cols: u16, rows: u16, cell_width: u16, cell_height: u16) -> Self {
        Self { cols, rows, cell_width, cell_height }
    }

    /// Total width in pixels
    pub fn width_px(&self) -> u32 {
        self.cols as u32 * self.cell_width as u32
    }

    /// Total height in pixels
    pub fn height_px(&self) -> u32 {
        self.rows as u32 * self.cell_height as u32
    }
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            cell_width: 10,
            cell_height: 20,
        }
    }
}

/// The main terminal state machine
pub struct Terminal {
    /// The cell grid
    grid: Grid,
    /// VTE parser
    parser: TerminalParser,
    /// Current terminal size
    size: TerminalSize,
    /// Damage regions for efficient redraw
    damage: Vec<DamageRect>,
    /// Whether full redraw is needed
    full_damage: bool,
    /// Current health status
    health: TerminalHealth,
    /// Error count for recovery decisions
    error_count: u32,
    /// Maximum errors before forced reset
    max_errors: u32,
}

/// A rectangular damage region
#[derive(Debug, Clone, Copy)]
pub struct DamageRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Terminal {
    /// Create a new terminal with the given size
    pub fn new(size: TerminalSize) -> Self {
        Self {
            grid: Grid::new(size.cols as usize, size.rows as usize),
            parser: TerminalParser::new(),
            size,
            damage: Vec::new(),
            full_damage: true,
            health: TerminalHealth::Healthy,
            error_count: 0,
            max_errors: 10, // Reset after 10 consecutive errors
        }
    }

    /// Process input bytes from PTY
    pub fn process(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.parser.advance(&mut self.grid, *byte);
        }
        // Mark entire screen as damaged for now (optimize later with damage tracking)
        self.full_damage = true;

        // Clear error count on successful processing
        if self.error_count > 0 {
            self.error_count = 0;
            if self.health == TerminalHealth::Degraded {
                self.health = TerminalHealth::Recovered;
            }
        }
    }

    /// Process bytes with automatic recovery on errors
    /// Returns the health status after processing
    pub fn safe_process(&mut self, bytes: &[u8]) -> TerminalHealth {
        // First validate state
        if let Err(_) = self.validate_state() {
            self.error_count += 1;

            if self.error_count >= self.max_errors {
                // Too many errors, perform hard reset
                self.hard_reset();
                self.health = TerminalHealth::Recovered;
            } else {
                // Try soft reset first
                self.soft_reset();
                self.health = TerminalHealth::Degraded;
            }
        }

        // Process bytes (VTE parser handles invalid sequences gracefully)
        self.process(bytes);

        self.health
    }

    /// Validate terminal state is consistent
    pub fn validate_state(&self) -> Result<()> {
        let (cols, rows) = self.grid.dims();
        let cursor = self.grid.cursor();

        // Check cursor is within bounds
        if cursor.col >= cols {
            return Err(TerminalError::InvalidState(format!(
                "Cursor column {} out of bounds (max {})", cursor.col, cols - 1
            )));
        }

        if cursor.row >= rows {
            return Err(TerminalError::InvalidState(format!(
                "Cursor row {} out of bounds (max {})", cursor.row, rows - 1
            )));
        }

        // Check size consistency
        if self.size.cols as usize != cols || self.size.rows as usize != rows {
            return Err(TerminalError::InvalidState(
                "Grid dimensions don't match terminal size".into()
            ));
        }

        Ok(())
    }

    /// Soft reset - reset graphics state but preserve content
    /// Similar to DECSTR (Soft Terminal Reset)
    pub fn soft_reset(&mut self) {
        self.grid.reset_graphics();

        // Get bounds first to avoid borrow conflict
        let (cols, rows) = self.grid.dims();

        // Reset cursor to visible block and clamp to valid bounds
        let cursor = self.grid.cursor_mut();
        cursor.visible = true;
        cursor.shape = CursorShape::Block;
        cursor.blinking = true;

        if cursor.col >= cols {
            cursor.col = cols.saturating_sub(1);
        }
        if cursor.row >= rows {
            cursor.row = rows.saturating_sub(1);
        }

        self.full_damage = true;
    }

    /// Hard reset - clear everything and start fresh
    /// Similar to RIS (Reset to Initial State)
    pub fn hard_reset(&mut self) {
        self.grid = Grid::new(self.size.cols as usize, self.size.rows as usize);
        self.parser = TerminalParser::new();
        self.damage.clear();
        self.full_damage = true;
        self.error_count = 0;
        self.health = TerminalHealth::Healthy;
    }

    /// Get current health status
    pub fn health(&self) -> TerminalHealth {
        self.health
    }

    /// Check if terminal needs user intervention
    pub fn needs_reset(&self) -> bool {
        self.health == TerminalHealth::NeedsReset
    }

    /// Resize the terminal
    pub fn resize(&mut self, size: TerminalSize) {
        self.size = size;
        self.grid.resize(size.cols as usize, size.rows as usize);
        self.full_damage = true;
    }

    /// Get the cell grid
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Get mutable grid
    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Get terminal size
    pub fn size(&self) -> TerminalSize {
        self.size
    }

    /// Check if full redraw is needed
    pub fn needs_redraw(&self) -> bool {
        self.full_damage || !self.damage.is_empty()
    }

    /// Clear damage after rendering
    pub fn clear_damage(&mut self) {
        self.damage.clear();
        self.full_damage = false;
    }

    /// Get cursor position
    pub fn cursor(&self) -> &Cursor {
        self.grid.cursor()
    }

    /// Get error count (for diagnostics)
    pub fn error_count(&self) -> u32 {
        self.error_count
    }
}

/// Version of the terminal backend
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let term = Terminal::new(TerminalSize::default());
        assert_eq!(term.size().cols, 80);
        assert_eq!(term.size().rows, 24);
        assert_eq!(term.health(), TerminalHealth::Healthy);
    }

    #[test]
    fn test_process_simple_text() {
        let mut term = Terminal::new(TerminalSize::default());
        term.process(b"Hello, World!");

        // Check that text was written to grid
        let grid = term.grid();
        assert_eq!(grid.cell(0, 0).map(|c| c.c), Some('H'));
        assert_eq!(grid.cell(1, 0).map(|c| c.c), Some('e'));
    }

    #[test]
    fn test_resize() {
        let mut term = Terminal::new(TerminalSize::default());
        term.resize(TerminalSize::new(120, 40, 10, 20));
        assert_eq!(term.size().cols, 120);
        assert_eq!(term.size().rows, 40);
    }

    #[test]
    fn test_soft_reset() {
        let mut term = Terminal::new(TerminalSize::default());
        term.process(b"\x1b[31mRed text"); // Set red foreground

        term.soft_reset();

        // Graphics should be reset but content preserved
        assert_eq!(term.grid().cell(0, 0).map(|c| c.c), Some('R'));
        assert_eq!(term.cursor().visible, true);
        assert_eq!(term.cursor().shape, CursorShape::Block);
    }

    #[test]
    fn test_hard_reset() {
        let mut term = Terminal::new(TerminalSize::default());
        term.process(b"Hello, World!");

        term.hard_reset();

        // Everything should be cleared
        assert_eq!(term.grid().cell(0, 0).map(|c| c.c), Some(' '));
        assert_eq!(term.cursor().col, 0);
        assert_eq!(term.cursor().row, 0);
        assert_eq!(term.health(), TerminalHealth::Healthy);
    }

    #[test]
    fn test_validate_state() {
        let term = Terminal::new(TerminalSize::default());
        assert!(term.validate_state().is_ok());
    }

    #[test]
    fn test_safe_process() {
        let mut term = Terminal::new(TerminalSize::default());

        // Safe process should return Healthy on normal input
        let health = term.safe_process(b"Normal text");
        assert!(health == TerminalHealth::Healthy || health == TerminalHealth::Recovered);
    }
}
