//! AI assistant panel

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow, TextView};

/// AI panel modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPanelMode {
    Chat,
    Command,
    Explain,
}

/// AI assistant panel widget
pub struct AiPanel {
    container: Box,
    mode: AiPanelMode,
    chat_view: TextView,
    input: Entry,
}

impl AiPanel {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("ai-panel");

        // Header with mode selector
        let header = Box::new(Orientation::Horizontal, 6);
        header.set_margin_start(8);
        header.set_margin_end(8);
        header.set_margin_top(8);
        header.set_margin_bottom(8);

        let title = Label::new(Some("🐕 AI Assistant"));
        title.add_css_class("title-4");
        header.append(&title);

        container.append(&header);

        // Mode buttons
        let mode_box = Box::new(Orientation::Horizontal, 4);
        mode_box.set_margin_start(8);
        mode_box.set_margin_end(8);

        let chat_btn = Button::with_label("Chat");
        chat_btn.add_css_class("pill");
        chat_btn.add_css_class("suggested-action");
        mode_box.append(&chat_btn);

        let cmd_btn = Button::with_label("Command");
        cmd_btn.add_css_class("pill");
        mode_box.append(&cmd_btn);

        let explain_btn = Button::with_label("Explain");
        explain_btn.add_css_class("pill");
        mode_box.append(&explain_btn);

        container.append(&mode_box);

        // Chat view
        let chat_view = TextView::new();
        chat_view.set_editable(false);
        chat_view.set_wrap_mode(gtk4::WrapMode::Word);
        chat_view.set_margin_start(8);
        chat_view.set_margin_end(8);
        chat_view.set_margin_top(8);
        chat_view.set_margin_bottom(8);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&chat_view));
        container.append(&scrolled);

        // Input
        let input = Entry::new();
        input.set_placeholder_text(Some("Ask anything or describe a command..."));
        input.set_margin_start(8);
        input.set_margin_end(8);
        input.set_margin_top(8);
        input.set_margin_bottom(8);
        container.append(&input);

        // Provider selector
        let provider_box = Box::new(Orientation::Horizontal, 4);
        provider_box.set_margin_start(8);
        provider_box.set_margin_end(8);
        provider_box.set_margin_bottom(8);

        let provider_label = Label::new(Some("Provider:"));
        provider_box.append(&provider_label);

        let dropdown = gtk4::DropDown::from_strings(&["Claude", "OpenAI", "Gemini", "Local"]);
        provider_box.append(&dropdown);

        container.append(&provider_box);

        Self {
            container,
            mode: AiPanelMode::Chat,
            chat_view,
            input,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the current mode
    pub fn set_mode(&mut self, mode: AiPanelMode) {
        self.mode = mode;
        // Update placeholder based on mode
        let placeholder = match mode {
            AiPanelMode::Chat => "Ask anything...",
            AiPanelMode::Command => "Describe what you want to do...",
            AiPanelMode::Explain => "Paste command to explain...",
        };
        self.input.set_placeholder_text(Some(placeholder));
    }

    /// Add a message to the chat
    pub fn add_message(&self, role: &str, content: &str) {
        let buffer = self.chat_view.buffer();
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, &format!("\n{}: {}\n", role, content));
    }
}

impl Default for AiPanel {
    fn default() -> Self {
        Self::new()
    }
}
