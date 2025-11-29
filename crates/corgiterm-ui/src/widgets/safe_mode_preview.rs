//! Safe Mode command preview widget

use corgiterm_core::{CommandPreview, RiskLevel};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Revealer, RevealerTransitionType};
use std::cell::RefCell;
use std::rc::Rc;

/// Callback type aliases for clarity
type ExecuteCallback = std::boxed::Box<dyn Fn(String)>;
type CancelCallback = std::boxed::Box<dyn Fn()>;

/// Safe Mode preview popup widget
pub struct SafeModePreviewWidget {
    revealer: Revealer,
    container: GtkBox,
    /// Current command being previewed
    current_command: Rc<RefCell<Option<String>>>,
    /// Callbacks for user actions
    on_execute: Rc<RefCell<Option<ExecuteCallback>>>,
    on_cancel: Rc<RefCell<Option<CancelCallback>>>,
}

impl SafeModePreviewWidget {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 12);
        container.add_css_class("safe-mode-preview");
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::SlideUp);
        revealer.set_transition_duration(200);
        revealer.set_child(Some(&container));
        revealer.set_reveal_child(false);

        Self {
            revealer,
            container,
            current_command: Rc::new(RefCell::new(None)),
            on_execute: Rc::new(RefCell::new(None)),
            on_cancel: Rc::new(RefCell::new(None)),
        }
    }

    /// Set callback for when user clicks Execute
    pub fn set_on_execute<F: Fn(String) + 'static>(&self, callback: F) {
        *self.on_execute.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Set callback for when user cancels
    pub fn set_on_cancel<F: Fn() + 'static>(&self, callback: F) {
        *self.on_cancel.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Check if preview is currently visible
    pub fn is_visible(&self) -> bool {
        self.revealer.reveals_child()
    }

    /// Hide the preview
    pub fn hide(&self) {
        self.revealer.set_reveal_child(false);
        *self.current_command.borrow_mut() = None;
    }

    /// Cancel the preview (triggers callback and hides)
    pub fn cancel(&self) {
        if let Some(ref cb) = *self.on_cancel.borrow() {
            cb();
        }
        self.hide();
    }

    /// Show a command preview
    pub fn show_preview(&self, preview: &CommandPreview) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        // Header
        let header = GtkBox::new(Orientation::Horizontal, 8);
        let icon = Label::new(Some("🐕"));
        let title = Label::new(Some("Safe Mode Preview"));
        title.add_css_class("title-3");
        header.append(&icon);
        header.append(&title);
        self.container.append(&header);

        // Command
        let cmd_box = GtkBox::new(Orientation::Vertical, 4);
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
            let exp_box = GtkBox::new(Orientation::Vertical, 4);
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
                let alt_row = GtkBox::new(Orientation::Horizontal, 8);
                let alt_cmd = Label::new(Some(&alt.command));
                alt_cmd.add_css_class("monospace");
                alt_row.append(&alt_cmd);
                let alt_desc = Label::new(Some(&format!("({})", alt.description)));
                alt_desc.add_css_class("dim-label");
                alt_row.append(&alt_desc);
                self.container.append(&alt_row);
            }
        }

        // Store current command for execute callback
        *self.current_command.borrow_mut() = Some(preview.command.clone());

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_margin_top(12);
        button_box.set_halign(gtk4::Align::End);

        let cancel = Button::with_label("Cancel (Esc)");
        cancel.add_css_class("pill");

        // Wire cancel button
        let on_cancel = self.on_cancel.clone();
        let revealer_for_cancel = self.revealer.clone();
        let current_cmd_for_cancel = self.current_command.clone();
        cancel.connect_clicked(move |_| {
            if let Some(ref cb) = *on_cancel.borrow() {
                cb();
            }
            revealer_for_cancel.set_reveal_child(false);
            *current_cmd_for_cancel.borrow_mut() = None;
        });
        button_box.append(&cancel);

        let execute = Button::with_label("Execute");
        execute.add_css_class("pill");
        execute.add_css_class(if preview.risk == RiskLevel::Danger {
            "destructive-action"
        } else {
            "suggested-action"
        });

        // Wire execute button
        let on_execute = self.on_execute.clone();
        let revealer_for_exec = self.revealer.clone();
        let current_cmd_for_exec = self.current_command.clone();
        execute.connect_clicked(move |_| {
            let cmd = current_cmd_for_exec.borrow().clone();
            if let Some(cmd) = cmd {
                if let Some(ref cb) = *on_execute.borrow() {
                    cb(cmd);
                }
            }
            revealer_for_exec.set_reveal_child(false);
            *current_cmd_for_exec.borrow_mut() = None;
        });
        button_box.append(&execute);

        self.container.append(&button_box);

        // Show the preview
        self.revealer.set_reveal_child(true);
    }

    fn risk_description(&self, risk: &RiskLevel) -> &'static str {
        match risk {
            RiskLevel::Safe => "This command is safe to run",
            RiskLevel::Caution => "This will make changes that may be reversible",
            RiskLevel::Danger => "This will permanently delete or modify data",
            RiskLevel::Unknown => "Unable to assess risk level",
        }
    }

    pub fn widget(&self) -> &Revealer {
        &self.revealer
    }

    /// Get the inner container (for adding to layouts)
    pub fn container(&self) -> &GtkBox {
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
