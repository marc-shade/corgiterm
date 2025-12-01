//! ANSI color palette definitions
//!
//! Standard 256-color palette used by terminals.
//! Includes the 16 standard colors, 216 color cube, and 24 grayscale colors.

use crate::cell::Rgb;

/// Default ANSI color palette (256 colors)
pub struct AnsiPalette {
    colors: [Rgb; 256],
}

impl AnsiPalette {
    /// Create a new ANSI palette with default colors
    pub fn new() -> Self {
        let mut colors = [Rgb::default(); 256];

        // Standard 16 colors (0-15)
        // These match typical terminal defaults
        colors[0] = Rgb::new(0x00, 0x00, 0x00);  // Black
        colors[1] = Rgb::new(0xCD, 0x00, 0x00);  // Red
        colors[2] = Rgb::new(0x00, 0xCD, 0x00);  // Green
        colors[3] = Rgb::new(0xCD, 0xCD, 0x00);  // Yellow
        colors[4] = Rgb::new(0x00, 0x00, 0xEE);  // Blue
        colors[5] = Rgb::new(0xCD, 0x00, 0xCD);  // Magenta
        colors[6] = Rgb::new(0x00, 0xCD, 0xCD);  // Cyan
        colors[7] = Rgb::new(0xE5, 0xE5, 0xE5);  // White

        // Bright colors (8-15)
        colors[8] = Rgb::new(0x7F, 0x7F, 0x7F);   // Bright Black (Gray)
        colors[9] = Rgb::new(0xFF, 0x00, 0x00);   // Bright Red
        colors[10] = Rgb::new(0x00, 0xFF, 0x00);  // Bright Green
        colors[11] = Rgb::new(0xFF, 0xFF, 0x00);  // Bright Yellow
        colors[12] = Rgb::new(0x5C, 0x5C, 0xFF);  // Bright Blue
        colors[13] = Rgb::new(0xFF, 0x00, 0xFF);  // Bright Magenta
        colors[14] = Rgb::new(0x00, 0xFF, 0xFF);  // Bright Cyan
        colors[15] = Rgb::new(0xFF, 0xFF, 0xFF);  // Bright White

        // 216-color cube (16-231)
        // 6x6x6 RGB cube
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let index = 16 + r * 36 + g * 6 + b;
                    let rv = if r == 0 { 0 } else { 55 + r * 40 };
                    let gv = if g == 0 { 0 } else { 55 + g * 40 };
                    let bv = if b == 0 { 0 } else { 55 + b * 40 };
                    colors[index] = Rgb::new(rv as u8, gv as u8, bv as u8);
                }
            }
        }

        // 24 grayscale colors (232-255)
        for i in 0..24 {
            let v = (8 + i * 10) as u8;
            colors[232 + i] = Rgb::new(v, v, v);
        }

        Self { colors }
    }

    /// Get color by index (0-255)
    pub fn get(&self, index: u8) -> Rgb {
        self.colors[index as usize]
    }

    /// Set a color at index
    pub fn set(&mut self, index: u8, color: Rgb) {
        self.colors[index as usize] = color;
    }

    /// Reset to default colors
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get all colors
    pub fn colors(&self) -> &[Rgb; 256] {
        &self.colors
    }
}

impl Default for AnsiPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// Named ANSI color indices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AnsiColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl From<AnsiColor> for u8 {
    fn from(color: AnsiColor) -> u8 {
        color as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_creation() {
        let palette = AnsiPalette::new();

        // Check black
        assert_eq!(palette.get(0), Rgb::new(0, 0, 0));

        // Check white
        assert_eq!(palette.get(7), Rgb::new(0xE5, 0xE5, 0xE5));

        // Check bright white
        assert_eq!(palette.get(15), Rgb::new(0xFF, 0xFF, 0xFF));
    }

    #[test]
    fn test_color_cube() {
        let palette = AnsiPalette::new();

        // Index 16 should be black (0,0,0 in the cube)
        assert_eq!(palette.get(16), Rgb::new(0, 0, 0));

        // Index 231 should be white (5,5,5 in the cube)
        assert_eq!(palette.get(231), Rgb::new(255, 255, 255));
    }

    #[test]
    fn test_grayscale() {
        let palette = AnsiPalette::new();

        // First grayscale (232) should be dark gray
        assert_eq!(palette.get(232), Rgb::new(8, 8, 8));

        // Last grayscale (255) should be near white
        assert_eq!(palette.get(255), Rgb::new(238, 238, 238));
    }
}
