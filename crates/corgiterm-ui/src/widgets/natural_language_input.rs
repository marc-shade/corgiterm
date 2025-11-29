//! Natural language command input widget

use gtk4::prelude::*;
use gtk4::{Box, Entry, Label, Orientation};

/// Natural language input widget
pub struct NaturalLanguageInput {
    container: Box,
    entry: Entry,
    suggestion_label: Label,
}

impl NaturalLanguageInput {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 4);
        container.add_css_class("natural-language-input");

        // Input entry
        let entry = Entry::new();
        entry.set_placeholder_text(Some("🐕 Type naturally: \"show files bigger than 1GB\""));
        entry.add_css_class("natural-input");
        container.append(&entry);

        // Suggestion label (shows translated command)
        let suggestion_label = Label::new(None);
        suggestion_label.set_xalign(0.0);
        suggestion_label.add_css_class("monospace");
        suggestion_label.add_css_class("dim-label");
        suggestion_label.set_visible(false);
        container.append(&suggestion_label);

        Self {
            container,
            entry,
            suggestion_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Show a command suggestion
    pub fn show_suggestion(&self, command: &str) {
        self.suggestion_label.set_text(&format!("→ {}", command));
        self.suggestion_label.set_visible(true);
    }

    /// Hide the suggestion
    pub fn hide_suggestion(&self) {
        self.suggestion_label.set_visible(false);
    }

    /// Get current input text
    pub fn text(&self) -> String {
        self.entry.text().to_string()
    }

    /// Clear the input
    pub fn clear(&self) {
        self.entry.set_text("");
        self.hide_suggestion();
    }

    /// Connect to text changed signal
    pub fn connect_changed<F: Fn(&str) + 'static>(&self, f: F) {
        self.entry.connect_changed(move |entry| {
            f(&entry.text());
        });
    }

    /// Connect to activate signal (Enter pressed)
    pub fn connect_activate<F: Fn() + 'static>(&self, f: F) {
        self.entry.connect_activate(move |_| {
            f();
        });
    }
}

impl Default for NaturalLanguageInput {
    fn default() -> Self {
        Self::new()
    }
}
