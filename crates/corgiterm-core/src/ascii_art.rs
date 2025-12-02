//! # ASCII Art Generator
//!
//! Convert images and text to ASCII art for display in terminals.
//!
//! Features:
//! - Image to ASCII conversion with brightness mapping
//! - Multiple character sets (simple, detailed, blocks, braille)
//! - ANSI color support
//! - Edge detection for outline art
//! - Text to ASCII art with multiple figlet-style fonts
//! - Built-in Corgi art collection 🐕
//! - Template gallery with ASCII art patterns
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
use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Rgba};
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

/// Image processing filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFilter {
    /// No filter - use original brightness
    #[default]
    None,
    /// Sobel edge detection
    EdgeDetect,
    /// High contrast
    HighContrast,
    /// Posterize (reduce color levels)
    Posterize,
    /// Dithering for better gradients
    Dither,
}

impl ImageFilter {
    /// Get all available filters
    pub fn all() -> &'static [ImageFilter] {
        &[
            ImageFilter::None,
            ImageFilter::EdgeDetect,
            ImageFilter::HighContrast,
            ImageFilter::Posterize,
            ImageFilter::Dither,
        ]
    }

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            ImageFilter::None => "None",
            ImageFilter::EdgeDetect => "Edge Detect",
            ImageFilter::HighContrast => "High Contrast",
            ImageFilter::Posterize => "Posterize",
            ImageFilter::Dither => "Dither",
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
    /// Image filter to apply
    pub filter: ImageFilter,
    /// Brightness adjustment (-100 to 100)
    pub brightness: i32,
    /// Contrast adjustment (-100 to 100)
    pub contrast: i32,
}

impl Default for AsciiArtConfig {
    fn default() -> Self {
        Self {
            width: Some(80),
            charset: CharacterSet::Simple,
            colored: false,
            inverted: false,
            aspect_ratio: 0.5, // Chars are roughly 2x taller than wide
            filter: ImageFilter::None,
            brightness: 0,
            contrast: 0,
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
            (width as u32, height.max(1))
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

        // Apply filter if needed
        let processed = self.apply_filter(&img);

        let mut result = String::new();

        for y in 0..target_height {
            for x in 0..target_width {
                let pixel = img.get_pixel(x, y);
                let brightness = if let Some(ref gray) = processed {
                    // Use filtered grayscale
                    gray.get_pixel(x, y)[0] as f32 / 255.0
                } else {
                    self.calculate_brightness(&pixel)
                };

                // Apply brightness/contrast adjustments
                let brightness = self.adjust_brightness_contrast(brightness);

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

    /// Apply image filter
    fn apply_filter(&self, img: &DynamicImage) -> Option<GrayImage> {
        match self.config.filter {
            ImageFilter::None => None,
            ImageFilter::EdgeDetect => Some(self.sobel_edge_detect(img)),
            ImageFilter::HighContrast => Some(self.high_contrast(img)),
            ImageFilter::Posterize => Some(self.posterize(img)),
            ImageFilter::Dither => Some(self.floyd_steinberg_dither(img)),
        }
    }

    /// Sobel edge detection
    fn sobel_edge_detect(&self, img: &DynamicImage) -> GrayImage {
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();
        let mut output: GrayImage = ImageBuffer::new(width, height);

        // Sobel kernels
        let gx: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        let gy: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut sum_x: i32 = 0;
                let mut sum_y: i32 = 0;

                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = gray.get_pixel(x + kx - 1, y + ky - 1)[0] as i32;
                        sum_x += px * gx[ky as usize][kx as usize];
                        sum_y += px * gy[ky as usize][kx as usize];
                    }
                }

                let magnitude = ((sum_x * sum_x + sum_y * sum_y) as f32).sqrt() as u8;
                output.put_pixel(x, y, Luma([magnitude]));
            }
        }

        output
    }

    /// High contrast filter
    fn high_contrast(&self, img: &DynamicImage) -> GrayImage {
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();
        let mut output: GrayImage = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let pixel = gray.get_pixel(x, y)[0];
                // Apply S-curve for contrast
                let normalized = pixel as f32 / 255.0;
                let contrasted = if normalized < 0.5 {
                    2.0 * normalized * normalized
                } else {
                    1.0 - 2.0 * (1.0 - normalized) * (1.0 - normalized)
                };
                output.put_pixel(x, y, Luma([(contrasted * 255.0) as u8]));
            }
        }

        output
    }

    /// Posterize filter (reduce levels)
    fn posterize(&self, img: &DynamicImage) -> GrayImage {
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();
        let mut output: GrayImage = ImageBuffer::new(width, height);
        let levels = 4; // Number of gray levels

        for y in 0..height {
            for x in 0..width {
                let pixel = gray.get_pixel(x, y)[0];
                let level = (pixel as u32 * levels / 256) as u8;
                let posterized = level * 255 / (levels as u8 - 1);
                output.put_pixel(x, y, Luma([posterized]));
            }
        }

        output
    }

    /// Floyd-Steinberg dithering
    fn floyd_steinberg_dither(&self, img: &DynamicImage) -> GrayImage {
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();
        let mut output: Vec<f32> = gray.pixels().map(|p| p[0] as f32).collect();

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let old_pixel = output[idx];
                let new_pixel = if old_pixel > 127.0 { 255.0 } else { 0.0 };
                output[idx] = new_pixel;
                let error = old_pixel - new_pixel;

                // Distribute error to neighbors
                if x + 1 < width {
                    output[idx + 1] += error * 7.0 / 16.0;
                }
                if y + 1 < height {
                    if x > 0 {
                        output[idx + width as usize - 1] += error * 3.0 / 16.0;
                    }
                    output[idx + width as usize] += error * 5.0 / 16.0;
                    if x + 1 < width {
                        output[idx + width as usize + 1] += error * 1.0 / 16.0;
                    }
                }
            }
        }

        let mut result: GrayImage = ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                result.put_pixel(x, y, Luma([output[idx].clamp(0.0, 255.0) as u8]));
            }
        }

        result
    }

    /// Apply brightness and contrast adjustments
    fn adjust_brightness_contrast(&self, brightness: f32) -> f32 {
        let mut value = brightness;

        // Apply brightness (-100 to 100 maps to -0.5 to 0.5)
        if self.config.brightness != 0 {
            value += self.config.brightness as f32 / 200.0;
        }

        // Apply contrast (-100 to 100 maps to 0.5x to 2x)
        if self.config.contrast != 0 {
            let contrast_factor = (self.config.contrast as f32 + 100.0) / 100.0;
            value = (value - 0.5) * contrast_factor + 0.5;
        }

        value.clamp(0.0, 1.0)
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
        (0.299 * r + 0.587 * g + 0.114 * b) * a
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

/// Block font (big bold letters)
pub const FONT_BLOCK: AsciiFont = AsciiFont {
    name: "Block",
    height: 5,
    glyphs: block_glyph,
};

/// Banner font (simple caps)
pub const FONT_BANNER: AsciiFont = AsciiFont {
    name: "Banner",
    height: 3,
    glyphs: banner_glyph,
};

/// Mini font (tiny 3-line)
pub const FONT_MINI: AsciiFont = AsciiFont {
    name: "Mini",
    height: 3,
    glyphs: mini_glyph,
};

/// Shadow font (3D effect)
pub const FONT_SHADOW: AsciiFont = AsciiFont {
    name: "Shadow",
    height: 4,
    glyphs: shadow_glyph,
};

/// Slant font (classic diagonal)
pub const FONT_SLANT: AsciiFont = AsciiFont {
    name: "Slant",
    height: 6,
    glyphs: slant_glyph,
};

/// Big font (large bold letters)
pub const FONT_BIG: AsciiFont = AsciiFont {
    name: "Big",
    height: 8,
    glyphs: big_glyph,
};

/// Doom font (classic game style)
pub const FONT_DOOM: AsciiFont = AsciiFont {
    name: "Doom",
    height: 8,
    glyphs: doom_glyph,
};

/// Script font (cursive style)
pub const FONT_SCRIPT: AsciiFont = AsciiFont {
    name: "Script",
    height: 5,
    glyphs: script_glyph,
};

/// Digital font (7-segment LED display)
pub const FONT_DIGITAL: AsciiFont = AsciiFont {
    name: "Digital",
    height: 5,
    glyphs: digital_glyph,
};

/// 3D font (perspective effect)
pub const FONT_3D: AsciiFont = AsciiFont {
    name: "3D",
    height: 7,
    glyphs: font_3d_glyph,
};

/// Bubble font (rounded letters)
pub const FONT_BUBBLE: AsciiFont = AsciiFont {
    name: "Bubble",
    height: 4,
    glyphs: bubble_glyph,
};

/// Graffiti font (street art style)
pub const FONT_GRAFFITI: AsciiFont = AsciiFont {
    name: "Graffiti",
    height: 6,
    glyphs: graffiti_glyph,
};

/// Gothic font (old English style)
pub const FONT_GOTHIC: AsciiFont = AsciiFont {
    name: "Gothic",
    height: 6,
    glyphs: gothic_glyph,
};

/// Lean font (italicized/leaning)
pub const FONT_LEAN: AsciiFont = AsciiFont {
    name: "Lean",
    height: 5,
    glyphs: lean_glyph,
};

/// Isometric font (3D isometric view)
pub const FONT_ISOMETRIC: AsciiFont = AsciiFont {
    name: "Isometric",
    height: 7,
    glyphs: isometric_glyph,
};

/// Starwars font (sci-fi style)
pub const FONT_STARWARS: AsciiFont = AsciiFont {
    name: "Star Wars",
    height: 6,
    glyphs: starwars_glyph,
};

/// Cyberlarge font (modern cyber aesthetic)
pub const FONT_CYBERLARGE: AsciiFont = AsciiFont {
    name: "Cyberlarge",
    height: 6,
    glyphs: cyberlarge_glyph,
};

/// Alligator font (cool teeth style)
pub const FONT_ALLIGATOR: AsciiFont = AsciiFont {
    name: "Alligator",
    height: 5,
    glyphs: alligator_glyph,
};

/// Roman font (serif style)
pub const FONT_ROMAN: AsciiFont = AsciiFont {
    name: "Roman",
    height: 6,
    glyphs: roman_glyph,
};

/// Thick font (extra bold)
pub const FONT_THICK: AsciiFont = AsciiFont {
    name: "Thick",
    height: 5,
    glyphs: thick_glyph,
};

/// Ogre font (monster style)
pub const FONT_OGRE: AsciiFont = AsciiFont {
    name: "Ogre",
    height: 7,
    glyphs: ogre_glyph,
};

/// Ivrit font (Hebrew-inspired)
pub const FONT_IVRIT: AsciiFont = AsciiFont {
    name: "Ivrit",
    height: 5,
    glyphs: ivrit_glyph,
};

/// Rectangles font (box drawing)
pub const FONT_RECTANGLES: AsciiFont = AsciiFont {
    name: "Rectangles",
    height: 5,
    glyphs: rectangles_glyph,
};

/// Get all available fonts
pub fn all_fonts() -> &'static [&'static AsciiFont] {
    &[
        &FONT_STANDARD,
        &FONT_SMALL,
        &FONT_BLOCK,
        &FONT_BANNER,
        &FONT_MINI,
        &FONT_SHADOW,
        &FONT_SLANT,
        &FONT_BIG,
        &FONT_DOOM,
        &FONT_SCRIPT,
        &FONT_DIGITAL,
        &FONT_3D,
        &FONT_BUBBLE,
        &FONT_GRAFFITI,
        &FONT_GOTHIC,
        &FONT_LEAN,
        &FONT_ISOMETRIC,
        &FONT_STARWARS,
        &FONT_CYBERLARGE,
        &FONT_ALLIGATOR,
        &FONT_ROMAN,
        &FONT_THICK,
        &FONT_OGRE,
        &FONT_IVRIT,
        &FONT_RECTANGLES,
    ]
}

fn standard_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_lowercase() {
        'a' => Some(vec![
            "  ___  ", " / _ \\ ", "| |_| |", "|  _  |", "| | | |", "|_| |_|",
        ]),
        'b' => Some(vec![
            " ____  ", "| __ ) ", "|  _ \\ ", "| |_) |", "|____/ ", "       ",
        ]),
        'c' => Some(vec![
            "  ____ ", " / ___|", "| |    ", "| |___ ", " \\____|", "       ",
        ]),
        'd' => Some(vec![
            " ____  ", "|  _ \\ ", "| | | |", "| |_| |", "|____/ ", "       ",
        ]),
        'e' => Some(vec![
            " _____ ", "| ____|", "|  _|  ", "| |___ ", "|_____|", "       ",
        ]),
        'f' => Some(vec![
            " _____ ", "|  ___|", "| |_   ", "|  _|  ", "|_|    ", "       ",
        ]),
        'g' => Some(vec![
            "  ____ ", " / ___|", "| |  _ ", "| |_| |", " \\____|", "       ",
        ]),
        'h' => Some(vec![
            " _   _ ", "| | | |", "| |_| |", "|  _  |", "|_| |_|", "       ",
        ]),
        'i' => Some(vec![" ___ ", "|_ _|", " | | ", " | | ", "|___|", "     "]),
        'j' => Some(vec![
            "     _ ", "    | |", " _  | |", "| |_| |", " \\___/ ", "       ",
        ]),
        'k' => Some(vec![
            " _  __", "| |/ /", "| ' / ", "| . \\ ", "|_|\\_\\", "      ",
        ]),
        'l' => Some(vec![
            " _     ", "| |    ", "| |    ", "| |___ ", "|_____|", "       ",
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
            " _   _ ", "| \\ | |", "|  \\| |", "| |\\  |", "|_| \\_|", "       ",
        ]),
        'o' => Some(vec![
            "  ___  ", " / _ \\ ", "| | | |", "| |_| |", " \\___/ ", "       ",
        ]),
        'p' => Some(vec![
            " ____  ", "|  _ \\ ", "| |_) |", "|  __/ ", "|_|    ", "       ",
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
            " _____ ", "|_   _|", "  | |  ", "  | |  ", "  |_|  ", "       ",
        ]),
        'u' => Some(vec![
            " _   _ ", "| | | |", "| | | |", "| |_| |", " \\___/ ", "       ",
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
            "__  __", "\\ \\/ /", " \\  / ", " /  \\ ", "/_/\\_\\", "      ",
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
            " _____", "|__  /", "  / / ", " / /_ ", "/____|", "      ",
        ]),
        '0' => Some(vec![
            "  ___  ", " / _ \\ ", "| | | |", "| |_| |", " \\___/ ", "       ",
        ]),
        '1' => Some(vec![" _ ", "/ |", "| |", "| |", "|_|", "   "]),
        '2' => Some(vec![
            " ____  ", "|___ \\ ", "  __) |", " / __/ ", "|_____|", "       ",
        ]),
        '3' => Some(vec![
            " _____ ", "|___ / ", "  |_ \\ ", " ___) |", "|____/ ", "       ",
        ]),
        '4' => Some(vec![
            " _  _   ", "| || |  ", "| || |_ ", "|__   _|", "   |_|  ", "        ",
        ]),
        '5' => Some(vec![
            " ____  ", "| ___| ", "|___ \\ ", " ___) |", "|____/ ", "       ",
        ]),
        '6' => Some(vec![
            "  __   ", " / /_  ", "| '_ \\ ", "| (_) |", " \\___/ ", "       ",
        ]),
        '7' => Some(vec![
            " _____ ", "|___  |", "   / / ", "  / /  ", " /_/   ", "       ",
        ]),
        '8' => Some(vec![
            "  ___  ", " ( _ ) ", " / _ \\ ", "| (_) |", " \\___/ ", "       ",
        ]),
        '9' => Some(vec![
            "  ___  ", " / _ \\ ", "| (_) |", " \\__, |", "   /_/ ", "       ",
        ]),
        '!' => Some(vec![" _ ", "| |", "| |", "|_|", "(_)", "   "]),
        '?' => Some(vec![" ___ ", "|__ \\", "  / /", " |_| ", " (_) ", "     "]),
        '.' => Some(vec!["   ", "   ", "   ", " _ ", "(_)", "   "]),
        ',' => Some(vec!["   ", "   ", "   ", " _ ", "( )", "|/ "]),
        '-' => Some(vec![
            "       ", "       ", " _____ ", "|_____|", "       ", "       ",
        ]),
        '_' => Some(vec![
            "       ", "       ", "       ", "       ", " _____ ", "|_____|",
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
        'w' => Some(vec![
            "        ",
            "__    __",
            "\\ \\/\\/ /",
            " \\_/\\_/ ",
            "        ",
        ]),
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

fn block_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec!["█████", "█   █", "█████", "█   █", "█   █"]),
        'B' => Some(vec!["████ ", "█   █", "████ ", "█   █", "████ "]),
        'C' => Some(vec!["█████", "█    ", "█    ", "█    ", "█████"]),
        'D' => Some(vec!["████ ", "█   █", "█   █", "█   █", "████ "]),
        'E' => Some(vec!["█████", "█    ", "███  ", "█    ", "█████"]),
        'F' => Some(vec!["█████", "█    ", "███  ", "█    ", "█    "]),
        'G' => Some(vec!["█████", "█    ", "█ ███", "█   █", "█████"]),
        'H' => Some(vec!["█   █", "█   █", "█████", "█   █", "█   █"]),
        'I' => Some(vec!["█████", "  █  ", "  █  ", "  █  ", "█████"]),
        'J' => Some(vec!["█████", "    █", "    █", "█   █", "█████"]),
        'K' => Some(vec!["█   █", "█  █ ", "███  ", "█  █ ", "█   █"]),
        'L' => Some(vec!["█    ", "█    ", "█    ", "█    ", "█████"]),
        'M' => Some(vec!["█   █", "██ ██", "█ █ █", "█   █", "█   █"]),
        'N' => Some(vec!["█   █", "██  █", "█ █ █", "█  ██", "█   █"]),
        'O' => Some(vec!["█████", "█   █", "█   █", "█   █", "█████"]),
        'P' => Some(vec!["█████", "█   █", "█████", "█    ", "█    "]),
        'Q' => Some(vec!["█████", "█   █", "█   █", "█  █ ", "███ █"]),
        'R' => Some(vec!["█████", "█   █", "████ ", "█  █ ", "█   █"]),
        'S' => Some(vec!["█████", "█    ", "█████", "    █", "█████"]),
        'T' => Some(vec!["█████", "  █  ", "  █  ", "  █  ", "  █  "]),
        'U' => Some(vec!["█   █", "█   █", "█   █", "█   █", "█████"]),
        'V' => Some(vec!["█   █", "█   █", "█   █", " █ █ ", "  █  "]),
        'W' => Some(vec!["█   █", "█   █", "█ █ █", "██ ██", "█   █"]),
        'X' => Some(vec!["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"]),
        'Y' => Some(vec!["█   █", " █ █ ", "  █  ", "  █  ", "  █  "]),
        'Z' => Some(vec!["█████", "   █ ", "  █  ", " █   ", "█████"]),
        '0' => Some(vec!["█████", "█  ██", "█ █ █", "██  █", "█████"]),
        '1' => Some(vec![" ██  ", "  █  ", "  █  ", "  █  ", "█████"]),
        '2' => Some(vec!["█████", "    █", "█████", "█    ", "█████"]),
        '3' => Some(vec!["█████", "    █", " ████", "    █", "█████"]),
        '4' => Some(vec!["█   █", "█   █", "█████", "    █", "    █"]),
        '5' => Some(vec!["█████", "█    ", "█████", "    █", "█████"]),
        '6' => Some(vec!["█████", "█    ", "█████", "█   █", "█████"]),
        '7' => Some(vec!["█████", "    █", "   █ ", "  █  ", "  █  "]),
        '8' => Some(vec!["█████", "█   █", "█████", "█   █", "█████"]),
        '9' => Some(vec!["█████", "█   █", "█████", "    █", "█████"]),
        '!' => Some(vec!["  █  ", "  █  ", "  █  ", "     ", "  █  "]),
        '?' => Some(vec!["█████", "    █", "  ██ ", "     ", "  █  "]),
        '.' => Some(vec!["     ", "     ", "     ", "     ", "  █  "]),
        ' ' => Some(vec!["     ", "     ", "     ", "     ", "     "]),
        _ => Some(vec!["█████", "█   █", "█   █", "█   █", "█████"]),
    }
}

fn banner_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![" /\\ ", "/--\\", "    "]),
        'B' => Some(vec!["|-\\", "|-/", "   "]),
        'C' => Some(vec!["/-", "\\-", "  "]),
        'D' => Some(vec!["/-\\", "\\-/", "   "]),
        'E' => Some(vec!["[-", "[-", "  "]),
        'F' => Some(vec!["[-", "[", " "]),
        'G' => Some(vec!["/-\\", "\\_]", "   "]),
        'H' => Some(vec!["|-|", "|-|", "   "]),
        'I' => Some(vec!["|", "|", " "]),
        'J' => Some(vec![" |", "\\_|", "  "]),
        'K' => Some(vec!["|/", "|\\", "  "]),
        'L' => Some(vec!["|", "L", " "]),
        'M' => Some(vec!["/\\/\\", "    ", "    "]),
        'N' => Some(vec!["|\\|", "| |", "   "]),
        'O' => Some(vec!["/-\\", "\\-/", "   "]),
        'P' => Some(vec!["[-\\", "[", "  "]),
        'Q' => Some(vec!["/-\\", "\\_\\", "   "]),
        'R' => Some(vec!["[-\\", "[\\", "  "]),
        'S' => Some(vec!["/-", "-\\", "  "]),
        'T' => Some(vec!["-|-", " | ", "   "]),
        'U' => Some(vec!["| |", "\\_/", "   "]),
        'V' => Some(vec!["\\ /", " V ", "   "]),
        'W' => Some(vec!["\\ /\\ /", " V  V ", "      "]),
        'X' => Some(vec!["\\/'", "/\\ ", "   "]),
        'Y' => Some(vec!["\\ /", " Y ", "   "]),
        'Z' => Some(vec!["--/", "/-", "  "]),
        ' ' => Some(vec!["  ", "  ", "  "]),
        _ => Some(vec!["?", "?", " "]),
    }
}

fn mini_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec!["_._", "|-|", "   "]),
        'B' => Some(vec!["[-,", "[-'", "   "]),
        'C' => Some(vec![",-", "`-", "  "]),
        'D' => Some(vec!["[-,", "[-'", "   "]),
        'E' => Some(vec!["[-", "[-", "  "]),
        'F' => Some(vec!["[-", "|", " "]),
        'G' => Some(vec![",-", "`]", "  "]),
        'H' => Some(vec!["|-|", "|-|", "   "]),
        'I' => Some(vec!["i", "|", " "]),
        'J' => Some(vec!["_|", "_|", "  "]),
        'K' => Some(vec!["|<", "|>", "  "]),
        'L' => Some(vec!["|", "L", " "]),
        'M' => Some(vec!["/v\\", "   ", "   "]),
        'N' => Some(vec!["|\\|", "| |", "   "]),
        'O' => Some(vec![",-,", "`-'", "   "]),
        'P' => Some(vec!["[-,", "|", "  "]),
        'Q' => Some(vec!["o", "\\", " "]),
        'R' => Some(vec!["[-,", "|\\", "  "]),
        'S' => Some(vec!["_,", "_'", "  "]),
        'T' => Some(vec!["-+-", " | ", "   "]),
        'U' => Some(vec!["| |", "`-'", "   "]),
        'V' => Some(vec!["\\ /", " v ", "   "]),
        'W' => Some(vec!["\\_/", " v ", "   "]),
        'X' => Some(vec!["><", "><", "  "]),
        'Y' => Some(vec!["\\/", "|", " "]),
        'Z' => Some(vec!["_/", "/_", "  "]),
        ' ' => Some(vec!["  ", "  ", "  "]),
        _ => Some(vec!["?", "?", " "]),
    }
}

fn shadow_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![" ▄▄▄ ", "█   █", "█▀▀▀█", "█   █"]),
        'B' => Some(vec!["█▀▀▀▄", "█▀▀▀▄", "█▄▄▄▀", "     "]),
        'C' => Some(vec![" ▄▀▀▀", "█    ", " ▀▄▄▄", "     "]),
        'D' => Some(vec!["█▀▀▀▄", "█   █", "█▄▄▄▀", "     "]),
        'E' => Some(vec!["█▀▀▀▀", "█▀▀▀ ", "█▄▄▄▄", "     "]),
        'F' => Some(vec!["█▀▀▀▀", "█▀▀▀ ", "█    ", "     "]),
        'G' => Some(vec![" ▄▀▀▀", "█  ▀█", " ▀▄▄█", "     "]),
        'H' => Some(vec!["█   █", "█▀▀▀█", "█   █", "     "]),
        'I' => Some(vec!["▀█▀", " █ ", "▄█▄", "   "]),
        'J' => Some(vec!["   █", "   █", "▀▄▄█", "    "]),
        'K' => Some(vec!["█  ▄▀", "█▀▀  ", "█  ▀▄", "     "]),
        'L' => Some(vec!["█    ", "█    ", "█▄▄▄▄", "     "]),
        'M' => Some(vec!["█▄ ▄█", "█ ▀ █", "█   █", "     "]),
        'N' => Some(vec!["█▄  █", "█ ▀▄█", "█   █", "     "]),
        'O' => Some(vec![" ▄▀▀▄ ", "█    █", " ▀▄▄▀ ", "      "]),
        'P' => Some(vec!["█▀▀▀▄", "█▄▄▄▀", "█    ", "     "]),
        'Q' => Some(vec![" ▄▀▀▄ ", "█    █", " ▀▄▄▀▄", "      "]),
        'R' => Some(vec!["█▀▀▀▄", "█▄▄▄▀", "█   █", "     "]),
        'S' => Some(vec![" ▄▀▀▄", "▀▄▄▄ ", "▄▄▄▀ ", "     "]),
        'T' => Some(vec!["▀▀█▀▀", "  █  ", "  █  ", "     "]),
        'U' => Some(vec!["█   █", "█   █", " ▀▀▀ ", "     "]),
        'V' => Some(vec!["█   █", " █ █ ", "  ▀  ", "     "]),
        'W' => Some(vec!["█   █", "█ ▄ █", " ▀ ▀ ", "     "]),
        'X' => Some(vec!["█   █", " ▀▄▀ ", "█   █", "     "]),
        'Y' => Some(vec!["█   █", " ▀▄▀ ", "  █  ", "     "]),
        'Z' => Some(vec!["▀▀▀▀█", " ▄▄▀ ", "█▄▄▄▄", "     "]),
        ' ' => Some(vec!["    ", "    ", "    ", "    "]),
        _ => Some(vec!["▄▄▄", "███", "▀▀▀", "   "]),
    }
}

// ============================================
// NEW FONTS - Added for enhanced ASCII art
// ============================================

fn slant_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "    ___   ",
            "   /   |  ",
            "  / /| |  ",
            " / ___ |  ",
            "/_/  |_|  ",
            "          ",
        ]),
        'B' => Some(vec![
            "    ____ ",
            "   / __ )",
            "  / __  |",
            " / /_/ / ",
            "/_____/  ",
            "         ",
        ]),
        'C' => Some(vec![
            "   ______",
            "  / ____/",
            " / /     ",
            "/ /___   ",
            "\\____/   ",
            "         ",
        ]),
        'D' => Some(vec![
            "    ____ ",
            "   / __ \\",
            "  / / / /",
            " / /_/ / ",
            "/_____/  ",
            "         ",
        ]),
        'E' => Some(vec![
            "    ______",
            "   / ____/",
            "  / __/   ",
            " / /___   ",
            "/_____/   ",
            "          ",
        ]),
        'F' => Some(vec![
            "    ______",
            "   / ____/",
            "  / /_    ",
            " / __/    ",
            "/_/       ",
            "          ",
        ]),
        'G' => Some(vec![
            "   ______",
            "  / ____/",
            " / / __  ",
            "/ /_/ /  ",
            "\\____/   ",
            "         ",
        ]),
        'H' => Some(vec![
            "    __  __",
            "   / / / /",
            "  / /_/ / ",
            " / __  /  ",
            "/_/ /_/   ",
            "          ",
        ]),
        'I' => Some(vec![
            "    ____", "   /  _/", "   / /  ", " _/ /   ", "/___/   ", "        ",
        ]),
        'J' => Some(vec![
            "       __",
            "      / /",
            "  __ / / ",
            " / /_/ /  ",
            " \\____/  ",
            "         ",
        ]),
        'K' => Some(vec![
            "    __ __",
            "   / //_/",
            "  / ,<   ",
            " / /| |  ",
            "/_/ |_|  ",
            "         ",
        ]),
        'L' => Some(vec![
            "    __ ", "   / / ", "  / /  ", " / /___", "/_____/", "       ",
        ]),
        'M' => Some(vec![
            "    __  ___",
            "   /  |/  /",
            "  / /|_/ / ",
            " / /  / /  ",
            "/_/  /_/   ",
            "           ",
        ]),
        'N' => Some(vec![
            "    _   __",
            "   / | / /",
            "  /  |/ / ",
            " / /|  /  ",
            "/_/ |_/   ",
            "          ",
        ]),
        'O' => Some(vec![
            "   ____ ",
            "  / __ \\",
            " / / / /",
            "/ /_/ / ",
            "\\____/  ",
            "        ",
        ]),
        'P' => Some(vec![
            "    ____ ",
            "   / __ \\",
            "  / /_/ /",
            " / ____/ ",
            "/_/      ",
            "         ",
        ]),
        'Q' => Some(vec![
            "   ____ ",
            "  / __ \\",
            " / / / /",
            "/ /_/ / ",
            "\\___\\_\\ ",
            "        ",
        ]),
        'R' => Some(vec![
            "    ____ ",
            "   / __ \\",
            "  / /_/ /",
            " / _, _/ ",
            "/_/ |_|  ",
            "         ",
        ]),
        'S' => Some(vec![
            "   _____",
            "  / ___/",
            "  \\__ \\ ",
            " ___/ / ",
            "/____/  ",
            "        ",
        ]),
        'T' => Some(vec![
            "  ______", " /_  __/", "  / /   ", " / /    ", "/_/     ", "        ",
        ]),
        'U' => Some(vec![
            "   __  __",
            "  / / / /",
            " / / / / ",
            "/ /_/ /  ",
            "\\____/   ",
            "         ",
        ]),
        'V' => Some(vec![
            "   _    __",
            "  | |  / /",
            "  | | / / ",
            "  | |/ /  ",
            "  |___/   ",
            "          ",
        ]),
        'W' => Some(vec![
            "  _       __",
            " | |     / /",
            " | | /| / / ",
            " | |/ |/ /  ",
            " |__/|__/   ",
            "            ",
        ]),
        'X' => Some(vec![
            "   _  __", "  | |/ /", "  |   / ", " /   |  ", "/_/|_|  ", "        ",
        ]),
        'Y' => Some(vec![
            "__  __", "\\ \\/ /", " \\  / ", " / /  ", "/_/   ", "      ",
        ]),
        'Z' => Some(vec![
            "  _____", " /__  /", "   / / ", "  / /__", " /____/", "       ",
        ]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    ", "    "]),
    }
}

fn big_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "          ",
            "    /\\    ",
            "   /  \\   ",
            "  / /\\ \\  ",
            " / ____ \\ ",
            "/_/    \\_\\",
            "          ",
            "          ",
        ]),
        'B' => Some(vec![
            " ____  ", "|  _ \\ ", "| |_) |", "|  _ < ", "| |_) |", "|____/ ", "       ", "       ",
        ]),
        'C' => Some(vec![
            "  _____ ",
            " / ____|",
            "| |     ",
            "| |     ",
            "| |____ ",
            " \\_____|",
            "        ",
            "        ",
        ]),
        'D' => Some(vec![
            " _____  ",
            "|  __ \\ ",
            "| |  | |",
            "| |  | |",
            "| |__| |",
            "|_____/ ",
            "        ",
            "        ",
        ]),
        'E' => Some(vec![
            " ______ ", "|  ____|", "| |__   ", "|  __|  ", "| |____ ", "|______|", "        ",
            "        ",
        ]),
        'F' => Some(vec![
            " ______ ", "|  ____|", "| |__   ", "|  __|  ", "| |     ", "|_|     ", "        ",
            "        ",
        ]),
        'G' => Some(vec![
            "  _____ ",
            " / ____|",
            "| |  __ ",
            "| | |_ |",
            "| |__| |",
            " \\_____|",
            "        ",
            "        ",
        ]),
        'H' => Some(vec![
            " _    _ ", "| |  | |", "| |__| |", "|  __  |", "| |  | |", "|_|  |_|", "        ",
            "        ",
        ]),
        'I' => Some(vec![
            " _____ ", "|_   _|", "  | |  ", "  | |  ", " _| |_ ", "|_____|", "       ", "       ",
        ]),
        'J' => Some(vec![
            "      _ ",
            "     | |",
            "     | |",
            " _   | |",
            "| |__| |",
            " \\____/ ",
            "        ",
            "        ",
        ]),
        'K' => Some(vec![
            " _  __", "| |/ /", "| ' / ", "|  <  ", "| . \\ ", "|_|\\_\\", "      ", "      ",
        ]),
        'L' => Some(vec![
            " _      ", "| |     ", "| |     ", "| |     ", "| |____ ", "|______|", "        ",
            "        ",
        ]),
        'M' => Some(vec![
            " __  __ ",
            "|  \\/  |",
            "| \\  / |",
            "| |\\/| |",
            "| |  | |",
            "|_|  |_|",
            "        ",
            "        ",
        ]),
        'N' => Some(vec![
            " _   _ ", "| \\ | |", "|  \\| |", "| . ` |", "| |\\  |", "|_| \\_|", "       ",
            "       ",
        ]),
        'O' => Some(vec![
            "  ____  ",
            " / __ \\ ",
            "| |  | |",
            "| |  | |",
            "| |__| |",
            " \\____/ ",
            "        ",
            "        ",
        ]),
        'P' => Some(vec![
            " _____  ",
            "|  __ \\ ",
            "| |__) |",
            "|  ___/ ",
            "| |     ",
            "|_|     ",
            "        ",
            "        ",
        ]),
        'Q' => Some(vec![
            "  ____  ",
            " / __ \\ ",
            "| |  | |",
            "| |  | |",
            "| |__| |",
            " \\___\\_\\",
            "        ",
            "        ",
        ]),
        'R' => Some(vec![
            " _____  ",
            "|  __ \\ ",
            "| |__) |",
            "|  _  / ",
            "| | \\ \\ ",
            "|_|  \\_\\",
            "        ",
            "        ",
        ]),
        'S' => Some(vec![
            "  _____ ",
            " / ____|",
            "| (___  ",
            " \\___ \\ ",
            " ____) |",
            "|_____/ ",
            "        ",
            "        ",
        ]),
        'T' => Some(vec![
            " _______ ",
            "|__   __|",
            "   | |   ",
            "   | |   ",
            "   | |   ",
            "   |_|   ",
            "         ",
            "         ",
        ]),
        'U' => Some(vec![
            " _    _ ",
            "| |  | |",
            "| |  | |",
            "| |  | |",
            "| |__| |",
            " \\____/ ",
            "        ",
            "        ",
        ]),
        'V' => Some(vec![
            "__      __",
            "\\ \\    / /",
            " \\ \\  / / ",
            "  \\ \\/ /  ",
            "   \\  /   ",
            "    \\/    ",
            "          ",
            "          ",
        ]),
        'W' => Some(vec![
            "__          __",
            "\\ \\        / /",
            " \\ \\  /\\  / / ",
            "  \\ \\/  \\/ /  ",
            "   \\  /\\  /   ",
            "    \\/  \\/    ",
            "              ",
            "              ",
        ]),
        'X' => Some(vec![
            "__   __",
            "\\ \\ / /",
            " \\ V / ",
            "  > <  ",
            " / . \\ ",
            "/_/ \\_\\",
            "       ",
            "       ",
        ]),
        'Y' => Some(vec![
            "__     __",
            "\\ \\   / /",
            " \\ \\_/ / ",
            "  \\   /  ",
            "   | |   ",
            "   |_|   ",
            "         ",
            "         ",
        ]),
        'Z' => Some(vec![
            " ______", "|___  /", "   / / ", "  / /  ", " / /__ ", "/_____|", "       ", "       ",
        ]),
        ' ' => Some(vec![
            "    ", "    ", "    ", "    ", "    ", "    ", "    ", "    ",
        ]),
        _ => Some(vec![
            "    ", "    ", " ?? ", "    ", " ?? ", "    ", "    ", "    ",
        ]),
    }
}

fn doom_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "        ",
            "   /\\   ",
            "  /  \\  ",
            " / /\\ \\ ",
            "/______\\",
            "\\      /",
            " \\    / ",
            "  \\__/  ",
        ]),
        'B' => Some(vec![
            "______  ",
            "| ___ \\ ",
            "| |_/ / ",
            "| ___ \\ ",
            "| |_/ / ",
            "|_____/ ",
            "        ",
            "        ",
        ]),
        'C' => Some(vec![
            " _____ ", "/  __ \\", "| /  \\|", "| |    ", "| \\__/|", " \\____/", "       ",
            "       ",
        ]),
        'D' => Some(vec![
            "______  ",
            "|  _  \\ ",
            "| | | | ",
            "| | | | ",
            "| |/ /  ",
            "|___/   ",
            "        ",
            "        ",
        ]),
        'E' => Some(vec![
            "_______ ", "|  ___| ", "| |__   ", "|  __|  ", "| |___  ", "|_____| ", "        ",
            "        ",
        ]),
        'F' => Some(vec![
            "_______ ", "|  ___| ", "| |__   ", "|  __|  ", "| |     ", "|_|     ", "        ",
            "        ",
        ]),
        'G' => Some(vec![
            " _____ ",
            "/  __ \\",
            "| |  \\|",
            "| | __ ",
            "| |_\\ \\",
            " \\____/",
            "       ",
            "       ",
        ]),
        'H' => Some(vec![
            "_   _  ", "| | | | ", "| |_| | ", "|  _  | ", "| | | | ", "|_| |_| ", "        ",
            "        ",
        ]),
        'I' => Some(vec![
            "_____ ", "|_   _|", "  | |  ", "  | |  ", " _| |_ ", "|_____|", "       ", "       ",
        ]),
        'J' => Some(vec![
            "    _ ",
            "   | | ",
            "   | | ",
            "   | | ",
            "/\\__/ / ",
            "\\____/  ",
            "        ",
            "        ",
        ]),
        'K' => Some(vec![
            "_   __",
            "| | / /",
            "| |/ / ",
            "|    \\ ",
            "| |\\  \\",
            "|_| \\_\\",
            "       ",
            "       ",
        ]),
        'L' => Some(vec![
            "_     ", "| |    ", "| |    ", "| |    ", "| |___ ", "|_____|", "       ", "       ",
        ]),
        'M' => Some(vec![
            "___  ___",
            "|  \\/  |",
            "| .  . |",
            "| |\\/| |",
            "| |  | |",
            "\\_|  |_/",
            "        ",
            "        ",
        ]),
        'N' => Some(vec![
            "_   _ ",
            "| \\ | |",
            "|  \\| |",
            "| . ` |",
            "| |\\  |",
            "\\_| \\_/",
            "       ",
            "       ",
        ]),
        'O' => Some(vec![
            " _____ ", "/  _  \\", "| | | |", "| | | |", "| |_| |", "\\_____/", "       ",
            "       ",
        ]),
        'P' => Some(vec![
            "______ ", "| ___ \\", "| |_/ /", "|  __/ ", "| |    ", "\\_|    ", "       ",
            "       ",
        ]),
        'Q' => Some(vec![
            " _____ ",
            "/  _  \\",
            "| | | |",
            "| |_| |",
            " \\___\\\\",
            "     \\_\\",
            "       ",
            "       ",
        ]),
        'R' => Some(vec![
            "______ ",
            "| ___ \\",
            "| |_/ /",
            "|    / ",
            "| |\\ \\ ",
            "\\_| \\_\\",
            "       ",
            "       ",
        ]),
        'S' => Some(vec![
            " _____ ", "/  ___|", "\\ `--. ", " `--. \\", "/\\__/ /", "\\____/ ", "       ",
            "       ",
        ]),
        'T' => Some(vec![
            "_______", "|_   _|", "  | |  ", "  | |  ", "  | |  ", "  \\_/  ", "       ", "       ",
        ]),
        'U' => Some(vec![
            "_   _ ", "| | | |", "| | | |", "| | | |", "| |_| |", " \\___/ ", "       ", "       ",
        ]),
        'V' => Some(vec![
            "_   _ ", "| | | |", "| | | |", "| \\ / |", " \\ V / ", "  \\_/  ", "       ",
            "       ",
        ]),
        'W' => Some(vec![
            "_    _  ",
            "| |  | | ",
            "| |  | | ",
            "| |/\\| | ",
            "\\  /\\  / ",
            " \\/  \\/  ",
            "         ",
            "         ",
        ]),
        'X' => Some(vec![
            "__   __",
            "\\ \\ / /",
            " \\ V / ",
            " /   \\ ",
            "/ /^\\ \\",
            "\\/   \\/",
            "       ",
            "       ",
        ]),
        'Y' => Some(vec![
            "__   __",
            "\\ \\ / /",
            " \\ V / ",
            "  \\ /  ",
            "  | |  ",
            "  \\_/  ",
            "       ",
            "       ",
        ]),
        'Z' => Some(vec![
            "_______", "|___  /", "   / / ", "  / /  ", "./ /___", "/_____/", "       ", "       ",
        ]),
        ' ' => Some(vec![
            "    ", "    ", "    ", "    ", "    ", "    ", "    ", "    ",
        ]),
        _ => Some(vec![
            "    ", "    ", " ?? ", "    ", " ?? ", "    ", "    ", "    ",
        ]),
    }
}

fn script_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_lowercase() {
        'a' => Some(vec!["      ", "  __  ", " /  \\ ", "(____)", "      "]),
        'b' => Some(vec!["      ", " /~)  ", "/_/\\_ ", "\\   / ", " ~-~  "]),
        'c' => Some(vec!["     ", " ___ ", "/  _)", "\\__\\ ", "     "]),
        'd' => Some(vec!["      ", "  _/| ", " / _| ", "/_/  |", "     ~"]),
        'e' => Some(vec!["     ", " ___ ", "/ -_)", "\\___/", "     "]),
        'f' => Some(vec!["   __", "  / _)", " / /  ", "(  (  ", " \\__\\ "]),
        'g' => Some(vec!["      ", "  ___ ", " / _ \\", "( (_) )", " \\__,/"]),
        'h' => Some(vec!["      ", " /~)_ ", "/ / / ", "\\/_/  ", "      "]),
        'i' => Some(vec!["  ", " o", " |", " |", "  "]),
        'j' => Some(vec!["     ", "   o ", "   | ", "\\__| ", "     "]),
        'k' => Some(vec!["      ", " /~\\/~", "/ /\\/ ", "~    ~", "      "]),
        'l' => Some(vec!["   ", " | ", " | ", " |_", "   "]),
        'm' => Some(vec![
            "        ",
            " /~\\/~\\ ",
            "/ /  / /",
            "~    ~  ",
            "        ",
        ]),
        'n' => Some(vec!["      ", " /~\\ _ ", "/ / / /", "~  ~ ~ ", "      "]),
        'o' => Some(vec!["     ", " ___ ", "/ _ \\", "\\___/", "     "]),
        'p' => Some(vec!["      ", " ___  ", "/ _ \\ ", "\\  __/", "|_|   "]),
        'q' => Some(vec!["      ", "  ___ ", " / _ \\", "/_/ \\_\\", "   /_/"]),
        'r' => Some(vec!["    ", " __ ", "/ _)", "\\_\\ ", "    "]),
        's' => Some(vec!["    ", " __ ", "(_ \\", "__) ", "    "]),
        't' => Some(vec!["     ", " _|_ ", "  |  ", "  |_ ", "     "]),
        'u' => Some(vec!["      ", " _  _ ", "| || |", " \\_,_|", "      "]),
        'v' => Some(vec!["      ", " _  _ ", " \\ V /", "  \\_/ ", "      "]),
        'w' => Some(vec![
            "        ",
            " _    _ ",
            " \\ \\/\\/ /",
            "  \\_/\\_/ ",
            "        ",
        ]),
        'x' => Some(vec!["     ", " _  _", " \\\\//", " //\\\\", "~  ~ "]),
        'y' => Some(vec!["      ", " _  _ ", " \\ V /", "  |_| ", " /_/  "]),
        'z' => Some(vec!["    ", " ___", "|_ /", "/__|", "    "]),
        ' ' => Some(vec!["   ", "   ", "   ", "   ", "   "]),
        _ => Some(vec!["  ", "??", "  ", "??", "  "]),
    }
}

fn digital_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![" ___ ", "|___|", "| | |", "| | |", "     "]),
        'B' => Some(vec!["___ ", "|__|", "|__|", "|__|", "    "]),
        'C' => Some(vec![" ___", "|   ", "|   ", "|___", "    "]),
        'D' => Some(vec!["___ ", "|  |", "|  |", "|__|", "    "]),
        'E' => Some(vec![" ___", "|__ ", "|__ ", "|___", "    "]),
        'F' => Some(vec![" ___", "|__ ", "|   ", "|   ", "    "]),
        'G' => Some(vec![" ___", "|   ", "| _ ", "|__|", "    "]),
        'H' => Some(vec!["    ", "|__|", "|  |", "|  |", "    "]),
        'I' => Some(vec!["___", " | ", " | ", "_|_", "   "]),
        'J' => Some(vec!["___", "  |", "  |", "|_|", "   "]),
        'K' => Some(vec!["    ", "| / ", "|<  ", "| \\ ", "    "]),
        'L' => Some(vec!["    ", "|   ", "|   ", "|___", "    "]),
        'M' => Some(vec!["     ", "|\\/\\ ", "|  | ", "|  | ", "     "]),
        'N' => Some(vec!["     ", "|\\ | ", "| \\| ", "|  | ", "     "]),
        'O' => Some(vec![" ___ ", "|   |", "|   |", "|___|", "     "]),
        'P' => Some(vec![" ___ ", "|___|", "|    ", "|    ", "     "]),
        'Q' => Some(vec![" ___ ", "|   |", "|  \\|", "|___\\", "     "]),
        'R' => Some(vec![" ___ ", "|___|", "| \\  ", "|  \\ ", "     "]),
        'S' => Some(vec![" ___", "|__ ", " __|", "|__ ", "    "]),
        'T' => Some(vec!["___", " | ", " | ", " | ", "   "]),
        'U' => Some(vec!["     ", "|   |", "|   |", "|___|", "     "]),
        'V' => Some(vec!["     ", "|   |", " \\ / ", "  V  ", "     "]),
        'W' => Some(vec![
            "       ",
            "|     |",
            "|  |  |",
            " \\/ \\/ ",
            "       ",
        ]),
        'X' => Some(vec!["     ", "\\ / ", " X  ", "/ \\ ", "    "]),
        'Y' => Some(vec!["     ", "\\ / ", " |  ", " |  ", "    "]),
        'Z' => Some(vec!["___", "  /", " / ", "/__", "   "]),
        '0' => Some(vec![" ___ ", "|  /|", "| / |", "|/__|", "     "]),
        '1' => Some(vec!["   ", " | ", " | ", " | ", "   "]),
        '2' => Some(vec![" ___", " __|", "|__ ", "|___", "    "]),
        '3' => Some(vec!["___", "__|", "__|", "__|", "   "]),
        '4' => Some(vec!["    ", "|__|", "   |", "   |", "    "]),
        '5' => Some(vec![" ___", "|__ ", " __|", "|__ ", "    "]),
        '6' => Some(vec![" ___", "|__ ", "|__|", "|__|", "    "]),
        '7' => Some(vec!["___", "  |", "  |", "  |", "   "]),
        '8' => Some(vec![" ___", "|__|", "|__|", "|__|", "    "]),
        '9' => Some(vec![" ___", "|__|", " __|", " __|", "    "]),
        ' ' => Some(vec!["   ", "   ", "   ", "   ", "   "]),
        _ => Some(vec!["   ", " ? ", "   ", " ? ", "   "]),
    }
}

fn font_3d_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "    ___    ",
            "   /   |   ",
            "  / /| |   ",
            " / ___ |   ",
            "/_/  |_|__ ",
            "     /____|",
            "           ",
        ]),
        'B' => Some(vec![
            "  ______  ",
            " |  ___ \\ ",
            " | |___) |",
            " |  ___  <",
            " | |___) |",
            " |______/ ",
            "          ",
        ]),
        'C' => Some(vec![
            "  _______ ",
            " / _____/ ",
            "| |       ",
            "| |       ",
            "| |_____  ",
            " \\_____/  ",
            "          ",
        ]),
        'D' => Some(vec![
            "  _____   ",
            " |  __ \\  ",
            " | |  \\ \\ ",
            " | |   | |",
            " | |__/ / ",
            " |_____/  ",
            "          ",
        ]),
        'E' => Some(vec![
            "  _______ ",
            " |  _____|",
            " | |__    ",
            " |  __|   ",
            " | |_____ ",
            " |_______|",
            "          ",
        ]),
        'F' => Some(vec![
            "  _______ ",
            " |  _____|",
            " | |__    ",
            " |  __|   ",
            " | |      ",
            " |_|      ",
            "          ",
        ]),
        'G' => Some(vec![
            "  _______ ",
            " / _____/ ",
            "| |   __  ",
            "| |  |_ | ",
            "| |___| | ",
            " \\_____/  ",
            "          ",
        ]),
        'H' => Some(vec![
            "  _   _   ",
            " | | | |  ",
            " | |_| |  ",
            " |  _  |  ",
            " | | | |  ",
            " |_| |_|  ",
            "          ",
        ]),
        'I' => Some(vec![
            "  _____ ", " |_   _|", "   | |  ", "   | |  ", "  _| |_ ", " |_____|", "        ",
        ]),
        'J' => Some(vec![
            "       __ ",
            "      |  |",
            "      |  |",
            "  __  |  |",
            " |  |_|  |",
            " |_______/",
            "          ",
        ]),
        'K' => Some(vec![
            "  _  __   ",
            " | |/ /   ",
            " | ' /    ",
            " |  <     ",
            " | . \\    ",
            " |_|\\_\\   ",
            "          ",
        ]),
        'L' => Some(vec![
            "  _       ",
            " | |      ",
            " | |      ",
            " | |      ",
            " | |_____ ",
            " |_______|",
            "          ",
        ]),
        'M' => Some(vec![
            "  __    __ ",
            " |  \\  /  |",
            " |   \\/   |",
            " | |\\  /| |",
            " | | \\/ | |",
            " |_|    |_|",
            "           ",
        ]),
        'N' => Some(vec![
            "  _     _ ",
            " | \\   | |",
            " |  \\  | |",
            " |   \\ | |",
            " | |\\ \\| |",
            " |_| \\___/",
            "          ",
        ]),
        'O' => Some(vec![
            "  ______  ",
            " / ____ \\ ",
            "| |    | |",
            "| |    | |",
            "| |____| |",
            " \\______/ ",
            "          ",
        ]),
        'P' => Some(vec![
            "  ______  ",
            " |  ___ \\ ",
            " | |___) |",
            " |  ____/ ",
            " | |      ",
            " |_|      ",
            "          ",
        ]),
        'Q' => Some(vec![
            "  ______  ",
            " / ____ \\ ",
            "| |    | |",
            "| |  _ | |",
            "| |_| \\| |",
            " \\_____\\_\\",
            "          ",
        ]),
        'R' => Some(vec![
            "  ______  ",
            " |  ___ \\ ",
            " | |___) |",
            " |  _  _/ ",
            " | | \\ \\  ",
            " |_|  \\_\\ ",
            "          ",
        ]),
        'S' => Some(vec![
            "  _______ ",
            " / ____  |",
            "| (___  | ",
            " \\___ \\ | ",
            " ____) | |",
            "|________|",
            "          ",
        ]),
        'T' => Some(vec![
            "  ________ ",
            " |__  ____|",
            "    | |    ",
            "    | |    ",
            "    | |    ",
            "    |_|    ",
            "           ",
        ]),
        'U' => Some(vec![
            "  _     _ ",
            " | |   | |",
            " | |   | |",
            " | |   | |",
            " | |___| |",
            "  \\_____/ ",
            "          ",
        ]),
        'V' => Some(vec![
            "  _     _ ",
            " | |   | |",
            " | |   | |",
            "  \\ \\ / / ",
            "   \\ V /  ",
            "    \\_/   ",
            "          ",
        ]),
        'W' => Some(vec![
            "  _       _ ",
            " | |     | |",
            " | |     | |",
            " |  \\ _ /  |",
            "  \\ |_| /  ",
            "   |___|   ",
            "           ",
        ]),
        'X' => Some(vec![
            "  _     _ ",
            "  \\ \\   / /",
            "   \\ \\_/ / ",
            "    > < /  ",
            "   / /\\ \\  ",
            "  /_/  \\_\\ ",
            "          ",
        ]),
        'Y' => Some(vec![
            "  _     _ ",
            "  \\ \\   / /",
            "   \\ \\_/ / ",
            "    \\   /  ",
            "     | |   ",
            "     |_|   ",
            "          ",
        ]),
        'Z' => Some(vec![
            "  ________ ",
            " |______  |",
            "       / / ",
            "      / /  ",
            "     / /__ ",
            "    /_____|",
            "          ",
        ]),
        ' ' => Some(vec![
            "     ", "     ", "     ", "     ", "     ", "     ", "     ",
        ]),
        _ => Some(vec![
            "     ", " ??? ", "     ", " ??? ", "     ", "     ", "     ",
        ]),
    }
}

fn bubble_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec!["  ⓐ  ", " ⓐ ⓐ ", "ⓐⓐⓐⓐⓐ", "ⓐ   ⓐ"]),
        'B' => Some(vec!["ⓑⓑⓑⓑ ", "ⓑ   ⓑ", "ⓑⓑⓑⓑ ", "ⓑⓑⓑⓑ "]),
        'C' => Some(vec![" ⓒⓒⓒⓒ", "ⓒ    ", "ⓒ    ", " ⓒⓒⓒⓒ"]),
        'D' => Some(vec!["ⓓⓓⓓ  ", "ⓓ  ⓓ ", "ⓓ   ⓓ", "ⓓⓓⓓ  "]),
        'E' => Some(vec!["ⓔⓔⓔⓔⓔ", "ⓔⓔⓔ  ", "ⓔ    ", "ⓔⓔⓔⓔⓔ"]),
        'F' => Some(vec!["ⓕⓕⓕⓕⓕ", "ⓕⓕⓕ  ", "ⓕ    ", "ⓕ    "]),
        'G' => Some(vec![" ⓖⓖⓖⓖ", "ⓖ    ", "ⓖ  ⓖⓖ", " ⓖⓖⓖⓖ"]),
        'H' => Some(vec!["ⓗ   ⓗ", "ⓗⓗⓗⓗⓗ", "ⓗ   ⓗ", "ⓗ   ⓗ"]),
        'I' => Some(vec!["ⓘⓘⓘⓘⓘ", "  ⓘ  ", "  ⓘ  ", "ⓘⓘⓘⓘⓘ"]),
        'J' => Some(vec!["    ⓙ", "    ⓙ", "ⓙ   ⓙ", " ⓙⓙⓙ "]),
        'K' => Some(vec!["ⓚ  ⓚ ", "ⓚ ⓚ  ", "ⓚⓚ   ", "ⓚ  ⓚ "]),
        'L' => Some(vec!["ⓛ    ", "ⓛ    ", "ⓛ    ", "ⓛⓛⓛⓛⓛ"]),
        'M' => Some(vec!["ⓜ   ⓜ", "ⓜⓜ ⓜⓜ", "ⓜ ⓜ ⓜ", "ⓜ   ⓜ"]),
        'N' => Some(vec!["ⓝ   ⓝ", "ⓝⓝ  ⓝ", "ⓝ ⓝ ⓝ", "ⓝ  ⓝⓝ"]),
        'O' => Some(vec![" ⓞⓞⓞ ", "ⓞ   ⓞ", "ⓞ   ⓞ", " ⓞⓞⓞ "]),
        'P' => Some(vec!["ⓟⓟⓟⓟ ", "ⓟ   ⓟ", "ⓟⓟⓟⓟ ", "ⓟ    "]),
        'Q' => Some(vec![" ⓠⓠⓠ ", "ⓠ   ⓠ", "ⓠ  ⓠ ", " ⓠⓠ ⓠ"]),
        'R' => Some(vec!["ⓡⓡⓡⓡ ", "ⓡ   ⓡ", "ⓡⓡⓡⓡ ", "ⓡ   ⓡ"]),
        'S' => Some(vec![" ⓢⓢⓢⓢ", "ⓢ    ", " ⓢⓢⓢ ", "ⓢⓢⓢⓢ "]),
        'T' => Some(vec!["ⓣⓣⓣⓣⓣ", "  ⓣ  ", "  ⓣ  ", "  ⓣ  "]),
        'U' => Some(vec!["ⓤ   ⓤ", "ⓤ   ⓤ", "ⓤ   ⓤ", " ⓤⓤⓤ "]),
        'V' => Some(vec!["ⓥ   ⓥ", "ⓥ   ⓥ", " ⓥ ⓥ ", "  ⓥ  "]),
        'W' => Some(vec!["ⓦ   ⓦ", "ⓦ ⓦ ⓦ", "ⓦⓦ ⓦⓦ", "ⓦ   ⓦ"]),
        'X' => Some(vec!["ⓧ   ⓧ", " ⓧ ⓧ ", "  ⓧ  ", "ⓧ   ⓧ"]),
        'Y' => Some(vec!["ⓨ   ⓨ", " ⓨ ⓨ ", "  ⓨ  ", "  ⓨ  "]),
        'Z' => Some(vec!["ⓩⓩⓩⓩⓩ", "   ⓩ ", "  ⓩ  ", "ⓩⓩⓩⓩⓩ"]),
        ' ' => Some(vec!["    ", "    ", "    ", "    "]),
        _ => Some(vec!["ⓧ", "ⓧ", "ⓧ", "ⓧ"]),
    }
}

fn graffiti_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "   __   ",
            "  /  \\  ",
            " /    \\ ",
            "/======\\",
            "|      |",
            "|      |",
        ]),
        'B' => Some(vec![
            "======\\ ",
            "|      \\",
            "|======/",
            "|      \\",
            "|======/",
            "        ",
        ]),
        'C' => Some(vec![
            " /=====\\",
            "|       ",
            "|       ",
            "|       ",
            " \\=====/",
            "        ",
        ]),
        'D' => Some(vec![
            "=====\\  ",
            "|     \\ ",
            "|      |",
            "|     / ",
            "=====/  ",
            "        ",
        ]),
        'E' => Some(vec![
            "========", "|       ", "|=====  ", "|       ", "========", "        ",
        ]),
        'F' => Some(vec![
            "========", "|       ", "|=====  ", "|       ", "|       ", "        ",
        ]),
        'G' => Some(vec![
            " /=====\\",
            "|       ",
            "|   ====",
            "|      |",
            " \\=====/",
            "        ",
        ]),
        'H' => Some(vec![
            "|      |", "|      |", "|======|", "|      |", "|      |", "        ",
        ]),
        'I' => Some(vec![
            "========", "   ||   ", "   ||   ", "   ||   ", "========", "        ",
        ]),
        'J' => Some(vec![
            "     ===", "      | ", "      | ", "|     | ", " \\====/", "        ",
        ]),
        'K' => Some(vec![
            "|    / ", "|   /  ", "|==/   ", "|   \\  ", "|    \\ ", "       ",
        ]),
        'L' => Some(vec![
            "|      ", "|      ", "|      ", "|      ", "|======", "       ",
        ]),
        'M' => Some(vec![
            "|\\    /|",
            "| \\  / |",
            "|  \\/  |",
            "|      |",
            "|      |",
            "        ",
        ]),
        'N' => Some(vec![
            "|\\     |",
            "| \\    |",
            "|  \\   |",
            "|   \\  |",
            "|    \\ |",
            "        ",
        ]),
        'O' => Some(vec![
            " /====\\ ",
            "|      |",
            "|      |",
            "|      |",
            " \\====/ ",
            "        ",
        ]),
        'P' => Some(vec![
            "======\\ ",
            "|      |",
            "|======/",
            "|       ",
            "|       ",
            "        ",
        ]),
        'Q' => Some(vec![
            " /====\\ ",
            "|      |",
            "|   \\  |",
            "|    \\ |",
            " \\====\\|",
            "        ",
        ]),
        'R' => Some(vec![
            "======\\ ",
            "|      |",
            "|=====/ ",
            "|    \\  ",
            "|     \\ ",
            "        ",
        ]),
        'S' => Some(vec![
            " /=====\\",
            "|       ",
            " \\====\\ ",
            "       |",
            "\\=====/",
            "        ",
        ]),
        'T' => Some(vec![
            "========", "   ||   ", "   ||   ", "   ||   ", "   ||   ", "        ",
        ]),
        'U' => Some(vec![
            "|      |",
            "|      |",
            "|      |",
            "|      |",
            " \\====/ ",
            "        ",
        ]),
        'V' => Some(vec![
            "|      |",
            "|      |",
            " \\    / ",
            "  \\  /  ",
            "   \\/   ",
            "        ",
        ]),
        'W' => Some(vec![
            "|      |",
            "|      |",
            "|  /\\  |",
            "| /  \\ |",
            "|/    \\|",
            "        ",
        ]),
        'X' => Some(vec![
            "\\      /",
            " \\    / ",
            "  \\  /  ",
            "  /  \\  ",
            " /    \\ ",
            "        ",
        ]),
        'Y' => Some(vec![
            "\\      /",
            " \\    / ",
            "  \\  /  ",
            "   ||   ",
            "   ||   ",
            "        ",
        ]),
        'Z' => Some(vec![
            "========", "     /  ", "   /    ", " /      ", "========", "        ",
        ]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    ", "    "]),
    }
}

fn gothic_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "     _     ",
            "    /_\\    ",
            "   //_\\\\   ",
            "  //___\\\\  ",
            " //     \\\\ ",
            "//       \\\\",
        ]),
        'B' => Some(vec![
            "_______ ",
            "||___  \\",
            "||    ) )",
            "||___  / ",
            "||    ) )",
            "||____/ ",
        ]),
        'C' => Some(vec![
            "  _____ ",
            " / ____\\",
            "| |     ",
            "| |     ",
            "| |____ ",
            " \\_____/",
        ]),
        'D' => Some(vec![
            "______  ",
            "||    \\ ",
            "||     \\",
            "||     /",
            "||    / ",
            "||___/  ",
        ]),
        'E' => Some(vec![
            "_______ ", "||_____|", "||___   ", "||___   ", "||_____ ", "||_____|",
        ]),
        'F' => Some(vec![
            "_______ ", "||_____|", "||___   ", "||___   ", "||      ", "||      ",
        ]),
        'G' => Some(vec![
            "  _____ ",
            " / ____\\",
            "| |  __ ",
            "| | |_ |",
            "| |__| |",
            " \\_____/",
        ]),
        'H' => Some(vec![
            "||   || ", "||   || ", "||___|| ", "||---|| ", "||   || ", "||   || ",
        ]),
        'I' => Some(vec![
            "______", " _||_ ", "  ||  ", "  ||  ", " _||_ ", "______",
        ]),
        'J' => Some(vec![
            "  ____ ", "   || ", "   || ", "   || ", "|\\_|| ", " \\__/ ",
        ]),
        'K' => Some(vec![
            "||  //", "|| // ", "||//  ", "||\\\\  ", "|| \\\\ ", "||  \\\\",
        ]),
        'L' => Some(vec![
            "||     ", "||     ", "||     ", "||     ", "||____ ", "||_____|",
        ]),
        'M' => Some(vec![
            "||\\  /||",
            "|| \\/ ||",
            "||    ||",
            "||    ||",
            "||    ||",
            "||    ||",
        ]),
        'N' => Some(vec![
            "||\\   ||",
            "|| \\  ||",
            "||  \\ ||",
            "||   \\||",
            "||    ||",
            "||    ||",
        ]),
        'O' => Some(vec![
            "  ____  ",
            " /    \\ ",
            "||    ||",
            "||    ||",
            "||    ||",
            " \\____/ ",
        ]),
        'P' => Some(vec![
            "______  ",
            "||    \\ ",
            "||____/ ",
            "||      ",
            "||      ",
            "||      ",
        ]),
        'Q' => Some(vec![
            "  ____  ",
            " /    \\ ",
            "||    ||",
            "||  \\ ||",
            "||   \\||",
            " \\____\\\\",
        ]),
        'R' => Some(vec![
            "______  ",
            "||    \\ ",
            "||____/ ",
            "||  \\   ",
            "||   \\  ",
            "||    \\ ",
        ]),
        'S' => Some(vec![
            " _____  ",
            "/  ___| ",
            "\\___ \\  ",
            "  ___)| ",
            " \\___/  ",
            "        ",
        ]),
        'T' => Some(vec![
            "________", "  _||_  ", "   ||   ", "   ||   ", "   ||   ", "   ||   ",
        ]),
        'U' => Some(vec![
            "||    ||",
            "||    ||",
            "||    ||",
            "||    ||",
            "||    ||",
            " \\____/ ",
        ]),
        'V' => Some(vec![
            "||    ||",
            "||    ||",
            " \\\\  // ",
            "  \\\\//  ",
            "   \\/   ",
            "        ",
        ]),
        'W' => Some(vec![
            "||    ||",
            "||    ||",
            "|| /\\ ||",
            "||/  \\||",
            "||    ||",
            "        ",
        ]),
        'X' => Some(vec![
            "\\\\    //",
            " \\\\  // ",
            "  \\\\//  ",
            "  //\\\\  ",
            " //  \\\\ ",
            "//    \\\\",
        ]),
        'Y' => Some(vec![
            "\\\\    //",
            " \\\\  // ",
            "  \\\\//  ",
            "   ||   ",
            "   ||   ",
            "   ||   ",
        ]),
        'Z' => Some(vec![
            "________",
            "______//",
            "    //  ",
            "  //    ",
            "//______",
            "________|",
        ]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    ", "    "]),
    }
}

fn lean_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "    _/_/    ",
            "   _/  _/   ",
            "  _/_/_/_/  ",
            " _/    _/   ",
            "_/      _/  ",
        ]),
        'B' => Some(vec![
            "  _/_/_/   ",
            " _/    _/  ",
            "_/_/_/_/   ",
            "_/    _/   ",
            "_/_/_/     ",
        ]),
        'C' => Some(vec![
            "  _/_/_/_/ ",
            " _/        ",
            "_/         ",
            "_/         ",
            " _/_/_/_/  ",
        ]),
        'D' => Some(vec![
            "  _/_/_/   ",
            " _/    _/  ",
            "_/      _/ ",
            "_/    _/   ",
            "_/_/_/     ",
        ]),
        'E' => Some(vec![
            "  _/_/_/_/_/",
            " _/         ",
            "_/_/_/_/    ",
            "_/          ",
            "_/_/_/_/_/  ",
        ]),
        'F' => Some(vec![
            "  _/_/_/_/_/",
            " _/         ",
            "_/_/_/_/    ",
            "_/          ",
            "_/          ",
        ]),
        'G' => Some(vec![
            "  _/_/_/_/ ",
            " _/        ",
            "_/   _/_/  ",
            "_/      _/ ",
            " _/_/_/_/  ",
        ]),
        'H' => Some(vec![
            "  _/    _/  ",
            " _/    _/   ",
            "_/_/_/_/_/  ",
            "_/      _/  ",
            "_/      _/  ",
        ]),
        'I' => Some(vec![
            "  _/_/_/_/",
            "     _/   ",
            "    _/    ",
            "   _/     ",
            "_/_/_/_/  ",
        ]),
        'J' => Some(vec![
            "      _/_/",
            "        _/",
            "       _/ ",
            "      _/  ",
            "_/_/_/    ",
        ]),
        'K' => Some(vec![
            "  _/  _/  ",
            " _/ _/    ",
            "_/_/      ",
            "_/  _/    ",
            "_/    _/  ",
        ]),
        'L' => Some(vec![
            "  _/       ",
            " _/        ",
            "_/         ",
            "_/         ",
            "_/_/_/_/_/ ",
        ]),
        'M' => Some(vec![
            "  _/      _/  ",
            " _/_/  _/_/   ",
            "_/  _/_/  _/  ",
            "_/        _/  ",
            "_/        _/  ",
        ]),
        'N' => Some(vec![
            "  _/      _/ ",
            " _/_/    _/  ",
            "_/  _/  _/   ",
            "_/    _/_/   ",
            "_/      _/   ",
        ]),
        'O' => Some(vec![
            "   _/_/_/  ",
            "  _/    _/ ",
            " _/      _/",
            "_/      _/ ",
            " _/_/_/    ",
        ]),
        'P' => Some(vec![
            "  _/_/_/   ",
            " _/    _/  ",
            "_/_/_/     ",
            "_/         ",
            "_/         ",
        ]),
        'Q' => Some(vec![
            "   _/_/_/  ",
            "  _/    _/ ",
            " _/      _/",
            "_/    _/_/ ",
            " _/_/_/ _/ ",
        ]),
        'R' => Some(vec![
            "  _/_/_/   ",
            " _/    _/  ",
            "_/_/_/     ",
            "_/   _/    ",
            "_/     _/  ",
        ]),
        'S' => Some(vec![
            "  _/_/_/_/ ",
            " _/        ",
            " _/_/_/    ",
            "      _/   ",
            "_/_/_/     ",
        ]),
        'T' => Some(vec![
            "_/_/_/_/_/_/",
            "     _/     ",
            "    _/      ",
            "   _/       ",
            "  _/        ",
        ]),
        'U' => Some(vec![
            "  _/      _/",
            " _/      _/ ",
            "_/      _/  ",
            "_/    _/    ",
            " _/_/_/     ",
        ]),
        'V' => Some(vec![
            "  _/        _/",
            " _/        _/ ",
            "_/        _/  ",
            " _/      _/   ",
            "  _/_/_/_/    ",
        ]),
        'W' => Some(vec![
            "  _/          _/",
            " _/          _/ ",
            "_/    _/    _/  ",
            " _/  _/ _/ _/   ",
            "  _/      _/    ",
        ]),
        'X' => Some(vec![
            "  _/      _/",
            "   _/  _/   ",
            "    _/_/    ",
            "   _/  _/   ",
            "  _/    _/  ",
        ]),
        'Y' => Some(vec![
            "  _/      _/",
            "   _/  _/   ",
            "    _/_/    ",
            "     _/     ",
            "    _/      ",
        ]),
        'Z' => Some(vec![
            "_/_/_/_/_/_/",
            "       _/   ",
            "    _/      ",
            " _/         ",
            "_/_/_/_/_/_/",
        ]),
        ' ' => Some(vec!["     ", "     ", "     ", "     ", "     "]),
        _ => Some(vec!["     ", " ??? ", "     ", " ??? ", "     "]),
    }
}

fn isometric_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "      ___     ",
            "     /\\  \\    ",
            "    /::\\  \\   ",
            "   /:/\\:\\  \\  ",
            "  /:/_\\:\\__\\ ",
            " /:/ `\\/__/ ",
            " \\/__/       ",
        ]),
        'B' => Some(vec![
            "   ___     ",
            "  /\\  \\    ",
            " |::\\  \\   ",
            " |:|:\\  \\  ",
            " |:| \\:\\__\\",
            " |:|  \\/__/",
            "  \\|__|     ",
        ]),
        'C' => Some(vec![
            "      ___   ",
            "     /\\__\\  ",
            "    /:/  /  ",
            "   /:/  /   ",
            "  /:/  /    ",
            " /:/__/     ",
            " \\/__/      ",
        ]),
        'D' => Some(vec![
            "   ___      ",
            "  /\\  \\     ",
            " |::\\  \\    ",
            " |:|:\\  \\   ",
            " |:| \\:\\__\\ ",
            " |:|  \\/__/ ",
            "  \\|__|      ",
        ]),
        'E' => Some(vec![
            "   _____   ",
            "  /\\  ___ \\ ",
            " |::\\/___/|",
            " |:::_____/",
            " |:::\\____\\",
            "  \\::/__/  ",
            "   \\/__/    ",
        ]),
        'F' => Some(vec![
            "   _____   ",
            "  /\\  ___ \\ ",
            " |::\\/___/|",
            " |::_____/ ",
            " |::|      ",
            "  \\|__|     ",
            "            ",
        ]),
        'G' => Some(vec![
            "      ____  ",
            "     /\\  __\\",
            "    |::|\\_\\|",
            "    |:|  __ ",
            "    |::|\\_\\|",
            "     \\:|\\_\\|",
            "      \\|__| ",
        ]),
        'H' => Some(vec![
            "   ___  ___ ",
            "  /\\  \\/\\  \\",
            " |::| |::| |",
            " |::|_|::| |",
            " |:::::/__/ ",
            " |::::|____ ",
            "  \\|__|     ",
        ]),
        'I' => Some(vec![
            "   _____ ",
            "  /\\__  \\",
            " |::| | |",
            " |::| | |",
            " |::| | |",
            " |::|_|/ ",
            "  \\/__/  ",
        ]),
        'J' => Some(vec![
            "     _____ ",
            "    /\\__  \\",
            "   /::| | |",
            "  /::/ | | ",
            " /::/__| | ",
            " \\:\\/___/ ",
            "  \\/__/    ",
        ]),
        'K' => Some(vec![
            "   ___  ___",
            "  /\\  \\/\\__\\",
            " |::|_\\:| |",
            " |:::__::| ",
            " |:|  |::| ",
            " |:|__|::| ",
            "  \\/__/    ",
        ]),
        'L' => Some(vec![
            "   ___     ",
            "  /\\  \\    ",
            " |::\\  \\   ",
            " |::\\  \\   ",
            " |::\\  \\   ",
            " |::::\\___\\",
            "  \\/__/    ",
        ]),
        'M' => Some(vec![
            "   ___ ___  ",
            "  /\\__/\\__\\ ",
            " |:| |:| | |",
            " |:| |:| | |",
            " |:|_|:|_| |",
            "  \\/_/\\/_/ /",
            "    \\/__/   ",
        ]),
        'N' => Some(vec![
            "   ___  ___ ",
            "  /\\__\\/\\  \\",
            " |::\\ \\::\\ \\",
            " |::\\ \\::\\  \\",
            " |::\\ \\::/__/",
            "  \\::\\/:/   ",
            "   \\/__/     ",
        ]),
        'O' => Some(vec![
            "      ___   ",
            "     /\\  \\  ",
            "    /::\\  \\ ",
            "   /:/\\:\\  \\",
            "  /:/ /::\\__\\",
            " /:/_/:/ /  ",
            " \\/__/::/   ",
        ]),
        'P' => Some(vec![
            "   _____  ",
            "  /\\  __ \\ ",
            " |::|___ \\|",
            " |:::____/ ",
            " |::|      ",
            " |::|      ",
            "  \\|__|     ",
        ]),
        'Q' => Some(vec![
            "      ___   ",
            "     /\\  \\  ",
            "    /::\\  \\ ",
            "   /:/\\:\\  \\",
            "  /:/ /::\\__\\",
            " /:/_/:/ _/ ",
            " \\:\\ \\/_/   ",
        ]),
        'R' => Some(vec![
            "   _____  ",
            "  /\\  __ \\ ",
            " |::|___ \\|",
            " |:::____/ ",
            " |::|_/ /  ",
            " |::/ /    ",
            "  \\/__/     ",
        ]),
        'S' => Some(vec![
            "      ___   ",
            "     /\\__\\  ",
            "    /:/ _/_ ",
            "   /:/ /\\__\\",
            "  /:/ /:/__/",
            " /:/_/::\\ \\ ",
            " \\/__/\\/__/ ",
        ]),
        'T' => Some(vec![
            "  _________ ",
            " /\\________\\",
            " \\/___ __::/",
            "     |::| / ",
            "    /:::|/  ",
            "   /:::/    ",
            "   \\/__/    ",
        ]),
        'U' => Some(vec![
            "   ___  ___ ",
            "  /\\  \\/\\  \\",
            " |::| |::| |",
            " |::| |::| |",
            " |::\\_/::/ /",
            "  \\::/\\/__/ ",
            "   \\/__/    ",
        ]),
        'V' => Some(vec![
            "  ___    ___ ",
            " /\\  \\  /\\  \\",
            " \\:\\  \\/:\\  \\",
            "  \\:\\__\\:\\__\\",
            "   \\/__/\\_\\/ ",
            "        \\/_/ ",
            "             ",
        ]),
        'W' => Some(vec![
            "  ___  ___  ___ ",
            " /\\  \\/\\  \\/\\  \\",
            " \\:\\  \\:\\  \\:\\  \\",
            "  \\:\\__\\:\\__\\:\\__\\",
            "   \\/__/\\_\\:\\/__/",
            "        /:/\\/__/ ",
            "        \\/__/    ",
        ]),
        'X' => Some(vec![
            "   ___  ___ ",
            "  /\\  \\/\\  \\",
            "  \\:\\::\\ \\:\\ ",
            "   \\:/::\\__\\",
            "   /:/\\/__/",
            "  /:/  /   ",
            "  \\/__/    ",
        ]),
        'Y' => Some(vec![
            "  ___    ___ ",
            " /\\  \\  /\\  \\",
            " \\:\\  \\/:\\__\\",
            "  \\:\\__::/  /",
            "   \\/_/:/  / ",
            "     /:/  /  ",
            "     \\/__/   ",
        ]),
        'Z' => Some(vec![
            "  _________ ",
            " /\\________\\",
            " \\/___ __::/",
            "  /:/\\::/ / ",
            " /:/\\::/ /  ",
            " \\/__\\/__/  ",
            "            ",
        ]),
        ' ' => Some(vec![
            "      ", "      ", "      ", "      ", "      ", "      ", "      ",
        ]),
        _ => Some(vec![
            "      ", " ???? ", "      ", " ???? ", "      ", "      ", "      ",
        ]),
    }
}

fn starwars_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            ".     ", "|     ", "|___  ", "|   | ", "|   | ", "      ",
        ]),
        'B' => Some(vec![
            "____  ", "|   \\ ", "|___/ ", "|   \\ ", "|___/ ", "      ",
        ]),
        'C' => Some(vec![
            " ____ ", "/    |", "|     ", "|     ", "\\____|", "      ",
        ]),
        'D' => Some(vec![
            "____  ", "|   \\ ", "|    |", "|   / ", "|___/ ", "      ",
        ]),
        'E' => Some(vec![
            "_____ ", "|     ", "|__   ", "|     ", "|____ ", "      ",
        ]),
        'F' => Some(vec![
            "_____ ", "|     ", "|__   ", "|     ", "|     ", "      ",
        ]),
        'G' => Some(vec![
            " ____ ", "/    |", "| __ |", "|   ||", "\\___/|", "      ",
        ]),
        'H' => Some(vec![
            "|   | ", "|   | ", "|___| ", "|   | ", "|   | ", "      ",
        ]),
        'I' => Some(vec![
            "_____ ", "  |   ", "  |   ", "  |   ", "__|__ ", "      ",
        ]),
        'J' => Some(vec![
            "  ___ ", "    | ", "    | ", "|   | ", " \\__/ ", "      ",
        ]),
        'K' => Some(vec![
            "|   / ", "|  /  ", "|-<   ", "|  \\  ", "|   \\ ", "      ",
        ]),
        'L' => Some(vec![
            "|     ", "|     ", "|     ", "|     ", "|____ ", "      ",
        ]),
        'M' => Some(vec![
            "|\\  /|", "| \\/ |", "|    |", "|    |", "|    |", "      ",
        ]),
        'N' => Some(vec![
            "|\\   |", "| \\  |", "|  \\ |", "|   \\|", "|    |", "      ",
        ]),
        'O' => Some(vec![
            " ____ ", "/    \\", "|    |", "|    |", "\\____/", "      ",
        ]),
        'P' => Some(vec![
            "____  ", "|   \\ ", "|___/ ", "|     ", "|     ", "      ",
        ]),
        'Q' => Some(vec![
            " ____ ", "/    \\", "|    |", "|  \\ |", "\\___\\|", "      ",
        ]),
        'R' => Some(vec![
            "____  ", "|   \\ ", "|___/ ", "|  \\  ", "|   \\ ", "      ",
        ]),
        'S' => Some(vec![
            " ____ ", "/    |", "\\___  ", "    \\ ", "|___/ ", "      ",
        ]),
        'T' => Some(vec![
            "_____ ", "  |   ", "  |   ", "  |   ", "  |   ", "      ",
        ]),
        'U' => Some(vec![
            "|   | ", "|   | ", "|   | ", "|   | ", " \\__/ ", "      ",
        ]),
        'V' => Some(vec!["|   |", "|   |", " \\ / ", " \\ / ", "  V  ", "     "]),
        'W' => Some(vec![
            "|    |", "|    |", "|    |", "| /\\ |", "|/  \\|", "      ",
        ]),
        'X' => Some(vec![
            "\\   /", " \\ / ", "  X  ", " / \\ ", "/   \\", "     ",
        ]),
        'Y' => Some(vec!["\\   /", " \\ / ", "  |  ", "  |  ", "  |  ", "     "]),
        'Z' => Some(vec!["_____", "   / ", "  /  ", " /   ", "/____|", "     "]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    ", "    "]),
    }
}

fn cyberlarge_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "   /\\   ",
            "  /  \\  ",
            " / /\\ \\ ",
            "/ /__\\ \\",
            "\\        \\",
            "         ",
        ]),
        'B' => Some(vec![
            "______  ",
            "| ___ \\ ",
            "| |_/ / ",
            "| ___ \\ ",
            "| |_/ / ",
            "|_____/ ",
        ]),
        'C' => Some(vec![
            "  _____ ",
            " / ___ \\",
            "| |   |_|",
            "| |      ",
            "| |___   ",
            " \\_____\\ ",
        ]),
        'D' => Some(vec![
            "______  ",
            "|  _  \\ ",
            "| | | | ",
            "| |_| | ",
            "|_____/ ",
            "        ",
        ]),
        'E' => Some(vec![
            "_______ ", "|  ___| ", "| |___  ", "|  ___| ", "| |___  ", "|_____| ",
        ]),
        'F' => Some(vec![
            "_______ ", "|  ___| ", "| |___  ", "|  ___| ", "| |     ", "|_|     ",
        ]),
        'G' => Some(vec![
            "  _____ ",
            " / ___ \\",
            "| |   |_|",
            "| |  __ ",
            "| |__|  |",
            " \\______/",
        ]),
        'H' => Some(vec![
            "_     _ ",
            "| |   | |",
            "| |___| |",
            "|  ___  |",
            "| |   | |",
            "|_|   |_|",
        ]),
        'I' => Some(vec![
            "_____ ", "|_   _|", "  | |  ", "  | |  ", " _| |_ ", "|_____|",
        ]),
        'J' => Some(vec![
            "    _ ", "   | |", "   | |", "_  | |", "| |_| |", " \\___/ ",
        ]),
        'K' => Some(vec![
            "_   __",
            "| | / /",
            "| |/ / ",
            "|   <  ",
            "| |\\ \\ ",
            "|_| \\_\\",
        ]),
        'L' => Some(vec![
            "_      ", "| |     ", "| |     ", "| |     ", "| |___  ", "|______/",
        ]),
        'M' => Some(vec![
            "__    __",
            "|\\ \\  / /|",
            "| \\ \\/ / |",
            "|  \\  /  |",
            "|   \\/   |",
            "|        |",
        ]),
        'N' => Some(vec![
            "_     _ ",
            "| \\   | |",
            "|  \\  | |",
            "|   \\ | |",
            "|    \\| |",
            "|     \\_|",
        ]),
        'O' => Some(vec![
            "  ____  ",
            " / __ \\ ",
            "| |  | |",
            "| |  | |",
            "| |__| |",
            " \\____/ ",
        ]),
        'P' => Some(vec![
            "______  ",
            "| ___ \\ ",
            "| |_/ / ",
            "|  __/  ",
            "| |     ",
            "|_|     ",
        ]),
        'Q' => Some(vec![
            "  ____  ",
            " / __ \\ ",
            "| |  | |",
            "| |  | |",
            "| |_\\| |",
            " \\___\\_\\",
        ]),
        'R' => Some(vec![
            "______  ",
            "| ___ \\ ",
            "| |_/ / ",
            "|    /  ",
            "| |\\ \\  ",
            "|_| \\_\\ ",
        ]),
        'S' => Some(vec![
            "  ____ ",
            " / ___|",
            "| \\___ ",
            " \\___  \\",
            " ___/ /",
            "|____/ ",
        ]),
        'T' => Some(vec![
            "_______ ",
            "|__   __|",
            "   | |   ",
            "   | |   ",
            "   | |   ",
            "   |_|   ",
        ]),
        'U' => Some(vec![
            "_     _ ",
            "| |   | |",
            "| |   | |",
            "| |   | |",
            "| |___| |",
            " \\_____/ ",
        ]),
        'V' => Some(vec![
            "_     _ ",
            "| |   | |",
            "| |   | |",
            " \\ \\ / / ",
            "  \\ V /  ",
            "   \\_/   ",
        ]),
        'W' => Some(vec![
            "_       _ ",
            "| |     | |",
            "| |  _  | |",
            "| | / \\ | |",
            "| |/ / \\| |",
            "|___/ \\___|",
        ]),
        'X' => Some(vec![
            "_     _ ",
            " \\ \\ / / ",
            "  \\ V /  ",
            "   > <   ",
            "  / . \\  ",
            " /_/ \\_\\ ",
        ]),
        'Y' => Some(vec![
            "_     _ ",
            " \\ \\ / / ",
            "  \\ V /  ",
            "   | |   ",
            "   | |   ",
            "   |_|   ",
        ]),
        'Z' => Some(vec![
            "_______ ",
            "|__   __|",
            "   / /   ",
            "  / /    ",
            " / /___  ",
            "|_______|",
        ]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    ", "    "]),
    }
}

fn alligator_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec!["      ", " /\\   ", "/--\\  ", "/  \\/\\", "      "]),
        'B' => Some(vec!["      ", "|\\  /|", "|/\\/\\|", "|\\  /|", "      "]),
        'C' => Some(vec![
            "      ",
            "/\\/\\/\\",
            "\\/    ",
            "/\\/\\/\\",
            "      ",
        ]),
        'D' => Some(vec!["      ", "|\\  /|", "| \\/ |", "|/\\/\\|", "      "]),
        'E' => Some(vec!["      ", "|/\\/\\|", "|/\\/  ", "|/\\/\\|", "      "]),
        'F' => Some(vec!["      ", "|/\\/\\|", "|/\\/  ", "|     ", "      "]),
        'G' => Some(vec!["      ", "/\\/\\/|", "\\/  \\/", "/\\/\\/|", "      "]),
        'H' => Some(vec!["      ", "|\\/\\/|", "|    |", "|/\\/\\|", "      "]),
        'I' => Some(vec!["     ", "/\\/\\ ", " /\\  ", "/\\/\\ ", "     "]),
        'J' => Some(vec!["      ", " /\\/\\|", "    /|", "/\\/\\||", "      "]),
        'K' => Some(vec!["     ", "|\\/\\ ", "|/\\  ", "|\\/\\ ", "     "]),
        'L' => Some(vec!["      ", "|     ", "|     ", "|/\\/\\|", "      "]),
        'M' => Some(vec![
            "       ",
            "|\\ /\\ /|",
            "| V  V |",
            "|      |",
            "       ",
        ]),
        'N' => Some(vec!["      ", "|\\  /|", "| \\/ |", "|    |", "      "]),
        'O' => Some(vec![
            "      ",
            "/\\/\\/\\",
            "\\/  \\/",
            "/\\/\\/\\",
            "      ",
        ]),
        'P' => Some(vec!["      ", "|/\\/\\|", "|/\\/  ", "|     ", "      "]),
        'Q' => Some(vec![
            "      ",
            "/\\/\\/\\",
            "\\/  \\/",
            "/\\/\\\\|",
            "      ",
        ]),
        'R' => Some(vec!["      ", "|/\\/\\|", "|/\\/  ", "|\\/\\ ", "      "]),
        'S' => Some(vec!["      ", " /\\/\\|", "|/\\/  ", "|/\\/\\ ", "      "]),
        'T' => Some(vec!["      ", "|/\\/\\|", "  /\\  ", "  \\/  ", "      "]),
        'U' => Some(vec!["      ", "|\\/\\/|", "|    |", " \\/\\/ ", "      "]),
        'V' => Some(vec!["     ", "|\\/\\|", " \\\\// ", "  \\/  ", "     "]),
        'W' => Some(vec![
            "       ",
            "|\\/\\/\\|",
            "| /\\/\\ |",
            " \\/  \\/ ",
            "       ",
        ]),
        'X' => Some(vec!["     ", "\\\\/\\/", " /\\/ ", "\\/\\/\\", "     "]),
        'Y' => Some(vec!["     ", "\\\\/\\/", " /\\/ ", "  /\\  ", "     "]),
        'Z' => Some(vec!["      ", "/\\/\\/|", "  /\\/ ", "|/\\/\\ ", "      "]),
        ' ' => Some(vec!["   ", "   ", "   ", "   ", "   "]),
        _ => Some(vec!["  ", "??", "??", "??", "  "]),
    }
}

fn roman_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            ".oOOOo.  ",
            ".O    o  ",
            "o      O ",
            "O.    .O ",
            " oOooOoOo",
            "       O ",
        ]),
        'B' => Some(vec![
            "OOOoOoo. ",
            "o     `O ",
            "O      o ",
            "o     .O ",
            "OOooOoo' ",
            "         ",
        ]),
        'C' => Some(vec![
            "  .oOOOo. ",
            " .O    o. ",
            " o        ",
            " o        ",
            " `OoooooO'",
            "          ",
        ]),
        'D' => Some(vec![
            "OOOoOoo. ",
            "o     `O ",
            "O      o ",
            "o     .O ",
            "OOooOoo' ",
            "         ",
        ]),
        'E' => Some(vec![
            "OOooOooO", "o       ", "O       ", "o       ", "OOooOooO", "        ",
        ]),
        'F' => Some(vec![
            "OOooOooO", "o       ", "O       ", "o       ", "O       ", "        ",
        ]),
        'G' => Some(vec![
            "  .oOOOo. ",
            " .O    o  ",
            " o      O ",
            " O.    .O ",
            " `oOooOoo ",
            "        O ",
        ]),
        'H' => Some(vec![
            "o      O", "O      o", "oOooOOoo", "O      O", "o      o", "        ",
        ]),
        'I' => Some(vec![
            "oOoOOoOo", "   o    ", "   O    ", "   o    ", "oOoOOoOo", "        ",
        ]),
        'J' => Some(vec![
            "      O", "      o", "      O", "o    .O", "`oooO' ", "       ",
        ]),
        'K' => Some(vec![
            "o   .O ", "O  .o  ", "oOo    ", "O  `o  ", "o   .O ", "       ",
        ]),
        'L' => Some(vec![
            "o      ", "O      ", "o      ", "O      ", "OooOOoo", "       ",
        ]),
        'M' => Some(vec![
            "O     o ", "Oo   oO ", "O O O o ", "o  `o  O", "O     O ", "        ",
        ]),
        'N' => Some(vec![
            "o\\    o ",
            "O `o  O ",
            "o  `O o ",
            "O   `oO ",
            "O     o ",
            "        ",
        ]),
        'O' => Some(vec![
            "  .oOOOo. ",
            " .O    Oo ",
            " o      O ",
            " O      o ",
            " `OoooOO' ",
            "          ",
        ]),
        'P' => Some(vec![
            "OOOoOoo.", "O     `o", "o     .O", "OOooOO' ", "o       ", "        ",
        ]),
        'Q' => Some(vec![
            "  .oOOOo.  ",
            " .O    oO  ",
            " o      O  ",
            " O    oO' O",
            " `OoooOO  o",
            "           ",
        ]),
        'R' => Some(vec![
            "OOOoOoo. ",
            "O     `O ",
            "o     .O ",
            "OOooOO'  ",
            "o    `o  ",
            "         ",
        ]),
        'S' => Some(vec![
            "  .oOOOo.",
            " .O    o ",
            " `o      ",
            " .oOoO'  ",
            "o     .o ",
            "         ",
        ]),
        'T' => Some(vec![
            "oOoOOoOOo",
            "    o    ",
            "    O    ",
            "    o    ",
            "    O    ",
            "         ",
        ]),
        'U' => Some(vec![
            "o      O", "O      o", "o      O", "O      o", " oOooOo ", "        ",
        ]),
        'V' => Some(vec![
            "O      o", "o      O", " O    o ", "  o  O  ", "   Oo   ", "        ",
        ]),
        'W' => Some(vec![
            "O          o",
            "o          O",
            "O   .Oo.   o",
            " o.O'  'O.o ",
            "  O'    'O  ",
            "            ",
        ]),
        'X' => Some(vec![
            "o     O", " O   o ", "  oOo  ", " O   o ", "o     O", "       ",
        ]),
        'Y' => Some(vec![
            "o     O", " O   o ", "  ooo  ", "   O   ", "   o   ", "       ",
        ]),
        'Z' => Some(vec![
            "OooOOoo", "     o ", "   .O  ", "  oOo  ", "OOOooOO", "       ",
        ]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    ", "    "]),
    }
}

fn thick_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "   AA   ", "  AAAA  ", " AA  AA ", " AAAAAA ", "AA    AA",
        ]),
        'B' => Some(vec!["BBBBB  ", "BB   BB", "BBBBB  ", "BB   BB", "BBBBB  "]),
        'C' => Some(vec![" CCCCC ", "CC     ", "CC     ", "CC     ", " CCCCC "]),
        'D' => Some(vec!["DDDDD  ", "DD   DD", "DD   DD", "DD   DD", "DDDDD  "]),
        'E' => Some(vec!["EEEEEE", "EE    ", "EEEE  ", "EE    ", "EEEEEE"]),
        'F' => Some(vec!["FFFFFF", "FF    ", "FFFF  ", "FF    ", "FF    "]),
        'G' => Some(vec![" GGGGG ", "GG     ", "GG  GGG", "GG   GG", " GGGGG "]),
        'H' => Some(vec!["HH   HH", "HH   HH", "HHHHHHH", "HH   HH", "HH   HH"]),
        'I' => Some(vec!["IIIII", "  II ", "  II ", "  II ", "IIIII"]),
        'J' => Some(vec!["    JJ", "    JJ", "    JJ", "JJ  JJ", " JJJJ "]),
        'K' => Some(vec!["KK  KK", "KK KK ", "KKKK  ", "KK KK ", "KK  KK"]),
        'L' => Some(vec!["LL    ", "LL    ", "LL    ", "LL    ", "LLLLLL"]),
        'M' => Some(vec![
            "MM    MM", "MMM  MMM", "MM MM MM", "MM    MM", "MM    MM",
        ]),
        'N' => Some(vec!["NN   NN", "NNN  NN", "NN N NN", "NN  NNN", "NN   NN"]),
        'O' => Some(vec![" OOOOO ", "OO   OO", "OO   OO", "OO   OO", " OOOOO "]),
        'P' => Some(vec!["PPPPP ", "PP   PP", "PPPPP ", "PP    ", "PP    "]),
        'Q' => Some(vec![" QQQQQ ", "QQ   QQ", "QQ Q QQ", "QQ  QQ ", " QQQ QQ"]),
        'R' => Some(vec!["RRRRR ", "RR   RR", "RRRRR ", "RR RR ", "RR  RR"]),
        'S' => Some(vec![" SSSSS", "SS    ", " SSSS ", "    SS", "SSSSS "]),
        'T' => Some(vec!["TTTTTT", "  TT  ", "  TT  ", "  TT  ", "  TT  "]),
        'U' => Some(vec!["UU   UU", "UU   UU", "UU   UU", "UU   UU", " UUUUU "]),
        'V' => Some(vec!["VV   VV", "VV   VV", " VV VV ", " VV VV ", "  VVV  "]),
        'W' => Some(vec![
            "WW    WW", "WW    WW", "WW WW WW", "WWW  WWW", "WW    WW",
        ]),
        'X' => Some(vec!["XX   XX", " XX XX ", "  XXX  ", " XX XX ", "XX   XX"]),
        'Y' => Some(vec!["YY   YY", " YY YY ", "  YYY  ", "  YY   ", "  YY   "]),
        'Z' => Some(vec!["ZZZZZZ", "   ZZ ", "  ZZ  ", " ZZ   ", "ZZZZZZ"]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    "]),
    }
}

fn ogre_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "        _ ",
            "       / \\",
            "      / _ \\",
            "     / ___ \\",
            "    /_/   \\_\\",
            "            ",
            "            ",
        ]),
        'B' => Some(vec![
            "  ____  ",
            " | __ ) ",
            " |  _ \\ ",
            " | |_) |",
            " |____/ ",
            "        ",
            "        ",
        ]),
        'C' => Some(vec![
            "   ____ ",
            "  / ___|",
            " | |    ",
            " | |___ ",
            "  \\____|",
            "        ",
            "        ",
        ]),
        'D' => Some(vec![
            "  ____  ",
            " |  _ \\ ",
            " | | | |",
            " | |_| |",
            " |____/ ",
            "        ",
            "        ",
        ]),
        'E' => Some(vec![
            "  _____ ", " | ____|", " |  _|  ", " | |___ ", " |_____|", "        ", "        ",
        ]),
        'F' => Some(vec![
            "  _____ ", " |  ___|", " | |_   ", " |  _|  ", " |_|    ", "        ", "        ",
        ]),
        'G' => Some(vec![
            "   ____ ",
            "  / ___|",
            " | |  _ ",
            " | |_| |",
            "  \\____|",
            "        ",
            "        ",
        ]),
        'H' => Some(vec![
            "  _   _ ", " | | | |", " | |_| |", " |  _  |", " |_| |_|", "        ", "        ",
        ]),
        'I' => Some(vec![
            "  ___ ", " |_ _|", "  | | ", "  | | ", " |___|", "      ", "      ",
        ]),
        'J' => Some(vec![
            "      _ ",
            "     | |",
            "  _  | |",
            " | |_| |",
            "  \\___/ ",
            "        ",
            "        ",
        ]),
        'K' => Some(vec![
            "  _  __",
            " | |/ /",
            " | ' / ",
            " | . \\ ",
            " |_|\\_\\",
            "       ",
            "       ",
        ]),
        'L' => Some(vec![
            "  _     ", " | |    ", " | |    ", " | |___ ", " |_____|", "        ", "        ",
        ]),
        'M' => Some(vec![
            "  __  __ ",
            " |  \\/  |",
            " | |\\/| |",
            " | |  | |",
            " |_|  |_|",
            "         ",
            "         ",
        ]),
        'N' => Some(vec![
            "  _   _ ",
            " | \\ | |",
            " |  \\| |",
            " | |\\  |",
            " |_| \\_|",
            "        ",
            "        ",
        ]),
        'O' => Some(vec![
            "   ___  ",
            "  / _ \\ ",
            " | | | |",
            " | |_| |",
            "  \\___/ ",
            "        ",
            "        ",
        ]),
        'P' => Some(vec![
            "  ____  ",
            " |  _ \\ ",
            " | |_) |",
            " |  __/ ",
            " |_|    ",
            "        ",
            "        ",
        ]),
        'Q' => Some(vec![
            "   ___  ",
            "  / _ \\ ",
            " | | | |",
            " | |_| |",
            "  \\___\\_\\",
            "        ",
            "        ",
        ]),
        'R' => Some(vec![
            "  ____  ",
            " |  _ \\ ",
            " | |_) |",
            " |  _ < ",
            " |_| \\_\\",
            "        ",
            "        ",
        ]),
        'S' => Some(vec![
            "  ____  ",
            " / ___| ",
            " \\___ \\ ",
            "  ___) |",
            " |____/ ",
            "        ",
            "        ",
        ]),
        'T' => Some(vec![
            "  _____ ", " |_   _|", "   | |  ", "   | |  ", "   |_|  ", "        ", "        ",
        ]),
        'U' => Some(vec![
            "  _   _ ",
            " | | | |",
            " | | | |",
            " | |_| |",
            "  \\___/ ",
            "        ",
            "        ",
        ]),
        'V' => Some(vec![
            "__     __",
            "\\ \\   / /",
            " \\ \\ / / ",
            "  \\ V /  ",
            "   \\_/   ",
            "         ",
            "         ",
        ]),
        'W' => Some(vec![
            "__        __",
            "\\ \\      / /",
            " \\ \\ /\\ / / ",
            "  \\ V  V /  ",
            "   \\_/\\_/   ",
            "            ",
            "            ",
        ]),
        'X' => Some(vec![
            "__  __", "\\ \\/ /", " \\  / ", " /  \\ ", "/_/\\_\\", "      ", "      ",
        ]),
        'Y' => Some(vec![
            "__   __",
            "\\ \\ / /",
            " \\ V / ",
            "  | |  ",
            "  |_|  ",
            "       ",
            "       ",
        ]),
        'Z' => Some(vec![
            "  _____", " |__  /", "   / / ", "  / /_ ", " /____|", "       ", "       ",
        ]),
        ' ' => Some(vec![
            "     ", "     ", "     ", "     ", "     ", "     ", "     ",
        ]),
        _ => Some(vec![
            "     ", " ??? ", "     ", " ??? ", "     ", "     ", "     ",
        ]),
    }
}

fn ivrit_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec![
            "  ___   ",
            " /   \\  ",
            "|  _  | ",
            "| | | | ",
            "|_| |_| ",
        ]),
        'B' => Some(vec!["______ ", "|   __)", "|  |_  ", "|   __)", "|______)"]),
        'C' => Some(vec![" _____ ", "( ___ )", "|  ___|", "| |___ ", " \\___/ "]),
        'D' => Some(vec![
            "______ ", "(  __  )", "| |  | |", "| |__| |", "(______)",
        ]),
        'E' => Some(vec![
            "______ ", "|  ____|", "| |____ ", "|  ____|", "|______|",
        ]),
        'F' => Some(vec![
            "______ ", "|  ____|", "| |____ ", "|  ____|", "|_|     ",
        ]),
        'G' => Some(vec![
            " _____ ",
            "( ___ )",
            "|   __| ",
            "| |___  ",
            " \\____| ",
        ]),
        'H' => Some(vec![
            "_    _ ", "| |  | |", "|_|__|_|", "| |  | |", "|_|  |_|",
        ]),
        'I' => Some(vec!["______ ", " |  |  ", " |  |  ", " |  |  ", " |__|  "]),
        'J' => Some(vec!["  ____ ", "    |  ", "    |  ", " |  |  ", " |__|  "]),
        'K' => Some(vec![
            "_   __ ",
            "| | / /",
            "|  < < ",
            "| |\\  \\",
            "|_| \\_\\",
        ]),
        'L' => Some(vec![
            "_      ", "| |     ", "| |     ", "| |____ ", "|______|",
        ]),
        'M' => Some(vec![
            "__   __ ", "| |_| | ", "|  _  | ", "| | | | ", "|_| |_| ",
        ]),
        'N' => Some(vec![
            "__    _ ",
            "| \\  | |",
            "|  \\ | |",
            "| |\\ \\| |",
            "|_| \\__|",
        ]),
        'O' => Some(vec![" _____ ", "|  _  |", "| |_| |", "|  _  |", "|_| |_|"]),
        'P' => Some(vec![
            "______ ", "|   __ )", "|  |__) ", "|   ___/", "|__|    ",
        ]),
        'Q' => Some(vec![
            " _____ ",
            "|  _  |",
            "| |_| |",
            "|  _\\_\\",
            "|_|  \\_\\",
        ]),
        'R' => Some(vec![
            "______ ",
            "|   __ )",
            "|  |__) ",
            "|   __/ ",
            "|__|  \\ ",
        ]),
        'S' => Some(vec![
            " ____  ",
            "/  __\\ ",
            "\\___  \\",
            " ___/  /",
            "/_____/ ",
        ]),
        'T' => Some(vec!["______ ", "  |  | ", "  |  | ", "  |  | ", "  |__| "]),
        'U' => Some(vec![
            "_    _ ",
            "| |  | |",
            "| |  | |",
            "| |__| |",
            " \\____/ ",
        ]),
        'V' => Some(vec![
            "_    _ ",
            "| |  | |",
            " \\ \\/ / ",
            "  \\  /  ",
            "   \\/   ",
        ]),
        'W' => Some(vec![
            "__    __ ",
            "| |  | | ",
            "| |/\\| | ",
            "|  /\\  | ",
            "|_/  \\_| ",
        ]),
        'X' => Some(vec!["_   _ ", " \\ / ", "  X  ", " / \\ ", "|_| |_|"]),
        'Y' => Some(vec!["_   _ ", " \\ / ", "  |  ", "  |  ", "  |  "]),
        'Z' => Some(vec!["______ ", "    / /", "   / / ", "  / /  ", "/____|  "]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    "]),
    }
}

fn rectangles_glyph(ch: char) -> Option<Vec<&'static str>> {
    match ch.to_ascii_uppercase() {
        'A' => Some(vec!["┌───┐", "│┌─┐│", "│└─┘│", "│   │", "└   ┘"]),
        'B' => Some(vec!["┌───┐", "│┌─┘│", "│└─┐│", "│┌─┘│", "└───┘"]),
        'C' => Some(vec!["┌────", "│    ", "│    ", "│    ", "└────"]),
        'D' => Some(vec!["┌───┐", "│   │", "│   │", "│   │", "└───┘"]),
        'E' => Some(vec!["┌────", "│───┐", "│    ", "│───┘", "└────"]),
        'F' => Some(vec!["┌────", "│───┐", "│    ", "│    ", "└    "]),
        'G' => Some(vec!["┌────", "│    ", "│ ───", "│   │", "└───┘"]),
        'H' => Some(vec!["┌   ┐", "│   │", "│───│", "│   │", "└   ┘"]),
        'I' => Some(vec!["────┐", "  │ │", "  │ │", "  │ │", "────┘"]),
        'J' => Some(vec!["────┐", "   ││", "   ││", "┌──┘│", "└───┘"]),
        'K' => Some(vec!["┌  ┌┘", "│ ┌┘ ", "│─┤  ", "│ └┐ ", "└  └┐"]),
        'L' => Some(vec!["┌    ", "│    ", "│    ", "│    ", "└────"]),
        'M' => Some(vec!["┌─┐ ┌─┐", "│ └┬┘ │", "│     │", "│     │", "└     ┘"]),
        'N' => Some(vec!["┌─┐  ┐", "│ └┐ │", "│  └┐│", "│   └│", "└    ┘"]),
        'O' => Some(vec!["┌───┐", "│   │", "│   │", "│   │", "└───┘"]),
        'P' => Some(vec!["┌───┐", "│   │", "│───┘", "│    ", "└    "]),
        'Q' => Some(vec!["┌───┐", "│   │", "│   │", "│  ─┤", "└───┴"]),
        'R' => Some(vec!["┌───┐", "│   │", "│───┤", "│  └┐", "└   └"]),
        'S' => Some(vec!["┌────", "│    ", "└───┐", "    │", "────┘"]),
        'T' => Some(vec![
            "────┬────",
            "    │    ",
            "    │    ",
            "    │    ",
            "    └    ",
        ]),
        'U' => Some(vec!["┌   ┐", "│   │", "│   │", "│   │", "└───┘"]),
        'V' => Some(vec!["┐   ┌", "│   │", "│   │", "└┐ ┌┘", " └─┘ "]),
        'W' => Some(vec!["┐     ┌", "│     │", "│  │  │", "│ ─┼─ │", "└──┴──┘"]),
        'X' => Some(vec!["┐   ┌", "└┐ ┌┘", " └─┘ ", "┌┴─┴┐", "┘   └"]),
        'Y' => Some(vec!["┐   ┌", "└┐ ┌┘", " └─┘ ", "  │  ", "  └  "]),
        'Z' => Some(vec!["────┐", "  ┌─┘", " ┌┘  ", "┌┘   ", "└────"]),
        ' ' => Some(vec!["    ", "    ", "    ", "    ", "    "]),
        _ => Some(vec!["    ", " ?? ", "    ", " ?? ", "    "]),
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
        let arts = [
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
