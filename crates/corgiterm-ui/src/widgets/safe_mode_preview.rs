//! Safe Mode command preview widget

use corgiterm_core::{CommandPreview, RiskLevel};
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use libadwaita::prelude::*;

/// Safe Mode preview popup widget
pub struct SafeModePreviewWidget {
    container: Box,
}

impl SafeModePreviewWidget {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 12);
        container.add_css_class("safe-mode-preview");
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        Self { container }
    }

    /// Show a command preview
    pub fn show_preview(&self, preview: &CommandPreview) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        // Header
        let header = Box::new(Orientation::Horizontal, 8);
        let icon = Label::new(Some("🐕"));
        let title = Label::new(Some("Safe Mode Preview"));
        title.add_css_class("title-3");
        header.append(&icon);
        header.append(&title);
        self.container.append(&header);

        // Command
        let cmd_box = Box::new(Orientation::Vertical, 4);
        let cmd_label = Label::new(Some("Command:"));
        cmd_label.set_xalign(0.0);
        cmd_label.add_css_class("dim-label");
        cmd_box.append(&cmd_label);

        let cmd_text = Label::new(Some(&preview.command));
        cmd_text.set_xalign(0.0);
        cmd_text.add_css_class("monospace");
        cmd_box.append(&cmd_text);
        self.container.append(&cmd_box);

        // Risk level
        let risk_label = Label::new(Some(&format!(
            "{} {} - {}",
            preview.risk.emoji(),
            preview.risk.label(),
            self.risk_description(&preview.risk)
        )));
        risk_label.set_xalign(0.0);
        risk_label.add_css_class(match preview.risk {
            RiskLevel::Safe => "success",
            RiskLevel::Caution => "warning",
            RiskLevel::Danger => "error",
            RiskLevel::Unknown => "dim-label",
        });
        self.container.append(&risk_label);

        // Explanation
        if !preview.explanation.is_empty() {
            let exp_box = Box::new(Orientation::Vertical, 4);
            let exp_header = Label::new(Some("What it does:"));
            exp_header.set_xalign(0.0);
            exp_header.add_css_class("dim-label");
            exp_box.append(&exp_header);

            for exp in &preview.explanation {
                let bullet = Label::new(Some(&format!("• {}", exp)));
                bullet.set_xalign(0.0);
                bullet.set_wrap(true);
                exp_box.append(&bullet);
            }
            self.container.append(&exp_box);
        }

        // Affected files
        if let Some(count) = preview.affected_count {
            let affected = Label::new(Some(&format!(
                "• Will affect {} file(s) {}",
                count,
                preview
                    .affected_size
                    .map(|s| format!("({})", humanize_bytes(s)))
                    .unwrap_or_default()
            )));
            affected.set_xalign(0.0);
            self.container.append(&affected);
        }

        // Undo hint
        if let Some(ref undo) = preview.undo_hint {
            let undo_label = Label::new(Some(&format!("To undo: {}", undo)));
            undo_label.set_xalign(0.0);
            undo_label.add_css_class("dim-label");
            self.container.append(&undo_label);
        }

        // Alternatives
        if !preview.alternatives.is_empty() {
            let alt_header = Label::new(Some("Safer alternatives:"));
            alt_header.set_xalign(0.0);
            alt_header.add_css_class("dim-label");
            alt_header.set_margin_top(8);
            self.container.append(&alt_header);

            for alt in &preview.alternatives {
                let alt_row = Box::new(Orientation::Horizontal, 8);
                let alt_cmd = Label::new(Some(&alt.command));
                alt_cmd.add_css_class("monospace");
                alt_row.append(&alt_cmd);
                let alt_desc = Label::new(Some(&format!("({})", alt.description)));
                alt_desc.add_css_class("dim-label");
                alt_row.append(&alt_desc);
                self.container.append(&alt_row);
            }
        }

        // Buttons
        let button_box = Box::new(Orientation::Horizontal, 8);
        button_box.set_margin_top(12);
        button_box.set_halign(gtk4::Align::End);

        let cancel = Button::with_label("Cancel (Esc)");
        cancel.add_css_class("pill");
        button_box.append(&cancel);

        let execute = Button::with_label("Execute");
        execute.add_css_class("pill");
        execute.add_css_class(if preview.risk == RiskLevel::Danger {
            "destructive-action"
        } else {
            "suggested-action"
        });
        button_box.append(&execute);

        self.container.append(&button_box);
    }

    fn risk_description(&self, risk: &RiskLevel) -> &'static str {
        match risk {
            RiskLevel::Safe => "This command is safe to run",
            RiskLevel::Caution => "This will make changes that may be reversible",
            RiskLevel::Danger => "This will permanently delete or modify data",
            RiskLevel::Unknown => "Unable to assess risk level",
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}

impl Default for SafeModePreviewWidget {
    fn default() -> Self {
        Self::new()
    }
}

fn humanize_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
