//! GPU-accelerated terminal renderer
//!
//! Uses wgpu for GPU access and glyphon for text rendering.
//! Inspired by foot and Alacritty for performance optimizations.

use crate::ansi::AnsiPalette;
use crate::cell::{CellFlags, Color, Rgb};
use crate::grid::{Cursor, Grid};
use crate::selection::Selection;
use crate::{Result, TerminalError, TerminalSize};
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics, Resolution, Shaping,
    Style, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};
use wgpu::{
    Backends, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features,
    Instance, InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
};

/// Renderer configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Font family name
    pub font_family: String,
    /// Font size in points
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Default foreground color
    pub fg_color: Rgb,
    /// Default background color
    pub bg_color: Rgb,
    /// Cursor color
    pub cursor_color: Rgb,
    /// Selection color
    pub selection_color: Rgb,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            font_family: "Source Code Pro".to_string(),
            font_size: 14.0,
            line_height: 1.2,
            fg_color: Rgb::new(0xE0, 0xE0, 0xE0),
            bg_color: Rgb::new(0x1E, 0x1E, 0x1E),
            cursor_color: Rgb::new(0xFF, 0xFF, 0xFF),
            selection_color: Rgb::new(0x40, 0x60, 0x80),
        }
    }
}

/// GPU renderer for the terminal
pub struct GpuRenderer {
    // WGPU state
    device: Device,
    queue: Queue,
    surface: Option<Surface<'static>>,
    surface_config: SurfaceConfiguration,

    // Glyphon text rendering
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    viewport: Viewport,

    // Text buffers for each line
    buffers: Vec<Buffer>,

    // Terminal state
    size: TerminalSize,
    palette: AnsiPalette,
    config: RendererConfig,

    // Cached metrics
    cell_width: f32,
    cell_height: f32,

    // Damage tracking
    buffer_dirty: Vec<bool>,
    force_full_redraw: bool,
    last_stats: RenderStats,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        // Create wgpu instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        // Get adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| TerminalError::RendererInit("No suitable GPU adapter found".into()))?;

        // Request device
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("CorgiTerm Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_webgl2_defaults(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| TerminalError::RendererInit(format!("Device request failed: {}", e)))?;

        // Surface configuration (will be updated when surface is set)
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        // Initialize font system
        let mut font_system = FontSystem::new();

        // Load system fonts
        font_system.db_mut().load_system_fonts();

        // Create text rendering components
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surface_config.format);
        let text_renderer =
            TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);

        let viewport = Viewport::new(&device, &cache);

        // Use default config - can be updated later
        let config = RendererConfig::default();

        // Calculate cell metrics
        let metrics = Metrics::new(config.font_size, config.font_size * config.line_height);

        // Estimate cell size (will be refined when text is actually rendered)
        let cell_width = config.font_size * 0.6; // Approximate for monospace
        let cell_height = metrics.line_height;

        let size = TerminalSize {
            cols: (width as f32 / cell_width) as u16,
            rows: (height as f32 / cell_height) as u16,
            cell_width: cell_width as u16,
            cell_height: cell_height as u16,
        };

        Ok(Self {
            device,
            queue,
            surface: None,
            surface_config,
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            viewport,
            buffers: Vec::new(),
            size,
            palette: AnsiPalette::new(),
            config,
            cell_width,
            cell_height,
            buffer_dirty: Vec::new(),
            force_full_redraw: true,
            last_stats: RenderStats::default(),
        })
    }

    /// Create a new GPU renderer with custom configuration
    pub async fn with_config(width: u32, height: u32, config: RendererConfig) -> Result<Self> {
        let mut renderer = Self::new(width, height).await?;
        renderer.set_config(config);
        Ok(renderer)
    }

    /// Update renderer configuration
    pub fn set_config(&mut self, config: RendererConfig) {
        self.config = config;
        // Recalculate cell metrics
        let metrics = Metrics::new(self.config.font_size, self.config.font_size * self.config.line_height);
        self.cell_width = self.config.font_size * 0.6;
        self.cell_height = metrics.line_height;
        // Recalculate terminal size
        self.size.cols = (self.surface_config.width as f32 / self.cell_width).max(1.0) as u16;
        self.size.rows = (self.surface_config.height as f32 / self.cell_height).max(1.0) as u16;
        self.size.cell_width = self.cell_width as u16;
        self.size.cell_height = self.cell_height as u16;
    }

    /// Get current configuration
    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);

        if let Some(ref surface) = self.surface {
            surface.configure(&self.device, &self.surface_config);
        }

        // Recalculate terminal size
        self.size.cols = (width as f32 / self.cell_width).max(1.0) as u16;
        self.size.rows = (height as f32 / self.cell_height).max(1.0) as u16;

        // Update viewport
        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.surface_config.width,
                height: self.surface_config.height,
            },
        );

        // Force full redraw after resize
        self.force_full_redraw = true;
    }

    /// Force a full redraw on next render
    pub fn invalidate(&mut self) {
        self.force_full_redraw = true;
    }

    /// Get last render statistics
    pub fn last_stats(&self) -> &RenderStats {
        &self.last_stats
    }

    /// Get terminal size in cells
    pub fn terminal_size(&self) -> TerminalSize {
        self.size
    }

    /// Set default colors
    pub fn set_default_colors(&mut self, fg: Rgb, bg: Rgb) {
        self.config.fg_color = fg;
        self.config.bg_color = bg;
    }

    /// Convert terminal color to RGB
    fn resolve_color(&self, color: Color) -> Rgb {
        match color {
            Color::Indexed(idx) => self.palette.get(idx),
            Color::Rgb(rgb) => rgb,
            Color::Foreground => self.config.fg_color,
            Color::Background => self.config.bg_color,
        }
    }

    /// Get cell attributes for glyphon rendering (for future use with cell-level rendering)
    #[allow(dead_code)]
    fn cell_attrs(&self, cell: &crate::cell::Cell) -> Attrs<'_> {
        let weight = if cell.flags.contains(CellFlags::BOLD) {
            Weight::BOLD
        } else if cell.flags.contains(CellFlags::DIM) {
            Weight::LIGHT
        } else {
            Weight::NORMAL
        };

        let style = if cell.flags.contains(CellFlags::ITALIC) {
            Style::Italic
        } else {
            Style::Normal
        };

        Attrs::new()
            .family(Family::Monospace)
            .weight(weight)
            .style(style)
    }

    /// Convert RGB to glyphon Color (utility for future use)
    #[allow(dead_code)]
    fn to_glyphon_color(&self, rgb: Rgb) -> GlyphonColor {
        GlyphonColor::rgb(rgb.r, rgb.g, rgb.b)
    }

    /// Render the terminal grid
    pub fn render(
        &mut self,
        grid: &Grid,
        selection: Option<&Selection>,
    ) -> Result<()> {
        let surface = match &self.surface {
            Some(s) => s,
            None => return Ok(()), // No surface to render to
        };

        // Get surface texture
        let output = surface
            .get_current_texture()
            .map_err(|e| TerminalError::Gpu(format!("Surface error: {}", e)))?;

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        // Update text buffers for each row
        self.update_text_buffers(grid, selection);

        // Prepare text areas
        let default_fg = self.config.fg_color;
        let text_areas: Vec<TextArea> = self
            .buffers
            .iter()
            .enumerate()
            .map(|(row, buffer)| TextArea {
                buffer,
                left: 0.0,
                top: row as f32 * self.cell_height,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.surface_config.width as i32,
                    bottom: self.surface_config.height as i32,
                },
                default_color: GlyphonColor::rgb(default_fg.r, default_fg.g, default_fg.b),
                custom_glyphs: &[],
            })
            .collect();

        // Prepare text renderer
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .map_err(|e| TerminalError::Gpu(format!("Text prepare error: {:?}", e)))?;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Render pass
        {
            let bg = self.config.bg_color;
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: bg.r as f64 / 255.0,
                            g: bg.g as f64 / 255.0,
                            b: bg.b as f64 / 255.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render text
            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut render_pass)
                .map_err(|e| TerminalError::Gpu(format!("Text render error: {:?}", e)))?;
        }

        // Submit
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Update text buffers from grid with damage tracking
    fn update_text_buffers(&mut self, grid: &Grid, _selection: Option<&Selection>) {
        let start_time = std::time::Instant::now();
        let (_cols, rows) = grid.dims();
        let metrics = Metrics::new(self.config.font_size, self.cell_height);

        // Ensure we have enough buffers and dirty tracking
        while self.buffers.len() < rows {
            let mut buffer = Buffer::new(&mut self.font_system, metrics);
            buffer.set_size(&mut self.font_system, Some(self.surface_config.width as f32), Some(self.cell_height));
            self.buffers.push(buffer);
            self.buffer_dirty.push(true); // New buffers need content
        }

        // Truncate if too many
        self.buffers.truncate(rows);
        self.buffer_dirty.truncate(rows);

        // Determine which rows need updating
        let full_redraw = self.force_full_redraw;
        let mut rows_updated = 0;

        // Collect row data for dirty rows only (or all if full redraw)
        let row_data: Vec<_> = grid.rows().iter().enumerate()
            .filter(|(row_idx, grid_row)| {
                // Update if: full redraw, grid row is dirty, or our buffer was dirty
                full_redraw
                    || grid_row.is_dirty()
                    || self.buffer_dirty.get(*row_idx).copied().unwrap_or(true)
            })
            .map(|(row_idx, grid_row)| {
            // Build spans with per-cell attributes and resolved colors
            let mut spans: Vec<(String, CellFlags, Rgb)> = Vec::new();
            let mut current_text = String::new();
            let mut current_flags = CellFlags::empty();
            let mut current_fg = Color::Foreground;

            for cell in grid_row.cells() {
                // Check if attributes changed
                if cell.flags != current_flags || cell.fg != current_fg {
                    if !current_text.is_empty() {
                        // Resolve color now while we have immutable access
                        let resolved_color = self.resolve_color(current_fg);
                        spans.push((current_text.clone(), current_flags, resolved_color));
                        current_text.clear();
                    }
                    current_flags = cell.flags;
                    current_fg = cell.fg;
                }
                current_text.push(cell.c);
            }

            // Push remaining text
            if !current_text.is_empty() {
                let resolved_color = self.resolve_color(current_fg);
                spans.push((current_text, current_flags, resolved_color));
            }

            (row_idx, spans)
        }).collect();

        // Now update buffers using collected data - colors already resolved
        for (row_idx, spans) in row_data {
            if let Some(buffer) = self.buffers.get_mut(row_idx) {
                if spans.is_empty() {
                    buffer.set_text(
                        &mut self.font_system,
                        "",
                        Attrs::new().family(Family::Monospace),
                        Shaping::Advanced,
                    );
                } else if spans.len() == 1 {
                    // Single span - use simple set_text
                    let (text, flags, color) = &spans[0];
                    let weight = if flags.contains(CellFlags::BOLD) {
                        Weight::BOLD
                    } else if flags.contains(CellFlags::DIM) {
                        Weight::LIGHT
                    } else {
                        Weight::NORMAL
                    };

                    let style = if flags.contains(CellFlags::ITALIC) {
                        Style::Italic
                    } else {
                        Style::Normal
                    };

                    let attrs = Attrs::new()
                        .family(Family::Monospace)
                        .weight(weight)
                        .style(style)
                        .color(GlyphonColor::rgb(color.r, color.g, color.b));

                    buffer.set_text(
                        &mut self.font_system,
                        text,
                        attrs,
                        Shaping::Advanced,
                    );
                } else {
                    // Multiple spans - build rich text (colors already resolved)
                    let rich_text: Vec<_> = spans.iter().map(|(text, flags, color)| {
                        let weight = if flags.contains(CellFlags::BOLD) {
                            Weight::BOLD
                        } else if flags.contains(CellFlags::DIM) {
                            Weight::LIGHT
                        } else {
                            Weight::NORMAL
                        };

                        let style = if flags.contains(CellFlags::ITALIC) {
                            Style::Italic
                        } else {
                            Style::Normal
                        };

                        let attrs = Attrs::new()
                            .family(Family::Monospace)
                            .weight(weight)
                            .style(style)
                            .color(GlyphonColor::rgb(color.r, color.g, color.b));

                        (text.as_str(), attrs)
                    }).collect();

                    buffer.set_rich_text(
                        &mut self.font_system,
                        rich_text,
                        Attrs::new().family(Family::Monospace),
                        Shaping::Advanced,
                    );
                }

                buffer.shape_until_scroll(&mut self.font_system, false);

                // Mark this buffer as clean and count the update
                if row_idx < self.buffer_dirty.len() {
                    self.buffer_dirty[row_idx] = false;
                }
                rows_updated += 1;
            }
        }

        // Clear full redraw flag after processing
        self.force_full_redraw = false;

        // Update stats
        let elapsed = start_time.elapsed();
        self.last_stats = RenderStats {
            frame_time_ms: elapsed.as_secs_f32() * 1000.0,
            glyph_count: 0, // Would need to count actual glyphs from buffers
            draw_calls: 1,
            rows_updated,
            total_rows: rows,
            full_redraw,
        };
    }

    /// Draw cursor at position
    #[allow(dead_code)]
    fn draw_cursor(&self, _cursor: &Cursor) {
        // Cursor drawing would go here
        // For now, handled by text attributes
    }

    /// Get the wgpu device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get the wgpu queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}

/// Render stats for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
    /// Number of glyphs rendered
    pub glyph_count: usize,
    /// Number of draw calls
    pub draw_calls: usize,
    /// Number of rows updated this frame (damage tracking)
    pub rows_updated: usize,
    /// Total rows in terminal
    pub total_rows: usize,
    /// Whether full redraw was performed
    pub full_redraw: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_resolution() {
        // Basic test - full renderer tests require GPU
        let palette = AnsiPalette::new();
        let red = palette.get(1);
        assert_eq!(red, Rgb::new(0xCD, 0x00, 0x00));
    }
}
