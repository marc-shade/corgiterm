//! Session thumbnail widget for sidebar

use gtk4::prelude::*;
use gtk4::{Box, DrawingArea, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Session thumbnail widget
pub struct SessionThumbnail {
    container: Box,
    drawing_area: DrawingArea,
    name_label: Label,
    /// Cached lines from terminal for rendering
    lines: Rc<RefCell<Vec<String>>>,
    /// Working directory path for this session
    path: Rc<RefCell<String>>,
}

impl SessionThumbnail {
    pub fn new(name: &str) -> Self {
        let container = Box::new(Orientation::Vertical, 4);
        container.add_css_class("session-thumbnail");

        let lines: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        // Thumbnail preview
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(180, 80);
        drawing_area.add_css_class("thumbnail-preview");

        // Draw terminal preview
        let lines_for_draw = lines.clone();
        let path_for_draw = path.clone();
        drawing_area.set_draw_func(move |_area, cr, width, _height| {
            let lines = lines_for_draw.borrow();
            let path = path_for_draw.borrow();

            // Background (corgi dark theme)
            cr.set_source_rgb(0.118, 0.106, 0.086);
            cr.paint().ok();

            // Draw border
            cr.set_source_rgb(0.25, 0.23, 0.20);
            cr.set_line_width(1.0);
            cr.rectangle(0.5, 0.5, width as f64 - 1.0, 79.0);
            cr.stroke().ok();

            cr.set_font_size(6.0);

            if lines.is_empty() {
                // Show placeholder with path
                cr.set_source_rgb(0.5, 0.45, 0.4);
                let display_text = if path.is_empty() {
                    "No terminal output"
                } else {
                    &path
                };
                cr.move_to(4.0, 40.0);
                cr.show_text(display_text).ok();
            } else {
                // Draw actual terminal content
                let max_display_lines = 8;
                let start = if lines.len() > max_display_lines {
                    lines.len() - max_display_lines
                } else {
                    0
                };

                for (i, line) in lines.iter().skip(start).take(max_display_lines).enumerate() {
                    // Color code based on content
                    if line.starts_with('$') || line.starts_with('#') || line.starts_with('>') {
                        cr.set_source_rgb(0.573, 0.706, 0.447); // Green for prompts
                    } else if line.contains("error") || line.contains("Error") || line.contains("ERROR") {
                        cr.set_source_rgb(0.800, 0.341, 0.322); // Red for errors
                    } else if line.contains("warning") || line.contains("Warning") || line.contains("WARN") {
                        cr.set_source_rgb(0.898, 0.659, 0.294); // Yellow for warnings
                    } else {
                        cr.set_source_rgb(0.7, 0.65, 0.58); // Light for normal text
                    }

                    // Truncate line if too long
                    let display_line = if line.len() > 35 {
                        format!("{}...", &line[..32])
                    } else {
                        line.clone()
                    };

                    cr.move_to(4.0, 10.0 + (i as f64 * 9.0));
                    cr.show_text(&display_line).ok();
                }
            }
        });

        container.append(&drawing_area);

        // Name label
        let name_label = Label::new(Some(name));
        name_label.set_xalign(0.0);
        name_label.add_css_class("session-thumbnail-label");
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        container.append(&name_label);

        Self {
            container,
            drawing_area,
            name_label,
            lines,
            path,
        }
    }

    /// Create a thumbnail for a specific path (e.g., from sidebar project)
    pub fn for_path(name: &str, path: &str) -> Self {
        let thumb = Self::new(name);
        *thumb.path.borrow_mut() = path.to_string();
        thumb.drawing_area.queue_draw();
        thumb
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Update the name
    pub fn set_name(&self, name: &str) {
        self.name_label.set_text(name);
    }

    /// Update the thumbnail with terminal lines
    pub fn update_lines(&self, lines: Vec<String>) {
        *self.lines.borrow_mut() = lines;
        self.drawing_area.queue_draw();
    }

    /// Update the path displayed when no content is available
    pub fn set_path(&self, path: &str) {
        *self.path.borrow_mut() = path.to_string();
        self.drawing_area.queue_draw();
    }

    /// Queue a redraw
    pub fn queue_draw(&self) {
        self.drawing_area.queue_draw();
    }
}
