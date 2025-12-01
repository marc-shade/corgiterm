//! VTE-based terminal parser
//!
//! Uses the vte crate (same as Alacritty) for escape sequence parsing.
//! This provides robust, well-tested handling of all terminal escape codes.

use crate::cell::{CellFlags, Color, Rgb};
use crate::grid::Grid;
use tracing::{debug, trace};
use vte::{Params, Parser, Perform};

/// Terminal parser using VTE
pub struct TerminalParser {
    parser: Parser,
}

impl TerminalParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Process a single byte
    pub fn advance(&mut self, grid: &mut Grid, byte: u8) {
        let mut performer = TerminalPerformer { grid };
        self.parser.advance(&mut performer, byte);
    }

    /// Process multiple bytes
    pub fn process(&mut self, grid: &mut Grid, bytes: &[u8]) {
        let mut performer = TerminalPerformer { grid };
        for byte in bytes {
            self.parser.advance(&mut performer, *byte);
        }
    }
}

impl Default for TerminalParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Performer that applies escape sequences to the grid
struct TerminalPerformer<'a> {
    grid: &'a mut Grid,
}

impl<'a> Perform for TerminalPerformer<'a> {
    /// Print a character to the terminal
    fn print(&mut self, c: char) {
        trace!("print: {:?}", c);
        self.grid.write_char(c);
    }

    /// Execute a C0/C1 control character
    fn execute(&mut self, byte: u8) {
        trace!("execute: 0x{:02X}", byte);
        match byte {
            // Bell
            0x07 => {
                debug!("bell");
            }
            // Backspace
            0x08 => {
                self.grid.backspace();
            }
            // Horizontal Tab
            0x09 => {
                self.grid.tab();
            }
            // Line Feed / New Line / Vertical Tab
            0x0A | 0x0B | 0x0C => {
                self.grid.linefeed();
            }
            // Carriage Return
            0x0D => {
                self.grid.carriage_return();
            }
            // Shift Out (switch to G1 charset) - ignore for now
            0x0E => {}
            // Shift In (switch to G0 charset) - ignore for now
            0x0F => {}
            _ => {
                trace!("unhandled execute: 0x{:02X}", byte);
            }
        }
    }

    /// Handle a CSI (Control Sequence Introducer) sequence
    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        trace!(
            "CSI: params={:?}, intermediates={:?}, action={:?}",
            params.iter().collect::<Vec<_>>(),
            intermediates,
            action
        );

        let params: Vec<u16> = params.iter().flat_map(|p| p.iter().copied()).collect();
        let param = |idx: usize, default: u16| params.get(idx).copied().unwrap_or(default);

        match action {
            // Cursor Up (CUU)
            'A' => {
                let n = param(0, 1).max(1) as isize;
                self.grid.move_cursor_relative(0, -n);
            }
            // Cursor Down (CUD)
            'B' => {
                let n = param(0, 1).max(1) as isize;
                self.grid.move_cursor_relative(0, n);
            }
            // Cursor Forward (CUF)
            'C' => {
                let n = param(0, 1).max(1) as isize;
                self.grid.move_cursor_relative(n, 0);
            }
            // Cursor Back (CUB)
            'D' => {
                let n = param(0, 1).max(1) as isize;
                self.grid.move_cursor_relative(-n, 0);
            }
            // Cursor Next Line (CNL)
            'E' => {
                let n = param(0, 1).max(1) as isize;
                self.grid.carriage_return();
                self.grid.move_cursor_relative(0, n);
            }
            // Cursor Previous Line (CPL)
            'F' => {
                let n = param(0, 1).max(1) as isize;
                self.grid.carriage_return();
                self.grid.move_cursor_relative(0, -n);
            }
            // Cursor Horizontal Absolute (CHA)
            'G' => {
                let col = (param(0, 1).saturating_sub(1)) as usize;
                let row = self.grid.cursor().row;
                self.grid.move_cursor(col, row);
            }
            // Cursor Position (CUP) / Horizontal and Vertical Position (HVP)
            'H' | 'f' => {
                let row = (param(0, 1).saturating_sub(1)) as usize;
                let col = (param(1, 1).saturating_sub(1)) as usize;
                self.grid.move_cursor(col, row);
            }
            // Erase in Display (ED)
            'J' => {
                match param(0, 0) {
                    0 => self.grid.clear_below(),    // From cursor to end
                    1 => self.grid.clear_above(),    // From start to cursor
                    2 | 3 => self.grid.clear(),      // Entire screen
                    _ => {}
                }
            }
            // Erase in Line (EL)
            'K' => {
                match param(0, 0) {
                    0 => self.grid.clear_line_right(), // From cursor to end
                    1 => self.grid.clear_line_left(),  // From start to cursor
                    2 => self.grid.clear_line(),       // Entire line
                    _ => {}
                }
            }
            // Insert Lines (IL)
            'L' => {
                let n = param(0, 1).max(1) as usize;
                self.grid.scroll_down(n);
            }
            // Delete Lines (DL)
            'M' => {
                let n = param(0, 1).max(1) as usize;
                self.grid.scroll_up(n);
            }
            // Scroll Up (SU)
            'S' => {
                let n = param(0, 1).max(1) as usize;
                self.grid.scroll_up(n);
            }
            // Scroll Down (SD)
            'T' => {
                let n = param(0, 1).max(1) as usize;
                self.grid.scroll_down(n);
            }
            // Select Graphic Rendition (SGR)
            'm' => {
                self.handle_sgr(&params);
            }
            // Device Status Report (DSR)
            'n' => {
                // Would need to write response to PTY - skip for now
                debug!("DSR request: {:?}", params);
            }
            // Save Cursor Position (DECSC via CSI)
            's' => {
                self.grid.save_cursor();
            }
            // Restore Cursor Position (DECRC via CSI)
            'u' => {
                self.grid.restore_cursor();
            }
            // Set Mode (SM) / Reset Mode (RM)
            'h' | 'l' => {
                let set = action == 'h';
                if intermediates.first() == Some(&b'?') {
                    // DEC private modes
                    self.handle_dec_mode(&params, set);
                }
            }
            _ => {
                debug!("unhandled CSI: {:?} {:?}", action, params);
            }
        }
    }

    /// Handle ESC sequence
    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        trace!("ESC: intermediates={:?}, byte=0x{:02X}", intermediates, byte);

        match (intermediates.first(), byte) {
            // Save Cursor (DECSC)
            (None, b'7') => {
                self.grid.save_cursor();
            }
            // Restore Cursor (DECRC)
            (None, b'8') => {
                self.grid.restore_cursor();
            }
            // Reset to Initial State (RIS)
            (None, b'c') => {
                self.grid.clear();
                self.grid.reset_graphics();
            }
            // Index (IND) - move cursor down, scroll if needed
            (None, b'D') => {
                self.grid.linefeed();
            }
            // Reverse Index (RI) - move cursor up, scroll if needed
            (None, b'M') => {
                let row = self.grid.cursor().row;
                if row == 0 {
                    self.grid.scroll_down(1);
                } else {
                    self.grid.move_cursor_relative(0, -1);
                }
            }
            // Next Line (NEL)
            (None, b'E') => {
                self.grid.carriage_return();
                self.grid.linefeed();
            }
            _ => {
                debug!("unhandled ESC: {:?} 0x{:02X}", intermediates, byte);
            }
        }
    }

    /// Handle OSC (Operating System Command)
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        trace!("OSC: {:?}", params);

        if params.is_empty() {
            return;
        }

        // Parse OSC command number
        let cmd_str = std::str::from_utf8(params[0]).unwrap_or("");
        let _cmd: u16 = cmd_str.parse().unwrap_or(0);

        // OSC commands like setting window title - would need callback
        debug!("OSC command: {:?}", params);
    }

    /// Hook for DCS (Device Control String) start
    fn hook(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, _action: char) {
        trace!("DCS hook: params={:?}, intermediates={:?}",
            params.iter().collect::<Vec<_>>(), intermediates);
    }

    /// DCS data
    fn put(&mut self, _byte: u8) {}

    /// DCS end
    fn unhook(&mut self) {}
}

/// Parse extended color (256-color or true color) - standalone function
fn parse_extended_color(params: &[u16], i: &mut usize) -> Option<Color> {
    if *i + 1 >= params.len() {
        return None;
    }

    match params[*i + 1] {
        // 256-color mode
        5 => {
            if *i + 2 < params.len() {
                *i += 2;
                Some(Color::Indexed(params[*i] as u8))
            } else {
                None
            }
        }
        // True color mode
        2 => {
            if *i + 4 < params.len() {
                let r = params[*i + 2] as u8;
                let g = params[*i + 3] as u8;
                let b = params[*i + 4] as u8;
                *i += 4;
                Some(Color::Rgb(Rgb::new(r, g, b)))
            } else {
                None
            }
        }
        _ => None,
    }
}

impl<'a> TerminalPerformer<'a> {
    /// Handle SGR (Select Graphic Rendition) parameters
    fn handle_sgr(&mut self, params: &[u16]) {
        let mut i = 0;

        while i < params.len() {
            let graphics = self.grid.graphics_mut();
            match params[i] {
                // Reset
                0 => {
                    graphics.fg = Color::Foreground;
                    graphics.bg = Color::Background;
                    graphics.flags = CellFlags::empty();
                    graphics.underline_color = None;
                }
                // Bold
                1 => graphics.flags.insert(CellFlags::BOLD),
                // Dim
                2 => graphics.flags.insert(CellFlags::DIM),
                // Italic
                3 => graphics.flags.insert(CellFlags::ITALIC),
                // Underline
                4 => {
                    graphics.flags.insert(CellFlags::UNDERLINE);
                    // Check for underline style parameter
                    if i + 1 < params.len() {
                        match params[i + 1] {
                            0 => graphics.flags.remove(CellFlags::UNDERLINE),
                            1 => {} // Single underline (default)
                            2 => graphics.flags.insert(CellFlags::DOUBLE_UNDERLINE),
                            3 => graphics.flags.insert(CellFlags::CURLY_UNDERLINE),
                            _ => {}
                        }
                    }
                }
                // Blink
                5 | 6 => graphics.flags.insert(CellFlags::BLINK),
                // Inverse
                7 => graphics.flags.insert(CellFlags::INVERSE),
                // Hidden
                8 => graphics.flags.insert(CellFlags::HIDDEN),
                // Strikethrough
                9 => graphics.flags.insert(CellFlags::STRIKETHROUGH),
                // Normal intensity (not bold, not dim)
                22 => {
                    graphics.flags.remove(CellFlags::BOLD);
                    graphics.flags.remove(CellFlags::DIM);
                }
                // Not italic
                23 => graphics.flags.remove(CellFlags::ITALIC),
                // Not underlined
                24 => {
                    graphics.flags.remove(CellFlags::UNDERLINE);
                    graphics.flags.remove(CellFlags::DOUBLE_UNDERLINE);
                    graphics.flags.remove(CellFlags::CURLY_UNDERLINE);
                }
                // Not blinking
                25 => graphics.flags.remove(CellFlags::BLINK),
                // Not inverse
                27 => graphics.flags.remove(CellFlags::INVERSE),
                // Not hidden
                28 => graphics.flags.remove(CellFlags::HIDDEN),
                // Not strikethrough
                29 => graphics.flags.remove(CellFlags::STRIKETHROUGH),
                // Foreground colors (30-37)
                30..=37 => graphics.fg = Color::Indexed((params[i] - 30) as u8),
                // Extended foreground color
                38 => {
                    if let Some(color) = parse_extended_color(params, &mut i) {
                        self.grid.graphics_mut().fg = color;
                    }
                }
                // Default foreground
                39 => graphics.fg = Color::Foreground,
                // Background colors (40-47)
                40..=47 => graphics.bg = Color::Indexed((params[i] - 40) as u8),
                // Extended background color
                48 => {
                    if let Some(color) = parse_extended_color(params, &mut i) {
                        self.grid.graphics_mut().bg = color;
                    }
                }
                // Default background
                49 => graphics.bg = Color::Background,
                // Bright foreground colors (90-97)
                90..=97 => graphics.fg = Color::Indexed((params[i] - 90 + 8) as u8),
                // Bright background colors (100-107)
                100..=107 => graphics.bg = Color::Indexed((params[i] - 100 + 8) as u8),
                _ => {
                    trace!("unhandled SGR: {}", params[i]);
                }
            }
            i += 1;
        }
    }

    /// Handle DEC private modes
    fn handle_dec_mode(&mut self, params: &[u16], set: bool) {
        for &mode in params {
            match mode {
                // Cursor visibility (DECTCEM)
                25 => {
                    self.grid.cursor_mut().visible = set;
                }
                // Alternate screen buffer
                1049 => {
                    if set {
                        self.grid.enter_alternate_screen();
                    } else {
                        self.grid.exit_alternate_screen();
                    }
                }
                // Many more modes exist - add as needed
                _ => {
                    debug!("unhandled DEC mode: {} = {}", mode, set);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text() {
        let mut grid = Grid::new(80, 24);
        let mut parser = TerminalParser::new();

        parser.process(&mut grid, b"Hello");

        assert_eq!(grid.cell(0, 0).map(|c| c.c), Some('H'));
        assert_eq!(grid.cell(4, 0).map(|c| c.c), Some('o'));
    }

    #[test]
    fn test_cursor_movement() {
        let mut grid = Grid::new(80, 24);
        let mut parser = TerminalParser::new();

        // Move to row 5, col 10
        parser.process(&mut grid, b"\x1b[5;10H");

        assert_eq!(grid.cursor().row, 4); // 0-indexed
        assert_eq!(grid.cursor().col, 9); // 0-indexed
    }

    #[test]
    fn test_sgr_colors() {
        let mut grid = Grid::new(80, 24);
        let mut parser = TerminalParser::new();

        // Set red foreground
        parser.process(&mut grid, b"\x1b[31mR");

        let cell = grid.cell(0, 0).unwrap();
        assert_eq!(cell.c, 'R');
        assert_eq!(cell.fg, Color::Indexed(1)); // Red
    }

    #[test]
    fn test_clear_screen() {
        let mut grid = Grid::new(80, 24);
        let mut parser = TerminalParser::new();

        parser.process(&mut grid, b"Hello");
        parser.process(&mut grid, b"\x1b[2J");

        // Screen should be cleared
        assert_eq!(grid.cell(0, 0).map(|c| c.c), Some(' '));
    }
}
