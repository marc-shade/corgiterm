//! AI assistant panel

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow, TextView, TextBuffer};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::ai_manager;

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
    mode: Rc<RefCell<AiPanelMode>>,
    #[allow(dead_code)]
    chat_view: TextView,
    chat_buffer: TextBuffer,
    input: Entry,
    #[allow(dead_code)]
    is_processing: Rc<RefCell<bool>>,
}

impl AiPanel {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("ai-panel");

        // Shared state
        let mode = Rc::new(RefCell::new(AiPanelMode::Chat));
        let is_processing = Rc::new(RefCell::new(false));

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

        // Wire up mode button clicks
        let mode_for_chat = mode.clone();
        let chat_btn_ref = chat_btn.clone();
        let cmd_btn_ref = cmd_btn.clone();
        let explain_btn_ref = explain_btn.clone();
        chat_btn.connect_clicked(move |btn| {
            *mode_for_chat.borrow_mut() = AiPanelMode::Chat;
            btn.add_css_class("suggested-action");
            cmd_btn_ref.remove_css_class("suggested-action");
            explain_btn_ref.remove_css_class("suggested-action");
        });

        let mode_for_cmd = mode.clone();
        let chat_btn_ref2 = chat_btn_ref.clone();
        let explain_btn_ref2 = explain_btn.clone();
        cmd_btn.connect_clicked(move |btn| {
            *mode_for_cmd.borrow_mut() = AiPanelMode::Command;
            btn.add_css_class("suggested-action");
            chat_btn_ref2.remove_css_class("suggested-action");
            explain_btn_ref2.remove_css_class("suggested-action");
        });

        let mode_for_explain = mode.clone();
        let chat_btn_ref3 = chat_btn_ref.clone();
        let cmd_btn_ref3 = cmd_btn.clone();
        explain_btn.connect_clicked(move |btn| {
            *mode_for_explain.borrow_mut() = AiPanelMode::Explain;
            btn.add_css_class("suggested-action");
            chat_btn_ref3.remove_css_class("suggested-action");
            cmd_btn_ref3.remove_css_class("suggested-action");
        });

        // Chat view with buffer
        let chat_buffer = TextBuffer::new(None::<&gtk4::TextTagTable>);
        let chat_view = TextView::with_buffer(&chat_buffer);
        chat_view.set_editable(false);
        chat_view.set_wrap_mode(gtk4::WrapMode::Word);
        chat_view.set_margin_start(8);
        chat_view.set_margin_end(8);
        chat_view.set_margin_top(8);
        chat_view.set_margin_bottom(8);

        // Add welcome message
        let mut end_iter = chat_buffer.end_iter();
        chat_buffer.insert(&mut end_iter, "🐕 Welcome to CorgiTerm AI Assistant!\n\nI can help you with:\n• Chat - Ask me anything\n• Command - Describe what you want to do\n• Explain - Paste a command to understand it\n\n");

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

        // Connect input activation (Enter key)
        let buffer_for_input = chat_buffer.clone();
        let mode_for_input = mode.clone();
        let processing_for_input = is_processing.clone();
        input.connect_activate(move |entry| {
            let text = entry.text().to_string();
            if text.is_empty() || *processing_for_input.borrow() {
                return;
            }

            // Show user message
            let mut end_iter = buffer_for_input.end_iter();
            buffer_for_input.insert(&mut end_iter, &format!("\n📝 You: {}\n", text));

            // Clear input
            entry.set_text("");

            // Get current mode
            let current_mode = *mode_for_input.borrow();

            // Check if AI is available
            if let Some(am) = ai_manager() {
                let ai_mgr = am.read();
                let providers = ai_mgr.list_providers();

                if providers.is_empty() {
                    let mut end_iter = buffer_for_input.end_iter();
                    buffer_for_input.insert(&mut end_iter, "\n⚠️ No AI providers configured. Add API keys in Preferences → AI.\n");
                    return;
                }

                // Show thinking indicator
                let mut end_iter = buffer_for_input.end_iter();
                buffer_for_input.insert(&mut end_iter, "\n🤔 Thinking...\n");

                // For now, show a placeholder - async AI calls will be added in a future iteration
                let response = match current_mode {
                    AiPanelMode::Chat => format!("🐕 [AI response pending - {} providers available: {}]\n\nTo enable AI, configure your API keys in Preferences → AI.",
                        providers.len(), providers.join(", ")),
                    AiPanelMode::Command => format!("💻 [Command translation pending]\n\nProviders: {}", providers.join(", ")),
                    AiPanelMode::Explain => format!("📖 [Explanation pending]\n\nProviders: {}", providers.join(", ")),
                };

                // Clear "Thinking..." and show response
                let start = buffer_for_input.start_iter();
                let end = buffer_for_input.end_iter();
                let full_text = buffer_for_input.text(&start, &end, false).to_string();
                if let Some(pos) = full_text.rfind("🤔 Thinking...") {
                    let byte_start = pos;
                    let byte_end = pos + "🤔 Thinking...\n".len();
                    let mut start_iter = buffer_for_input.start_iter();
                    let mut end_iter = buffer_for_input.start_iter();
                    start_iter.set_offset(full_text[..byte_start].chars().count() as i32);
                    end_iter.set_offset(full_text[..byte_end].chars().count() as i32);
                    buffer_for_input.delete(&mut start_iter, &mut end_iter);
                }

                let mut end_iter = buffer_for_input.end_iter();
                buffer_for_input.insert(&mut end_iter, &format!("\n{}\n", response));
            } else {
                let mut end_iter = buffer_for_input.end_iter();
                buffer_for_input.insert(&mut end_iter, "\n❌ AI system not initialized.\n");
            }
        });

        // Provider selector
        let provider_box = Box::new(Orientation::Horizontal, 4);
        provider_box.set_margin_start(8);
        provider_box.set_margin_end(8);
        provider_box.set_margin_bottom(8);

        let provider_label = Label::new(Some("Provider:"));
        provider_box.append(&provider_label);

        // Get available providers dynamically (need owned strings)
        let provider_strings: Vec<String> = if let Some(am) = ai_manager() {
            let ai_mgr = am.read();
            let providers = ai_mgr.list_providers();
            if providers.is_empty() {
                vec!["No providers configured".to_string()]
            } else {
                providers.iter().map(|s| s.to_string()).collect()
            }
        } else {
            vec!["Not initialized".to_string()]
        };

        // Create dropdown
        let provider_strs: Vec<&str> = provider_strings.iter().map(|s| s.as_str()).collect();
        let dropdown = gtk4::DropDown::from_strings(&provider_strs);
        provider_box.append(&dropdown);

        container.append(&provider_box);

        Self {
            container,
            mode,
            chat_view,
            chat_buffer,
            input,
            is_processing,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the current mode
    pub fn set_mode(&self, new_mode: AiPanelMode) {
        *self.mode.borrow_mut() = new_mode;
        // Update placeholder based on mode
        let placeholder = match new_mode {
            AiPanelMode::Chat => "Ask anything...",
            AiPanelMode::Command => "Describe what you want to do...",
            AiPanelMode::Explain => "Paste command to explain...",
        };
        self.input.set_placeholder_text(Some(placeholder));
    }

    /// Get current mode
    pub fn current_mode(&self) -> AiPanelMode {
        *self.mode.borrow()
    }

    /// Add a message to the chat
    pub fn add_message(&self, role: &str, content: &str) {
        let mut end = self.chat_buffer.end_iter();
        self.chat_buffer.insert(&mut end, &format!("\n{}: {}\n", role, content));
    }

    /// Clear the chat history
    pub fn clear_chat(&self) {
        let start = self.chat_buffer.start_iter();
        let end = self.chat_buffer.end_iter();
        self.chat_buffer.delete(&mut start.clone(), &mut end.clone());

        let mut iter = self.chat_buffer.start_iter();
        self.chat_buffer.insert(&mut iter, "🐕 Chat cleared. How can I help?\n");
    }

    /// Focus the input field
    pub fn focus_input(&self) {
        self.input.grab_focus();
    }
}

impl Default for AiPanel {
    fn default() -> Self {
        Self::new()
    }
}
