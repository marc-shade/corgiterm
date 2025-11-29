//! AI assistant panel

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow, TextView, TextBuffer};
use gtk4::glib;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::ai_manager;
use corgiterm_ai::{Message, Role};

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

            // Mark as processing
            *processing_for_input.borrow_mut() = true;

            // Show user message
            let mut end_iter = buffer_for_input.end_iter();
            buffer_for_input.insert(&mut end_iter, &format!("\n📝 You: {}\n", text));

            // Clear input
            entry.set_text("");

            // Get current mode
            let current_mode = *mode_for_input.borrow();

            // Check if AI is available
            if let Some(am) = ai_manager() {
                let providers = {
                    let ai_mgr = am.read();
                    ai_mgr.list_providers().iter().map(|s| s.to_string()).collect::<Vec<_>>()
                };

                if providers.is_empty() {
                    let mut end_iter = buffer_for_input.end_iter();
                    buffer_for_input.insert(&mut end_iter, "\n⚠️ No AI providers configured.\n\nTo enable AI:\n• Install `claude` CLI (OAuth login)\n• Install `gemini` CLI (OAuth login)\n• Or add API keys in Preferences → AI\n");
                    *processing_for_input.borrow_mut() = false;
                    return;
                }

                // Show thinking indicator
                let mut end_iter = buffer_for_input.end_iter();
                buffer_for_input.insert(&mut end_iter, "\n🤔 Thinking...\n");

                // Build messages for AI
                let system_prompt = match current_mode {
                    AiPanelMode::Chat => "You are a helpful AI assistant integrated into CorgiTerm, a terminal emulator. Be concise and helpful.".to_string(),
                    AiPanelMode::Command => "You are a shell command expert. Convert natural language requests into shell commands. Output ONLY the command, no explanation.".to_string(),
                    AiPanelMode::Explain => "You are a shell command expert. Explain what the given command does in simple, clear terms.".to_string(),
                };

                let messages = vec![
                    Message { role: Role::System, content: system_prompt },
                    Message { role: Role::User, content: text.clone() },
                ];

                // Clone what we need for the async block
                let buffer = buffer_for_input.clone();
                let processing = processing_for_input.clone();
                let ai_manager = am.clone();

                // Use crossbeam channel for thread communication
                let (sender, receiver) = crossbeam_channel::unbounded::<Result<(String, String, u64), (String, String)>>();

                // Spawn blocking work in a thread
                std::thread::spawn(move || {
                    // Create a tokio runtime for async work
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            let _ = sender.send(Err(("runtime".to_string(), e.to_string())));
                            return;
                        }
                    };

                    rt.block_on(async {
                        let ai_mgr = ai_manager.read();
                        if let Some(provider) = ai_mgr.default_provider() {
                            let provider_name = provider.name().to_string();
                            let result = provider.complete(&messages).await;
                            drop(ai_mgr);

                            match result {
                                Ok(response) => {
                                    let _ = sender.send(Ok((provider_name, response.content, response.latency_ms)));
                                }
                                Err(e) => {
                                    let _ = sender.send(Err((provider_name, e.to_string())));
                                }
                            }
                        } else {
                            let _ = sender.send(Err(("none".to_string(), "No provider".to_string())));
                        }
                    });
                });

                // Poll for response using glib timeout
                glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                    match receiver.try_recv() {
                        Ok(result) => {
                            // Clear "Thinking..." text
                            let start = buffer.start_iter();
                            let end = buffer.end_iter();
                            let full_text = buffer.text(&start, &end, false).to_string();
                            if let Some(pos) = full_text.rfind("🤔 Thinking...") {
                                let char_start = full_text[..pos].chars().count() as i32;
                                let char_end = char_start + "🤔 Thinking...\n".chars().count() as i32;
                                let mut start_iter = buffer.start_iter();
                                let mut end_iter = buffer.start_iter();
                                start_iter.set_offset(char_start);
                                end_iter.set_offset(char_end);
                                buffer.delete(&mut start_iter, &mut end_iter);
                            }

                            // Show response
                            match result {
                                Ok((provider_name, content, latency_ms)) => {
                                    let emoji = match current_mode {
                                        AiPanelMode::Chat => "🐕",
                                        AiPanelMode::Command => "💻",
                                        AiPanelMode::Explain => "📖",
                                    };
                                    let mut end_iter = buffer.end_iter();
                                    buffer.insert(&mut end_iter, &format!(
                                        "\n{} {} (via {}, {}ms):\n{}\n",
                                        emoji,
                                        match current_mode {
                                            AiPanelMode::Chat => "Assistant",
                                            AiPanelMode::Command => "Command",
                                            AiPanelMode::Explain => "Explanation",
                                        },
                                        provider_name,
                                        latency_ms,
                                        content
                                    ));
                                }
                                Err((provider_name, error)) => {
                                    let mut end_iter = buffer.end_iter();
                                    if provider_name == "none" {
                                        buffer.insert(&mut end_iter, "\n❌ No AI provider available.\n");
                                    } else {
                                        buffer.insert(&mut end_iter, &format!(
                                            "\n❌ Error from {}: {}\n",
                                            provider_name, error
                                        ));
                                    }
                                }
                            }

                            // Mark as done processing
                            *processing.borrow_mut() = false;

                            glib::ControlFlow::Break
                        }
                        Err(crossbeam_channel::TryRecvError::Empty) => {
                            // Keep polling
                            glib::ControlFlow::Continue
                        }
                        Err(crossbeam_channel::TryRecvError::Disconnected) => {
                            // Channel closed, something went wrong
                            let mut end_iter = buffer.end_iter();
                            buffer.insert(&mut end_iter, "\n❌ AI request failed.\n");
                            *processing.borrow_mut() = false;
                            glib::ControlFlow::Break
                        }
                    }
                });
            } else {
                let mut end_iter = buffer_for_input.end_iter();
                buffer_for_input.insert(&mut end_iter, "\n❌ AI system not initialized.\n");
                *processing_for_input.borrow_mut() = false;
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
