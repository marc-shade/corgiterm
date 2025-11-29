//! Inline Images support (Kitty Graphics Protocol and Sixel)
//!
//! Provides support for displaying images inline within terminal output.
//! Implements both the Kitty Graphics Protocol (primary) and Sixel (legacy).

use std::collections::HashMap;

/// Unique identifier for inline images
pub type ImageId = u32;

/// Image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Raw RGBA pixel data (32-bit)
    Rgba,
    /// PNG encoded image
    Png,
    /// Sixel format
    Sixel,
}

/// Image placement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementMode {
    /// Place at current cursor position
    AtCursor,
    /// Place at absolute cell position
    AtCell { row: usize, col: usize },
    /// Virtual placement (referenced by ID)
    Virtual,
}

/// An inline image stored in the terminal
#[derive(Debug, Clone)]
pub struct InlineImage {
    /// Unique image ID
    pub id: ImageId,
    /// Image data (decoded RGBA)
    pub data: Vec<u8>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in u32
    pub height: u32,
    /// Display width in cells (0 = auto)
    pub cell_width: u32,
    /// Display height in cells (0 = auto)
    pub cell_height: u32,
    /// X offset within cell (pixels)
    pub x_offset: u32,
    /// Y offset within cell (pixels)
    pub y_offset: u32,
    /// Z-index for layering
    pub z_index: i32,
}

/// Image placement on the terminal grid
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// Image ID to display
    pub image_id: ImageId,
    /// Row position (top)
    pub row: usize,
    /// Column position (left)
    pub col: usize,
    /// Width in cells
    pub width_cells: usize,
    /// Height in cells
    pub height_cells: usize,
    /// Visible (can be hidden temporarily)
    pub visible: bool,
}

/// Kitty Graphics Protocol command action
#[derive(Debug, Clone)]
pub enum KittyAction {
    /// Transmit image data
    Transmit,
    /// Transmit and display
    TransmitAndDisplay,
    /// Query terminal capabilities
    Query,
    /// Display a transmitted image
    Display,
    /// Delete images
    Delete,
    /// Animate frames
    Animate,
    /// Compose image
    Compose,
}

/// Kitty Graphics Protocol command parser state
#[derive(Debug, Default)]
pub struct KittyParser {
    /// Current command parameters
    params: HashMap<char, String>,
    /// Accumulated base64 data
    data_buffer: Vec<u8>,
    /// Is currently in data mode
    in_data_mode: bool,
    /// Current payload chunk
    payload: Vec<u8>,
}

impl KittyParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a Kitty graphics protocol APC sequence
    /// Format: ESC_G<key>=<value>,<key>=<value>,...;<base64data>
    pub fn parse(&mut self, data: &[u8]) -> Option<KittyCommand> {
        // Split on ; to separate params from payload
        let data_str = String::from_utf8_lossy(data);
        let (params_str, payload_str) = match data_str.find(';') {
            Some(idx) => (&data_str[..idx], Some(&data_str[idx + 1..])),
            None => (data_str.as_ref(), None),
        };

        // Parse key=value pairs
        self.params.clear();
        for pair in params_str.split(',') {
            if let Some((key, value)) = pair.split_once('=') {
                if let Some(k) = key.chars().next() {
                    self.params.insert(k, value.to_string());
                }
            }
        }

        // Decode base64 payload if present
        let payload = payload_str.and_then(|p| {
            use base64::{engine::general_purpose::STANDARD, Engine};
            STANDARD.decode(p.as_bytes()).ok()
        });

        // Build command
        let action = match self.get_char('a').unwrap_or('t') {
            't' => KittyAction::Transmit,
            'T' => KittyAction::TransmitAndDisplay,
            'q' => KittyAction::Query,
            'p' => KittyAction::Display,
            'd' => KittyAction::Delete,
            'a' => KittyAction::Animate,
            'c' => KittyAction::Compose,
            _ => KittyAction::Transmit,
        };

        let format = match self.get_int('f').unwrap_or(32) {
            24 => ImageFormat::Rgba, // RGB
            32 => ImageFormat::Rgba, // RGBA
            100 => ImageFormat::Png,
            _ => ImageFormat::Rgba,
        };

        Some(KittyCommand {
            action,
            format,
            id: self.get_int('i'),
            width: self.get_int('s'),
            height: self.get_int('v'),
            cell_width: self.get_int('c'),
            cell_height: self.get_int('r'),
            x_offset: self.get_int('X'),
            y_offset: self.get_int('Y'),
            z_index: self.get_int('z').map(|z| z as i32),
            more_data: self.get_int('m') == Some(1),
            placement_id: self.get_int('p'),
            quiet: self.get_int('q'),
            payload,
        })
    }

    fn get_char(&self, key: char) -> Option<char> {
        self.params.get(&key).and_then(|v| v.chars().next())
    }

    fn get_int(&self, key: char) -> Option<u32> {
        self.params.get(&key).and_then(|v| v.parse().ok())
    }
}

/// Parsed Kitty graphics command
#[derive(Debug, Clone)]
pub struct KittyCommand {
    /// Action to perform
    pub action: KittyAction,
    /// Image format
    pub format: ImageFormat,
    /// Image ID
    pub id: Option<u32>,
    /// Image width in pixels
    pub width: Option<u32>,
    /// Image height in pixels
    pub height: Option<u32>,
    /// Display width in cells
    pub cell_width: Option<u32>,
    /// Display height in cells
    pub cell_height: Option<u32>,
    /// X offset in pixels
    pub x_offset: Option<u32>,
    /// Y offset in pixels
    pub y_offset: Option<u32>,
    /// Z-index for layering
    pub z_index: Option<i32>,
    /// More data chunks coming
    pub more_data: bool,
    /// Placement ID
    pub placement_id: Option<u32>,
    /// Quiet mode (0=normal, 1=no OK, 2=no errors)
    pub quiet: Option<u32>,
    /// Image data payload (decoded from base64)
    pub payload: Option<Vec<u8>>,
}

/// Sixel parser state
#[derive(Debug, Default)]
pub struct SixelParser {
    /// Accumulated sixel data
    data: Vec<u8>,
    /// Color palette (index -> RGB)
    palette: HashMap<u8, (u8, u8, u8)>,
    /// Current color index
    current_color: u8,
    /// Image width (derived from data)
    width: u32,
    /// Image height (derived from data)
    height: u32,
    /// X position during parsing
    x: u32,
    /// Y position during parsing
    y: u32,
}

impl SixelParser {
    pub fn new() -> Self {
        // Initialize with default VGA palette
        let mut palette = HashMap::new();
        for i in 0..16u8 {
            let (r, g, b) = default_vga_color(i);
            palette.insert(i, (r, g, b));
        }
        Self {
            palette,
            ..Default::default()
        }
    }

    /// Parse Sixel DCS sequence data
    /// Sixel format: ESC P <params> q <sixel-data> ESC \
    pub fn parse(&mut self, data: &[u8]) -> Option<SixelImage> {
        self.data.clear();
        self.x = 0;
        self.y = 0;
        self.width = 0;
        self.height = 0;

        // Sixel data starts after 'q' character
        let sixel_start = data.iter().position(|&b| b == b'q')?;
        let sixel_data = &data[sixel_start + 1..];

        // First pass: determine dimensions and parse colors
        self.first_pass(sixel_data);

        // Second pass: generate pixel data
        let pixels = self.second_pass(sixel_data);

        Some(SixelImage {
            width: self.width,
            height: self.height,
            data: pixels,
        })
    }

    fn first_pass(&mut self, data: &[u8]) {
        let mut i = 0;
        let mut max_x = 0u32;
        let mut max_y = 0u32;
        let mut x = 0u32;
        let mut y = 0u32;

        while i < data.len() {
            match data[i] {
                // Color introducer #
                b'#' => {
                    i += 1;
                    let (color_idx, consumed) = parse_number(&data[i..]);
                    i += consumed;

                    // Check for color definition (HLS or RGB)
                    if i < data.len() && data[i] == b';' {
                        i += 1;
                        let (coord_type, consumed) = parse_number(&data[i..]);
                        i += consumed;
                        if i < data.len() && data[i] == b';' {
                            i += 1;
                            let (p1, c1) = parse_number(&data[i..]);
                            i += c1;
                            if i < data.len() && data[i] == b';' {
                                i += 1;
                                let (p2, c2) = parse_number(&data[i..]);
                                i += c2;
                                if i < data.len() && data[i] == b';' {
                                    i += 1;
                                    let (p3, c3) = parse_number(&data[i..]);
                                    i += c3;

                                    // Convert to RGB
                                    let (r, g, b) = if coord_type == 2 {
                                        // RGB (0-100 scale)
                                        ((p1 * 255 / 100) as u8, (p2 * 255 / 100) as u8, (p3 * 255 / 100) as u8)
                                    } else {
                                        // HLS - convert to RGB
                                        hls_to_rgb(p1, p2, p3)
                                    };
                                    self.palette.insert(color_idx as u8, (r, g, b));
                                }
                            }
                        }
                    }
                }
                // Carriage return $
                b'$' => {
                    max_x = max_x.max(x);
                    x = 0;
                    i += 1;
                }
                // New line -
                b'-' => {
                    max_x = max_x.max(x);
                    x = 0;
                    y += 6; // Sixel is 6 pixels high
                    i += 1;
                }
                // Repeat !<count><char>
                b'!' => {
                    i += 1;
                    let (count, consumed) = parse_number(&data[i..]);
                    i += consumed;
                    if i < data.len() && data[i] >= 0x3F && data[i] <= 0x7E {
                        x += count;
                        i += 1;
                    }
                }
                // Sixel character (0x3F to 0x7E)
                c if c >= 0x3F && c <= 0x7E => {
                    x += 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        max_x = max_x.max(x);
        max_y = y + 6;

        self.width = max_x;
        self.height = max_y;
    }

    fn second_pass(&self, data: &[u8]) -> Vec<u8> {
        let mut pixels = vec![0u8; (self.width * self.height * 4) as usize];
        let mut x = 0u32;
        let mut y = 0u32;
        let mut current_color = 0u8;
        let mut i = 0;

        while i < data.len() {
            match data[i] {
                b'#' => {
                    i += 1;
                    let (color_idx, consumed) = parse_number(&data[i..]);
                    i += consumed;
                    current_color = color_idx as u8;

                    // Skip color definition if present
                    while i < data.len() && data[i] == b';' {
                        i += 1;
                        let (_, consumed) = parse_number(&data[i..]);
                        i += consumed;
                    }
                }
                b'$' => {
                    x = 0;
                    i += 1;
                }
                b'-' => {
                    x = 0;
                    y += 6;
                    i += 1;
                }
                b'!' => {
                    i += 1;
                    let (count, consumed) = parse_number(&data[i..]);
                    i += consumed;
                    if i < data.len() && data[i] >= 0x3F && data[i] <= 0x7E {
                        let sixel = data[i] - 0x3F;
                        for _ in 0..count {
                            self.draw_sixel(&mut pixels, x, y, sixel, current_color);
                            x += 1;
                        }
                        i += 1;
                    }
                }
                c if c >= 0x3F && c <= 0x7E => {
                    let sixel = c - 0x3F;
                    self.draw_sixel(&mut pixels, x, y, sixel, current_color);
                    x += 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        pixels
    }

    fn draw_sixel(&self, pixels: &mut [u8], x: u32, y: u32, sixel: u8, color: u8) {
        let (r, g, b) = self.palette.get(&color).copied().unwrap_or((255, 255, 255));

        for bit in 0..6 {
            if sixel & (1 << bit) != 0 {
                let py = y + bit as u32;
                if x < self.width && py < self.height {
                    let idx = ((py * self.width + x) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        pixels[idx] = r;
                        pixels[idx + 1] = g;
                        pixels[idx + 2] = b;
                        pixels[idx + 3] = 255;
                    }
                }
            }
        }
    }
}

/// Parsed Sixel image
#[derive(Debug, Clone)]
pub struct SixelImage {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// RGBA pixel data
    pub data: Vec<u8>,
}

/// Image storage for terminal
#[derive(Debug, Default)]
pub struct ImageStore {
    /// Stored images by ID
    images: HashMap<ImageId, InlineImage>,
    /// Image placements on screen
    placements: Vec<ImagePlacement>,
    /// Next available image ID
    next_id: ImageId,
    /// Partial image data being assembled
    partial_data: HashMap<ImageId, Vec<u8>>,
}

impl ImageStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a new image
    pub fn add_image(&mut self, image: InlineImage) -> ImageId {
        let id = image.id;
        self.images.insert(id, image);
        id
    }

    /// Get an image by ID
    pub fn get_image(&self, id: ImageId) -> Option<&InlineImage> {
        self.images.get(&id)
    }

    /// Remove an image
    pub fn remove_image(&mut self, id: ImageId) {
        self.images.remove(&id);
        self.placements.retain(|p| p.image_id != id);
    }

    /// Add a placement
    pub fn add_placement(&mut self, placement: ImagePlacement) {
        self.placements.push(placement);
    }

    /// Get all visible placements
    pub fn visible_placements(&self) -> impl Iterator<Item = &ImagePlacement> {
        self.placements.iter().filter(|p| p.visible)
    }

    /// Get placements at a specific cell
    pub fn placements_at(&self, row: usize, col: usize) -> Vec<&ImagePlacement> {
        self.placements
            .iter()
            .filter(|p| {
                p.visible
                    && row >= p.row
                    && row < p.row + p.height_cells
                    && col >= p.col
                    && col < p.col + p.width_cells
            })
            .collect()
    }

    /// Generate next image ID
    pub fn next_id(&mut self) -> ImageId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Append partial data for multi-chunk transmission
    pub fn append_partial(&mut self, id: ImageId, data: Vec<u8>) {
        self.partial_data
            .entry(id)
            .or_insert_with(Vec::new)
            .extend(data);
    }

    /// Get and clear partial data
    pub fn take_partial(&mut self, id: ImageId) -> Option<Vec<u8>> {
        self.partial_data.remove(&id)
    }

    /// Clear all images
    pub fn clear(&mut self) {
        self.images.clear();
        self.placements.clear();
        self.partial_data.clear();
    }

    /// Clear images outside visible area (garbage collection)
    pub fn gc(&mut self, visible_rows: std::ops::Range<usize>) {
        // Remove placements outside visible area
        self.placements.retain(|p| {
            p.row + p.height_cells > visible_rows.start && p.row < visible_rows.end
        });

        // Remove unreferenced images
        let referenced: std::collections::HashSet<ImageId> =
            self.placements.iter().map(|p| p.image_id).collect();
        self.images.retain(|id, _| referenced.contains(id));
    }
}

// Helper functions

fn parse_number(data: &[u8]) -> (u32, usize) {
    let mut value = 0u32;
    let mut i = 0;
    while i < data.len() && data[i].is_ascii_digit() {
        value = value * 10 + (data[i] - b'0') as u32;
        i += 1;
    }
    (value, i)
}

fn default_vga_color(index: u8) -> (u8, u8, u8) {
    match index {
        0 => (0, 0, 0),       // Black
        1 => (128, 0, 0),     // Dark Red
        2 => (0, 128, 0),     // Dark Green
        3 => (128, 128, 0),   // Dark Yellow
        4 => (0, 0, 128),     // Dark Blue
        5 => (128, 0, 128),   // Dark Magenta
        6 => (0, 128, 128),   // Dark Cyan
        7 => (192, 192, 192), // Light Gray
        8 => (128, 128, 128), // Dark Gray
        9 => (255, 0, 0),     // Red
        10 => (0, 255, 0),    // Green
        11 => (255, 255, 0),  // Yellow
        12 => (0, 0, 255),    // Blue
        13 => (255, 0, 255),  // Magenta
        14 => (0, 255, 255),  // Cyan
        15 => (255, 255, 255), // White
        _ => (0, 0, 0),
    }
}

fn hls_to_rgb(h: u32, l: u32, s: u32) -> (u8, u8, u8) {
    // HLS values are 0-360 for H, 0-100 for L and S
    let h = h as f64;
    let l = l as f64 / 100.0;
    let s = s as f64 / 100.0;

    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_k = h / 360.0;

    let t_r = (h_k + 1.0 / 3.0) % 1.0;
    let t_g = h_k;
    let t_b = (h_k - 1.0 / 3.0 + 1.0) % 1.0;

    fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    }

    let r = (hue_to_rgb(p, q, t_r) * 255.0) as u8;
    let g = (hue_to_rgb(p, q, t_g) * 255.0) as u8;
    let b = (hue_to_rgb(p, q, t_b) * 255.0) as u8;

    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kitty_parser_basic() {
        let mut parser = KittyParser::new();
        let cmd = parser.parse(b"a=t,f=100,s=64,v=64;iVBORw0KGgo=").unwrap();
        assert!(matches!(cmd.action, KittyAction::Transmit));
        assert!(matches!(cmd.format, ImageFormat::Png));
        assert_eq!(cmd.width, Some(64));
        assert_eq!(cmd.height, Some(64));
    }

    #[test]
    fn test_image_store() {
        let mut store = ImageStore::new();
        let id = store.next_id();
        let image = InlineImage {
            id,
            data: vec![255; 16],
            width: 2,
            height: 2,
            cell_width: 1,
            cell_height: 1,
            x_offset: 0,
            y_offset: 0,
            z_index: 0,
        };
        store.add_image(image);
        assert!(store.get_image(id).is_some());
        store.remove_image(id);
        assert!(store.get_image(id).is_none());
    }
}
