//! # ASCII Art Generator
//!
//! Convert images and text to ASCII art for display in terminals.
//!
//! Features:
//! - Image to ASCII conversion with brightness mapping
//! - Multiple character sets (simple, detailed, blocks)
//! - ANSI color support
//! - Text to ASCII art with figlet-style fonts
//! - Built-in Corgi art collection 🐕
//!
//! ```text
//!      ／＞　 フ
//!     | 　_　_|
//!   ／` ミ＿xノ   ASCII Art Generator
//!  /　　　　  |   Making terminals fun!
//! /　 ヽ　　 ﾉ
//! │　　|　|　|
//! ```

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, Rgba};
use std::path::Path;

/// Character sets for ASCII art generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSet {
    /// Simple set: " .:-=+*#@"
    Simple,
    /// Detailed set with more gradations
    Detailed,
    /// Unicode block characters
    Blocks,
    /// Braille characters for high detail
    Braille,
}

impl CharacterSet {
    /// Get the character array for this set
    pub fn chars(&self) -> &[char] {
        match self {
            CharacterSet::Simple => &[' ', '.', ':', '-', '=', '+', '*', '#', '@'],
            CharacterSet::Detailed => &[
                ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~',
                '+', '_', '-', '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f',
                'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0',
                'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W',
                '&', '8', '%', 'B', '@', '$',
            ],
            CharacterSet::Blocks => &[' ', '░', '▒', '▓', '█'],
            CharacterSet::Braille => &[' ', '⠁', '⠃', '⠇', '⠏', '⠟', '⠿', '⡿', '⣿'],
        }
    }

    /// Get character for brightness value (0.0 = black, 1.0 = white)
    pub fn char_for_brightness(&self, brightness: f32) -> char {
        let chars = self.chars();
        let index = ((brightness * (chars.len() - 1) as f32) as usize).min(chars.len() - 1);
        chars[index]
    }

    /// Get all available character sets
    pub fn all() -> &'static [CharacterSet] {
        &[
            CharacterSet::Simple,
            CharacterSet::Detailed,
            CharacterSet::Blocks,
            CharacterSet::Braille,
        ]
    }

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            CharacterSet::Simple => "Simple",
            CharacterSet::Detailed => "Detailed",
            CharacterSet::Blocks => "Blocks",
            CharacterSet::Braille => "Braille",
        }
    }
}

/// Configuration for ASCII art generation
#[derive(Debug, Clone)]
pub struct AsciiArtConfig {
    /// Target width in characters (None = auto-fit)
    pub width: Option<usize>,
    /// Character set to use
    pub charset: CharacterSet,
    /// Include ANSI colors
    pub colored: bool,
    /// Invert brightness (white background)
    pub inverted: bool,
    /// Aspect ratio correction (terminal chars are ~2:1)
    pub aspect_ratio: f32,
}

impl Default for AsciiArtConfig {
    fn default() -> Self {
        Self {
            width: Some(80),
            charset: CharacterSet::Simple,
            colored: false,
            inverted: false,
            aspect_ratio: 0.5, // Chars are roughly 2x taller than wide
        }
    }
}

/// ASCII art generator
pub struct AsciiArtGenerator {
    config: AsciiArtConfig,
}

impl AsciiArtGenerator {
    /// Create a new generator with default config
    pub fn new() -> Self {
        Self {
            config: AsciiArtConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: AsciiArtConfig) -> Self {
        Self { config }
    }

    /// Load image from file and convert to ASCII art
    pub fn from_image_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let img = image::open(path).context("Failed to load image")?;
        self.from_image(&img)
    }

    /// Convert an image to ASCII art
    pub fn from_image(&self, img: &DynamicImage) -> Result<String> {
        let (orig_width, orig_height) = img.dimensions();

        // Calculate target dimensions
        let (target_width, target_height) = if let Some(width) = self.config.width {
            let aspect = (orig_height as f32 / orig_width as f32) * self.config.aspect_ratio;
            let height = (width as f32 * aspect) as u32;
            (width as u32, height)
        } else {
            (
                orig_width,
                (orig_height as f32 * self.config.aspect_ratio) as u32,
            )
        };

        // Resize image
        let img = img.resize_exact(
            target_width,
            target_height,
            image::imageops::FilterType::Lanczos3,
        );

        let mut result = String::new();

        for y in 0..target_height {
            for x in 0..target_width {
                let pixel = img.get_pixel(x, y);
                let brightness = self.calculate_brightness(&pixel);
                let brightness = if self.config.inverted {
                    1.0 - brightness
                } else {
                    brightness
                };

                if self.config.colored {
                    // Add ANSI color code
                    let color = rgb_to_ansi(pixel[0], pixel[1], pixel[2]);
                    result.push_str(&format!("\x1b[38;5;{}m", color));
                }

                let ch = self.config.charset.char_for_brightness(brightness);
                result.push(ch);
            }

            if self.config.colored {
                result.push_str("\x1b[0m"); // Reset color
            }
            result.push('\n');
        }

        Ok(result)
    }

    /// Convert text to ASCII art using figlet-style font
    pub fn from_text(&self, text: &str, font: &AsciiFont) -> Result<String> {
        font.render(text)
    }

    /// Calculate brightness from pixel (0.0 = black, 1.0 = white)
    fn calculate_brightness(&self, pixel: &Rgba<u8>) -> f32 {
        // Use perceived brightness (human eye is more sensitive to green)
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let a = pixel[3] as f32 / 255.0;

        // Weighted average for perceived brightness
        let brightness = (0.299 * r + 0.587 * g + 0.114 * b) * a;
        brightness
    }
}

impl Default for AsciiArtGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert RGB to nearest ANSI 256 color code
fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    // Use 6x6x6 color cube (colors 16-231)
    let r = (r as u32 * 5 / 255) as u8;
    let g = (g as u32 * 5 / 255) as u8;
    let b = (b as u32 * 5 / 255) as u8;
    16 + 36 * r + 6 * g + b
}

/// ASCII font for text rendering
#[derive(Debug, Clone)]
pub struct AsciiFont {
    pub name: &'static str,
    pub height: usize,
    glyphs: fn(char) -> Option<Vec<&'static str>>,
}

impl AsciiFont {
    /// Render text with this font
    pub fn render(&self, text: &str) -> Result<String> {
        let mut lines = vec![String::new(); self.height];

        for ch in text.chars() {
            if let Some(glyph) = (self.glyphs)(ch) {
                for (i, line) in glyph.iter().enumerate() {
                    if i < lines.len() {
                        lines[i].push_str(line);
                    }
                }
            } else {
                // Unknown char, use space
                for line in &mut lines {
                    line.push(' ');
                }
            }
        }

        Ok(lines.join("\n"))
    }
}

/// Standard font
pub const FONT_STANDARD: AsciiFont = AsciiFont {
    name: "Standard",
    height: 6,
    glyphs: standard_glyph,
};

/// Small font
pub const FONT_SMALL: AsciiFont = AsciiFont {
    name: "Small",
    height: 5,
    glyphs: small_glyph,
};

fn standard_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_lowercase() {
        'a' => Some(vec![
            "  ___  ",
            " / _ \\ ",
            "| |_| |",
            "|  _  |",
            "| | | |",
            "|_| |_|",
        ]),
        'b' => Some(vec![
            " ____  ",
            "| __ ) ",
            "|  _ \\ ",
            "| |_) |",
            "|____/ ",
            "       ",
        ]),
        'c' => Some(vec![
            "  ____ ",
            " / ___|",
            "| |    ",
            "| |___ ",
            " \\____|",
            "       ",
        ]),
        'd' => Some(vec![
            " ____  ",
            "|  _ \\ ",
            "| | | |",
            "| |_| |",
            "|____/ ",
            "       ",
        ]),
        'e' => Some(vec![
            " _____ ",
            "| ____|",
            "|  _|  ",
            "| |___ ",
            "|_____|",
            "       ",
        ]),
        'f' => Some(vec![
            " _____ ",
            "|  ___|",
            "| |_   ",
            "|  _|  ",
            "|_|    ",
            "       ",
        ]),
        'g' => Some(vec![
            "  ____ ",
            " / ___|",
            "| |  _ ",
            "| |_| |",
            " \\____|",
            "       ",
        ]),
        'h' => Some(vec![
            " _   _ ",
            "| | | |",
            "| |_| |",
            "|  _  |",
            "|_| |_|",
            "       ",
        ]),
        'i' => Some(vec![
            " ___ ",
            "|_ _|",
            " | | ",
            " | | ",
            "|___|",
            "     ",
        ]),
        'j' => Some(vec![
            "     _ ",
            "    | |",
            " _  | |",
            "| |_| |",
            " \\___/ ",
            "       ",
        ]),
        'k' => Some(vec![
            " _  __",
            "| |/ /",
            "| ' / ",
            "| . \\ ",
            "|_|\\_\\",
            "      ",
        ]),
        'l' => Some(vec![
            " _     ",
            "| |    ",
            "| |    ",
            "| |___ ",
            "|_____|",
            "       ",
        ]),
        'm' => Some(vec![
            " __  __ ",
            "|  \\/  |",
            "| |\\/| |",
            "| |  | |",
            "|_|  |_|",
            "        ",
        ]),
        'n' => Some(vec![
            " _   _ ",
            "| \\ | |",
            "|  \\| |",
            "| |\\  |",
            "|_| \\_|",
            "       ",
        ]),
        'o' => Some(vec![
            "  ___  ",
            " / _ \\ ",
            "| | | |",
            "| |_| |",
            " \\___/ ",
            "       ",
        ]),
        'p' => Some(vec![
            " ____  ",
            "|  _ \\ ",
            "| |_) |",
            "|  __/ ",
            "|_|    ",
            "       ",
        ]),
        'q' => Some(vec![
            "  ___  ",
            " / _ \\ ",
            "| | | |",
            "| |_| |",
            " \\__\\_\\",
            "       ",
        ]),
        'r' => Some(vec![
            " ____  ",
            "|  _ \\ ",
            "| |_) |",
            "|  _ < ",
            "|_| \\_\\",
            "       ",
        ]),
        's' => Some(vec![
            " ____  ",
            "/ ___| ",
            "\\___ \\ ",
            " ___) |",
            "|____/ ",
            "       ",
        ]),
        't' => Some(vec![
            " _____ ",
            "|_   _|",
            "  | |  ",
            "  | |  ",
            "  |_|  ",
            "       ",
        ]),
        'u' => Some(vec![
            " _   _ ",
            "| | | |",
            "| | | |",
            "| |_| |",
            " \\___/ ",
            "       ",
        ]),
        'v' => Some(vec![
            "__     __",
            "\\ \\   / /",
            " \\ \\ / / ",
            "  \\ V /  ",
            "   \\_/   ",
            "         ",
        ]),
        'w' => Some(vec![
            "__        __",
            "\\ \\      / /",
            " \\ \\ /\\ / / ",
            "  \\ V  V /  ",
            "   \\_/\\_/   ",
            "            ",
        ]),
        'x' => Some(vec![
            "__  __",
            "\\ \\/ /",
            " \\  / ",
            " /  \\ ",
            "/_/\\_\\",
            "      ",
        ]),
        'y' => Some(vec![
            "__   __",
            "\\ \\ / /",
            " \\ V / ",
            "  | |  ",
            "  |_|  ",
            "       ",
        ]),
        'z' => Some(vec![
            " _____",
            "|__  /",
            "  / / ",
            " / /_ ",
            "/____|",
            "      ",
        ]),
        '0' => Some(vec![
            "  ___  ",
            " / _ \\ ",
            "| | | |",
            "| |_| |",
            " \\___/ ",
            "       ",
        ]),
        '1' => Some(vec![
            " _ ",
            "/ |",
            "| |",
            "| |",
            "|_|",
            "   ",
        ]),
        '2' => Some(vec![
            " ____  ",
            "|___ \\ ",
            "  __) |",
            " / __/ ",
            "|_____|",
            "       ",
        ]),
        '3' => Some(vec![
            " _____ ",
            "|___ / ",
            "  |_ \\ ",
            " ___) |",
            "|____/ ",
            "       ",
        ]),
        '4' => Some(vec![
            " _  _   ",
            "| || |  ",
            "| || |_ ",
            "|__   _|",
            "   |_|  ",
            "        ",
        ]),
        '5' => Some(vec![
            " ____  ",
            "| ___| ",
            "|___ \\ ",
            " ___) |",
            "|____/ ",
            "       ",
        ]),
        '6' => Some(vec![
            "  __   ",
            " / /_  ",
            "| '_ \\ ",
            "| (_) |",
            " \\___/ ",
            "       ",
        ]),
        '7' => Some(vec![
            " _____ ",
            "|___  |",
            "   / / ",
            "  / /  ",
            " /_/   ",
            "       ",
        ]),
        '8' => Some(vec![
            "  ___  ",
            " ( _ ) ",
            " / _ \\ ",
            "| (_) |",
            " \\___/ ",
            "       ",
        ]),
        '9' => Some(vec![
            "  ___  ",
            " / _ \\ ",
            "| (_) |",
            " \\__, |",
            "   /_/ ",
            "       ",
        ]),
        '!' => Some(vec![
            " _ ",
            "| |",
            "| |",
            "|_|",
            "(_)",
            "   ",
        ]),
        '?' => Some(vec![
            " ___ ",
            "|__ \\",
            "  / /",
            " |_| ",
            " (_) ",
            "     ",
        ]),
        '.' => Some(vec![
            "   ",
            "   ",
            "   ",
            " _ ",
            "(_)",
            "   ",
        ]),
        ',' => Some(vec![
            "   ",
            "   ",
            "   ",
            " _ ",
            "( )",
            "|/ ",
        ]),
        '-' => Some(vec![
            "       ",
            "       ",
            " _____ ",
            "|_____|",
            "       ",
            "       ",
        ]),
        '_' => Some(vec![
            "       ",
            "       ",
            "       ",
            "       ",
            " _____ ",
            "|_____|",
        ]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec![" ? ", " ? ", " ? ", " ? ", " ? ", "   "]), // Unknown char placeholder
    }
}

fn small_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_lowercase() {
        'a' => Some(vec!["     ", " __ _", "/ _` |", "\\__,_|", "     "]),
        'b' => Some(vec![" _   ", "| |_ ", "| '_ \\", "|_.__/", "     "]),
        'c' => Some(vec!["     ", " ___ ", "/ __|", "\\__ \\", "     "]),
        'd' => Some(vec!["    _", " __| |", "/ _` |", "\\__,_|", "     "]),
        'e' => Some(vec!["     ", " ___ ", "/ -_)", "\\___| ", "     "]),
        'f' => Some(vec![" __ ", "/ _|", "| _|", "|_| ", "    "]),
        'g' => Some(vec!["     ", " __ _", "/ _` |", "\\__, |", "|___/"]),
        'h' => Some(vec![" _   ", "| |_ ", "| ' \\", "|_||_|", "     "]),
        'i' => Some(vec![" _ ", "(_)", "| |", "|_|", "   "]),
        'j' => Some(vec!["   _ ", "  (_)", "  | |", " _/ |", "|__/"]),
        'k' => Some(vec![" _  ", "| |_", "| / ", "|_\\_\\", "    "]),
        'l' => Some(vec![" _ ", "| |", "| |", "|_|", "   "]),
        'm' => Some(vec!["      ", " _ __ ", "| '  \\", "|_|_|_|", "      "]),
        'n' => Some(vec!["     ", " _ _ ", "| ' \\", "|_||_|", "     "]),
        'o' => Some(vec!["     ", " ___ ", "/ _ \\", "\\___/", "     "]),
        'p' => Some(vec!["     ", " _ __ ", "| '_ \\", "| .__/", "|_|  "]),
        'q' => Some(vec!["     ", " __ _", "/ _` |", "\\__, |", "   |_|"]),
        'r' => Some(vec!["     ", " _ _ ", "| '_|", "|_|  ", "     "]),
        's' => Some(vec!["    ", " ___", "(_-<", "/__/", "    "]),
        't' => Some(vec![" _  ", "| |_", "|  _|", " \\__|", "    "]),
        'u' => Some(vec!["     ", " _ _ ", "| | |", "|___|", "     "]),
        'v' => Some(vec!["     ", "__  __", "\\ \\/ /", " \\__/ ", "     "]),
        'w' => Some(vec!["        ", "__    __", "\\ \\/\\/ /", " \\_/\\_/ ", "        "]),
        'x' => Some(vec!["    ", "__ __", "\\ \\ /", "/_\\_\\", "    "]),
        'y' => Some(vec!["     ", " _  _", "| || |", " \\_. |", " |__/"]),
        'z' => Some(vec!["    ", " ___", "|_ /", "/__|", "    "]),
        '0' => Some(vec![" ___", "/ _ \\", "| (_) |", "\\___/", "    "]),
        '1' => Some(vec![" _ ", "/ |", "| |", "|_|", "   "]),
        '2' => Some(vec![" ___ ", "|_  )", " / / ", "/___|", "     "]),
        '3' => Some(vec![" ___ ", "|__ \\", " |_ \\", "|___/", "     "]),
        '4' => Some(vec!["  _  ", " | | ", "|_  _|", "  |_| ", "     "]),
        '5' => Some(vec![" ___ ", "| __)", "|__ \\", "|___/", "     "]),
        '6' => Some(vec!["  __ ", " / / ", "| _ \\", "\\___/", "     "]),
        '7' => Some(vec![" ___ ", "|__  |", "  / / ", " /_/  ", "     "]),
        '8' => Some(vec![" ___ ", "( _ )", "/ _ \\", "\\___/", "     "]),
        '9' => Some(vec![" ___ ", "/ _ \\", "\\_, /", " /_/ ", "     "]),
        '!' => Some(vec![" _ ", "| |", "|_|", "(_)", "   "]),
        '?' => Some(vec![" __ ", "|_ )", " |_|", " (_)", "    "]),
        '.' => Some(vec!["  ", "  ", " _", "(_)", "  "]),
        ' ' => Some(vec!["   ", "   ", "   ", "   ", "   "]),
        _ => Some(vec![" ? ", " ? ", " ? ", " ? ", "   "]),
    }
}

/// Built-in Corgi ASCII art collection
pub struct CorgiArt;

impl CorgiArt {
    /// Get all corgi art pieces
    pub fn all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Classic", Self::CLASSIC),
            ("Sitting", Self::SITTING),
            ("Running", Self::RUNNING),
            ("Happy", Self::HAPPY),
            ("Sleepy", Self::SLEEPY),
            ("Pixel (CorgiTerm Mascot)", Self::PIXEL),
        ]
    }

    /// Get random corgi
    pub fn random() -> &'static str {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let arts = vec![
            Self::CLASSIC,
            Self::SITTING,
            Self::RUNNING,
            Self::HAPPY,
            Self::SLEEPY,
            Self::PIXEL,
        ];
        arts[seed % arts.len()]
    }

    pub const CLASSIC: &'static str = r#"
   ／＞　 フ
  | 　_　_|
／` ミ＿xノ
/　　　　 |
/　 ヽ　　ﾉ
│　　|　|　|
/ ￣|　　|　|
| ( ￣ヽ＿_ヽ_)__)
＼二つ
"#;

    pub const SITTING: &'static str = r#"
    __
   /  \
  |    |  /\_/\
  |    | ( o.o )
   \__/   > ^ <
    ||   /|   |\
    ||   \|___|/
   (  )
    ||
   (__)"#;

    pub const RUNNING: &'static str = r#"
     __
    /  \  _/|
   |    |/ /
   |    | /
    \__/ /
     /| |___
    < | |   \
     \|_|___/
       | |
       |_|"#;

    pub const HAPPY: &'static str = r#"
     /\_/\
    ( ^.^ )
   C(")(")
    _|  |_
   /  ||  \
  (__)||(__)"#;

    pub const SLEEPY: &'static str = r#"
     /\_/\
    ( -.- ) zzZ
   C(")(")
    _|  |_
   /  ||  \
  (__)||(__)"#;

    pub const PIXEL: &'static str = r#"
    ████████
  ██░░██░░██
  ██░░██░░██  PIXEL
  ██████████  CorgiTerm Mascot
██░░██████░░██
██░░░░░░░░░░██
██░░██░░██░░██
  ████  ████
  ██      ██
  ▓▓      ▓▓
NES-style Tri-Color Corgi!"#;

    pub const WELCOME: &'static str = r#"
  ╔═══════════════════════════════════════════╗
  ║                                           ║
  ║     ██████╗ ██████╗ ██████╗  ██████╗ ██╗ ║
  ║    ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██║ ║
  ║    ██║     ██║   ██║██████╔╝██║  ███╗██║ ║
  ║    ██║     ██║   ██║██╔══██╗██║   ██║██║ ║
  ║    ╚██████╗╚██████╔╝██║  ██║╚██████╔╝██║ ║
  ║     ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝ ║
  ║                                           ║
  ║          TERM - Terminal Emulator         ║
  ║                                           ║
  ║         Welcome to CorgiTerm! 🐕          ║
  ║     Making terminals friendly & fun       ║
  ║                                           ║
  ╚═══════════════════════════════════════════╝

       ／＞　 フ
      | 　_　_|     Type 'help' to get started
    ／` ミ＿xノ     Press Ctrl+Space for AI help
   /　　　　 |      Tools → ASCII Art Generator
  /　 ヽ　　ﾉ
  │　　|　|　|
"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_sets() {
        assert_eq!(CharacterSet::Simple.chars().len(), 9);
        assert_eq!(CharacterSet::Simple.char_for_brightness(0.0), ' ');
        assert_eq!(CharacterSet::Simple.char_for_brightness(1.0), '@');
    }

    #[test]
    fn test_brightness_mapping() {
        let charset = CharacterSet::Simple;
        assert_eq!(charset.char_for_brightness(0.0), ' ');
        // 0.5 brightness maps to index 4 of 9 chars (0-8), which is '='
        assert_eq!(charset.char_for_brightness(0.5), '=');
        assert_eq!(charset.char_for_brightness(1.0), '@');
    }

    #[test]
    fn test_rgb_to_ansi() {
        assert_eq!(rgb_to_ansi(0, 0, 0), 16); // Black
        assert_eq!(rgb_to_ansi(255, 255, 255), 231); // White
    }

    #[test]
    fn test_corgi_collection() {
        let all = CorgiArt::all();
        assert_eq!(all.len(), 6);
        assert!(CorgiArt::CLASSIC.contains("フ"));
    }

    #[test]
    fn test_random_corgi() {
        let art = CorgiArt::random();
        assert!(!art.is_empty());
    }
}
