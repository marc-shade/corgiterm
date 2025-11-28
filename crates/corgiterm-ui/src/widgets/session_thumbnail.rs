//! Session thumbnail widget for sidebar

use gtk4::prelude::*;
use gtk4::{Box, DrawingArea, Label, Orientation};

/// Session thumbnail widget
pub struct SessionThumbnail {
    container: Box,
    drawing_area: DrawingArea,
    name_label: Label,
}

impl SessionThumbnail {
    pub fn new(name: &str) -> Self {
        let container = Box::new(Orientation::Vertical, 4);
        container.add_css_class("session-thumbnail");

        // Thumbnail preview
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(180, 80);
        drawing_area.add_css_class("thumbnail-preview");

        // Draw a miniature terminal preview
        drawing_area.set_draw_func(|_area, cr, width, height| {
            // Background
            cr.set_source_rgb(0.118, 0.106, 0.086);
            cr.paint().ok();

            // Mini text lines (simulated)
            cr.set_source_rgb(0.5, 0.45, 0.4);
            cr.set_font_size(6.0);

            let lines = [
                "$ npm run dev",
                "> Starting server...",
                "> Ready on port 3000",
                "$ _",
            ];

            for (i, line) in lines.iter().enumerate() {
                cr.move_to(4.0, 12.0 + (i as f64 * 10.0));
                cr.show_text(line).ok();
            }
        });

        container.append(&drawing_area);

        // Name label
        let name_label = Label::new(Some(name));
        name_label.set_xalign(0.0);
        name_label.add_css_class("caption");
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        container.append(&name_label);

        Self {
            container,
            drawing_area,
            name_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Update the name
    pub fn set_name(&self, name: &str) {
        self.name_label.set_text(name);
    }

    /// Update the thumbnail from raw image data
    pub fn update_thumbnail(&self, _data: &[u8]) {
        // TODO: Render actual terminal content as thumbnail
        self.drawing_area.queue_draw();
    }
}
