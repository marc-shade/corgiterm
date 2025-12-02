//! Terminal cell representation
//!
//! A cell is the fundamental unit of the terminal grid.
//! Each cell contains a character and its associated attributes.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to array for GPU upload
    pub fn to_array(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    /// Parse from hex string like "#RRGGBB"
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

/// Color type - indexed (ANSI) or RGB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Color {
    /// Named/indexed color (0-255)
    Indexed(u8),
    /// True color RGB
    Rgb(Rgb),
    /// Default foreground
    #[default]
    Foreground,
    /// Default background
    Background,
}

impl Color {
    /// Standard ANSI colors
    pub const BLACK: Self = Color::Indexed(0);
    pub const RED: Self = Color::Indexed(1);
    pub const GREEN: Self = Color::Indexed(2);
    pub const YELLOW: Self = Color::Indexed(3);
    pub const BLUE: Self = Color::Indexed(4);
    pub const MAGENTA: Self = Color::Indexed(5);
    pub const CYAN: Self = Color::Indexed(6);
    pub const WHITE: Self = Color::Indexed(7);

    /// Bright ANSI colors
    pub const BRIGHT_BLACK: Self = Color::Indexed(8);
    pub const BRIGHT_RED: Self = Color::Indexed(9);
    pub const BRIGHT_GREEN: Self = Color::Indexed(10);
    pub const BRIGHT_YELLOW: Self = Color::Indexed(11);
    pub const BRIGHT_BLUE: Self = Color::Indexed(12);
    pub const BRIGHT_MAGENTA: Self = Color::Indexed(13);
    pub const BRIGHT_CYAN: Self = Color::Indexed(14);
    pub const BRIGHT_WHITE: Self = Color::Indexed(15);
}

bitflags! {
    /// Cell attribute flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct CellFlags: u16 {
        /// Bold text
        const BOLD = 0b0000_0000_0001;
        /// Dim/faint text
        const DIM = 0b0000_0000_0010;
        /// Italic text
        const ITALIC = 0b0000_0000_0100;
        /// Underlined text
        const UNDERLINE = 0b0000_0000_1000;
        /// Blinking text
        const BLINK = 0b0000_0001_0000;
        /// Inverse video (swap fg/bg)
        const INVERSE = 0b0000_0010_0000;
        /// Hidden/invisible
        const HIDDEN = 0b0000_0100_0000;
        /// Strikethrough
        const STRIKETHROUGH = 0b0000_1000_0000;
        /// Double underline
        const DOUBLE_UNDERLINE = 0b0001_0000_0000;
        /// Curly underline
        const CURLY_UNDERLINE = 0b0010_0000_0000;
        /// Wide character (occupies two cells)
        const WIDE = 0b0100_0000_0000;
        /// Continuation of wide character
        const WIDE_SPACER = 0b1000_0000_0000;
    }
}

/// A single terminal cell
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// The character in this cell
    pub c: char,
    /// Foreground color
    pub fg: Color,
    /// Background color
    pub bg: Color,
    /// Underline color (for colored underlines)
    pub underline_color: Option<Color>,
    /// Cell flags (bold, italic, etc.)
    pub flags: CellFlags,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color::Foreground,
            bg: Color::Background,
            underline_color: None,
            flags: CellFlags::empty(),
        }
    }
}

impl Cell {
    /// Create a new cell with the given character
    pub fn new(c: char) -> Self {
        Self {
            c,
            ..Default::default()
        }
    }

    /// Create a cell with character and colors
    pub fn with_colors(c: char, fg: Color, bg: Color) -> Self {
        Self {
            c,
            fg,
            bg,
            ..Default::default()
        }
    }

    /// Check if this cell is empty (space with default colors)
    pub fn is_empty(&self) -> bool {
        self.c == ' '
            && self.fg == Color::Foreground
            && self.bg == Color::Background
            && self.flags.is_empty()
    }

    /// Check if this is a wide character
    pub fn is_wide(&self) -> bool {
        self.flags.contains(CellFlags::WIDE)
    }

    /// Check if this is a wide character spacer
    pub fn is_wide_spacer(&self) -> bool {
        self.flags.contains(CellFlags::WIDE_SPACER)
    }

    /// Reset the cell to empty state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Apply SGR (Select Graphic Rendition) attributes
    pub fn set_bold(&mut self, bold: bool) {
        self.flags.set(CellFlags::BOLD, bold);
    }

    pub fn set_italic(&mut self, italic: bool) {
        self.flags.set(CellFlags::ITALIC, italic);
    }

    pub fn set_underline(&mut self, underline: bool) {
        self.flags.set(CellFlags::UNDERLINE, underline);
    }

    pub fn set_inverse(&mut self, inverse: bool) {
        self.flags.set(CellFlags::INVERSE, inverse);
    }

    pub fn set_strikethrough(&mut self, strikethrough: bool) {
        self.flags.set(CellFlags::STRIKETHROUGH, strikethrough);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_from_hex() {
        let color = Rgb::from_hex("#FF5500").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 85);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.c, ' ');
        assert!(cell.is_empty());
    }

    #[test]
    fn test_cell_flags() {
        let mut cell = Cell::new('A');
        cell.set_bold(true);
        cell.set_italic(true);
        assert!(cell.flags.contains(CellFlags::BOLD));
        assert!(cell.flags.contains(CellFlags::ITALIC));
    }
}
