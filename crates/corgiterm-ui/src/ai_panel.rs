//! AI assistant panel with mode-specific interfaces
//!
//! Inspired by:
//! - Warp's # command generation (atomic command blocks)
//! - Cursor's Cmd+K/L/I separation (inline, chat, agent)
//! - Cline's Plan/Act modes (approval before execution)
//! - GitHub Copilot's /slash commands

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Frame, Label, Orientation, ScrolledWindow, Separator, Stack, StackSwitcher,
    TextBuffer, TextView,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::ai_manager;
use corgiterm_ai::{Message, Role};

/// Helper trait to set all margins at once (GTK4 removed set_margin_all)
trait MarginExt {
    fn set_margins(&self, margin: i32);
}

impl<T: WidgetExt> MarginExt for T {
    fn set_margins(&self, margin: i32) {
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
        self.set_margin_start(margin);
        self.set_margin_end(margin);
    }
}

/// AI panel modes with distinct UIs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPanelMode {
    /// Chat mode - conversational AI assistant
    Chat,
    /// Command mode - natural language to shell command
    Command,
    /// Explain mode - understand commands and errors
    Explain,
}

impl AiPanelMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Command => "command",
            Self::Explain => "explain",
        }
    }
}

/// Generated command result for Command mode
#[derive(Debug, Clone)]
pub struct GeneratedCommand {
    pub command: String,
    pub explanation: Option<String>,
    pub risk_level: String, // safe, caution, danger
    pub provider: String,
    pub latency_ms: u64,
}

/// AI assistant panel widget with mode-specific interfaces
#[allow(dead_code)]
pub struct AiPanel {
    container: Box,
    mode: Rc<RefCell<AiPanelMode>>,
    stack: Stack,

    // Chat mode widgets
    chat_buffer: TextBuffer,
    chat_input: Entry,

    // Command mode widgets
    command_input: Entry,
    command_result_box: Box,
    generated_command: Rc<RefCell<Option<GeneratedCommand>>>,

    // Explain mode widgets
    explain_input: Entry,
    explain_result: TextBuffer,

    // Shared state
    is_processing: Rc<RefCell<bool>>,

    // Callback for executing commands (set by parent window)
    execute_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>>,

    // Callback for switching modes (used to switch to Explain with command)
    mode_switch_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(AiPanelMode, &str)>>>>,
}

impl AiPanel {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("ai-panel");

        let mode = Rc::new(RefCell::new(AiPanelMode::Chat));
        let is_processing = Rc::new(RefCell::new(false));
        let generated_command = Rc::new(RefCell::new(None));
        let execute_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>> =
            Rc::new(RefCell::new(None));
        let mode_switch_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(AiPanelMode, &str)>>>> =
            Rc::new(RefCell::new(None));

        // Header
        let header = Box::new(Orientation::Horizontal, 8);
        header.set_margins(8);

        let title = Label::new(Some("AI Assistant"));
        title.add_css_class("title-4");
        header.append(&title);

        // Provider indicator
        let provider_label = Label::new(None);
        provider_label.add_css_class("dim-label");
        provider_label.set_hexpand(true);
        provider_label.set_halign(gtk4::Align::End);
        Self::update_provider_label(&provider_label);
        header.append(&provider_label);

        container.append(&header);

        // Stack switcher for modes (like Cursor's Cmd+K/L/I)
        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);

        let switcher = StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_margin_start(8);
        switcher.set_margin_end(8);
        switcher.set_margin_bottom(8);
        container.append(&switcher);

        // === CHAT MODE (like Cursor's Cmd+L) ===
        let (chat_page, chat_buffer, chat_input) = Self::build_chat_mode(is_processing.clone());
        stack.add_titled(&chat_page, Some("chat"), "Chat");

        // === EXPLAIN MODE (like Copilot's /explain) ===
        let (explain_page, explain_input, explain_result) =
            Self::build_explain_mode(is_processing.clone());
        stack.add_titled(&explain_page, Some("explain"), "Explain");

        // === COMMAND MODE (like Warp's # trigger) ===
        let (command_page, command_input, command_result_box) = Self::build_command_mode(
            is_processing.clone(),
            generated_command.clone(),
            execute_callback.clone(),
            mode_switch_callback.clone(),
        );
        stack.add_titled(&command_page, Some("command"), "Command");

        container.append(&stack);

        // Wire up stack changes to update mode
        let mode_for_stack = mode.clone();
        stack.connect_visible_child_name_notify(move |s| {
            if let Some(name) = s.visible_child_name() {
                let new_mode = match name.as_str() {
                    "chat" => AiPanelMode::Chat,
                    "explain" => AiPanelMode::Explain,
                    "command" => AiPanelMode::Command,
                    _ => AiPanelMode::Chat,
                };
                *mode_for_stack.borrow_mut() = new_mode;
            }
        });

        // Default to Chat mode
        stack.set_visible_child_name("chat");

        let panel = Self {
            container,
            mode,
            stack,
            chat_buffer,
            chat_input,
            command_input,
            command_result_box,
            generated_command,
            explain_input,
            explain_result,
            is_processing,
            execute_callback,
            mode_switch_callback,
        };

        // Wire up internal mode switching
        panel.setup_mode_switching();

        panel
    }

    fn update_provider_label(label: &Label) {
        if let Some(am) = ai_manager() {
            let ai_mgr = am.read();
            if let Some(provider) = ai_mgr.default_provider() {
                label.set_text(&format!("via {}", provider.name()));
            } else {
                label.set_text("No AI configured");
            }
        } else {
            label.set_text("AI not ready");
        }
    }

    /// Build Command mode UI (Warp-style)
    fn build_command_mode(
        is_processing: Rc<RefCell<bool>>,
        generated_command: Rc<RefCell<Option<GeneratedCommand>>>,
        execute_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>>,
        mode_switch_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(AiPanelMode, &str)>>>>,
    ) -> (Box, Entry, Box) {
        let page = Box::new(Orientation::Vertical, 0);

        // Instruction
        let hint = Label::new(Some("Describe what you want to do in natural language"));
        hint.add_css_class("dim-label");
        hint.set_margins(8);
        hint.set_xalign(0.0);
        page.append(&hint);

        // Input with # prefix indicator (like Warp)
        let input_box = Box::new(Orientation::Horizontal, 4);
        input_box.set_margin_start(8);
        input_box.set_margin_end(8);

        let prefix = Label::new(Some("#"));
        prefix.add_css_class("accent");
        prefix.add_css_class("monospace");
        input_box.append(&prefix);

        let input = Entry::new();
        input.set_placeholder_text(Some("list all large files, find running processes..."));
        input.set_hexpand(true);
        input.add_css_class("monospace");
        input_box.append(&input);

        page.append(&input_box);

        // Result area (generated command + actions)
        let result_box = Box::new(Orientation::Vertical, 8);
        result_box.set_margins(8);
        result_box.set_visible(false); // Hidden until we have a result

        page.append(&result_box);

        // Spacer
        let spacer = Box::new(Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        page.append(&spacer);

        // Help text
        let help = Label::new(Some("Press Enter to generate command"));
        help.add_css_class("dim-label");
        help.set_margins(8);
        page.append(&help);

        // Connect input activation
        let result_box_for_input = result_box.clone();
        let processing = is_processing.clone();
        let gen_cmd = generated_command.clone();
        let exec_cb_for_input = execute_callback.clone();

        input.connect_activate(move |entry| {
            let text = entry.text().to_string();
            if text.is_empty() || *processing.borrow() {
                return;
            }

            *processing.borrow_mut() = true;

            // Clear previous result
            while let Some(child) = result_box_for_input.first_child() {
                result_box_for_input.remove(&child);
            }

            // Show loading
            let loading = Label::new(Some("Generating command..."));
            loading.add_css_class("dim-label");
            result_box_for_input.append(&loading);
            result_box_for_input.set_visible(true);

            // Generate command
            Self::generate_command(
                &text,
                result_box_for_input.clone(),
                processing.clone(),
                gen_cmd.clone(),
                exec_cb_for_input.clone(),
                mode_switch_callback.clone(),
            );
        });

        (page, input, result_box)
    }

    fn generate_command(
        prompt: &str,
        result_box: Box,
        is_processing: Rc<RefCell<bool>>,
        generated_command: Rc<RefCell<Option<GeneratedCommand>>>,
        execute_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>>,
        mode_switch_callback: Rc<RefCell<Option<std::boxed::Box<dyn Fn(AiPanelMode, &str)>>>>,
    ) {
        if let Some(am) = ai_manager() {
            let system = "You are a shell command expert. Convert the user's request into a single shell command. \
                         Output ONLY the command, nothing else. No explanation, no markdown, just the raw command.";

            let messages = vec![
                Message {
                    role: Role::System,
                    content: system.to_string(),
                },
                Message {
                    role: Role::User,
                    content: prompt.to_string(),
                },
            ];

            let (sender, receiver) =
                crossbeam_channel::unbounded::<Result<(String, String, u64), String>>();

            let ai_manager = am.clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = sender.send(Err(e.to_string()));
                        return;
                    }
                };

                rt.block_on(async {
                    let ai_mgr = ai_manager.read();
                    if let Some(provider) = ai_mgr.default_provider() {
                        let name = provider.name().to_string();
                        match provider.complete(&messages).await {
                            Ok(resp) => {
                                let _ = sender.send(Ok((name, resp.content, resp.latency_ms)));
                            }
                            Err(e) => {
                                let _ = sender.send(Err(e.to_string()));
                            }
                        }
                    } else {
                        let _ = sender.send(Err("No AI provider available".to_string()));
                    }
                });
            });

            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                match receiver.try_recv() {
                    Ok(result) => {
                        // Clear loading
                        while let Some(child) = result_box.first_child() {
                            result_box.remove(&child);
                        }

                        match result {
                            Ok((provider, command, latency)) => {
                                let cmd = command.trim().to_string();

                                // Store generated command
                                *generated_command.borrow_mut() = Some(GeneratedCommand {
                                    command: cmd.clone(),
                                    explanation: None,
                                    risk_level: "unknown".to_string(),
                                    provider: provider.clone(),
                                    latency_ms: latency,
                                });

                                // Command display frame
                                let cmd_frame = Frame::new(None);
                                cmd_frame.add_css_class("card");

                                let cmd_box = Box::new(Orientation::Vertical, 4);
                                cmd_box.set_margins(8);

                                // Command label
                                let cmd_label = Label::new(Some(&cmd));
                                cmd_label.add_css_class("monospace");
                                cmd_label.set_selectable(true);
                                cmd_label.set_xalign(0.0);
                                cmd_label.set_wrap(true);
                                cmd_box.append(&cmd_label);

                                // Provider info
                                let info =
                                    Label::new(Some(&format!("via {} ({}ms)", provider, latency)));
                                info.add_css_class("dim-label");
                                info.add_css_class("caption");
                                info.set_xalign(0.0);
                                cmd_box.append(&info);

                                cmd_frame.set_child(Some(&cmd_box));
                                result_box.append(&cmd_frame);

                                // Action buttons (like Cline's approval flow)
                                let actions = Box::new(Orientation::Horizontal, 8);
                                actions.set_margin_top(8);
                                actions.set_halign(gtk4::Align::Center);

                                // Copy button
                                let copy_btn = Button::with_label("Copy");
                                copy_btn.add_css_class("pill");
                                let cmd_for_copy = cmd.clone();
                                copy_btn.connect_clicked(move |btn| {
                                    let display = btn.display();
                                    let clipboard = display.clipboard();
                                    clipboard.set_text(&cmd_for_copy);
                                });
                                actions.append(&copy_btn);

                                // Execute button (primary action)
                                let exec_btn = Button::with_label("Execute");
                                exec_btn.add_css_class("pill");
                                exec_btn.add_css_class("suggested-action");

                                // Wire execute button to callback
                                let cmd_for_exec = cmd.clone();
                                let exec_cb = execute_callback.clone();
                                exec_btn.connect_clicked(move |_| {
                                    let callback = exec_cb.borrow();
                                    if let Some(ref cb) = *callback {
                                        cb(&cmd_for_exec);
                                        tracing::info!("Executed command from AI panel: {}", cmd_for_exec);
                                    } else {
                                        tracing::warn!("Execute button clicked but no callback set. Command: {}", cmd_for_exec);
                                    }
                                });

                                actions.append(&exec_btn);

                                // Explain button - switches to explain mode with this command
                                let explain_btn = Button::with_label("Explain");
                                explain_btn.add_css_class("pill");
                                let cmd_for_explain = cmd.clone();
                                let mode_switch_cb = mode_switch_callback.clone();
                                explain_btn.connect_clicked(move |_| {
                                    let callback = mode_switch_cb.borrow();
                                    if let Some(ref cb) = *callback {
                                        cb(AiPanelMode::Explain, &cmd_for_explain);
                                        tracing::info!("Switched to Explain mode with command: {}", cmd_for_explain);
                                    } else {
                                        tracing::warn!("Explain button clicked but no mode switch callback set");
                                    }
                                });
                                actions.append(&explain_btn);

                                result_box.append(&actions);
                            }
                            Err(error) => {
                                let err_label = Label::new(Some(&format!("Error: {}", error)));
                                err_label.add_css_class("error");
                                err_label.set_wrap(true);
                                result_box.append(&err_label);
                            }
                        }

                        *is_processing.borrow_mut() = false;
                        glib::ControlFlow::Break
                    }
                    Err(crossbeam_channel::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        while let Some(child) = result_box.first_child() {
                            result_box.remove(&child);
                        }
                        let err = Label::new(Some("Request failed"));
                        err.add_css_class("error");
                        result_box.append(&err);
                        *is_processing.borrow_mut() = false;
                        glib::ControlFlow::Break
                    }
                }
            });
        }
    }

    /// Build Chat mode UI (Cursor-style)
    fn build_chat_mode(is_processing: Rc<RefCell<bool>>) -> (Box, TextBuffer, Entry) {
        let page = Box::new(Orientation::Vertical, 0);

        // Chat history
        let buffer = TextBuffer::new(None::<&gtk4::TextTagTable>);
        let view = TextView::with_buffer(&buffer);
        view.set_editable(false);
        view.set_wrap_mode(gtk4::WrapMode::Word);
        view.set_margins(8);
        view.add_css_class("view");

        // Welcome message
        let mut iter = buffer.end_iter();
        buffer.insert(&mut iter, "Welcome! Ask me anything about terminal commands, scripting, or get help with errors.\n\n");

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&view));
        page.append(&scrolled);

        page.append(&Separator::new(Orientation::Horizontal));

        // Input area
        let input_box = Box::new(Orientation::Horizontal, 8);
        input_box.set_margins(8);

        let input = Entry::new();
        input.set_placeholder_text(Some("Ask a question..."));
        input.set_hexpand(true);
        input_box.append(&input);

        let send_btn = Button::with_label("Send");
        send_btn.add_css_class("suggested-action");
        input_box.append(&send_btn);

        page.append(&input_box);

        // Connect input
        let buffer_for_input = buffer.clone();
        let processing = is_processing.clone();
        let input_for_send = input.clone();

        let on_send = move || {
            let text = input_for_send.text().to_string();
            if text.is_empty() || *processing.borrow() {
                return;
            }

            *processing.borrow_mut() = true;

            // Show user message
            let mut iter = buffer_for_input.end_iter();
            buffer_for_input.insert(&mut iter, &format!("You: {}\n\n", text));

            input_for_send.set_text("");

            // Get AI response
            Self::chat_response(&text, buffer_for_input.clone(), processing.clone());
        };

        let on_send_for_enter = on_send.clone();
        input.connect_activate(move |_| {
            on_send_for_enter();
        });

        send_btn.connect_clicked(move |_| {
            on_send();
        });

        (page, buffer, input)
    }

    fn chat_response(prompt: &str, buffer: TextBuffer, is_processing: Rc<RefCell<bool>>) {
        if let Some(am) = ai_manager() {
            let system = "You are a helpful terminal assistant integrated into CorgiTerm. \
                         Help users with shell commands, scripting, and terminal usage. \
                         Be concise but thorough. Use code blocks for commands.";

            let messages = vec![
                Message {
                    role: Role::System,
                    content: system.to_string(),
                },
                Message {
                    role: Role::User,
                    content: prompt.to_string(),
                },
            ];

            let (sender, receiver) = crossbeam_channel::unbounded::<Result<String, String>>();

            let ai_manager = am.clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = sender.send(Err(e.to_string()));
                        return;
                    }
                };

                rt.block_on(async {
                    let ai_mgr = ai_manager.read();
                    if let Some(provider) = ai_mgr.default_provider() {
                        match provider.complete(&messages).await {
                            Ok(resp) => {
                                let _ = sender.send(Ok(resp.content));
                            }
                            Err(e) => {
                                let _ = sender.send(Err(e.to_string()));
                            }
                        }
                    } else {
                        let _ = sender.send(Err("No AI provider".to_string()));
                    }
                });
            });

            glib::timeout_add_local(std::time::Duration::from_millis(50), move || match receiver
                .try_recv()
            {
                Ok(result) => {
                    let mut iter = buffer.end_iter();
                    match result {
                        Ok(content) => {
                            buffer.insert(&mut iter, &format!("Assistant: {}\n\n", content));
                        }
                        Err(error) => {
                            buffer.insert(&mut iter, &format!("Error: {}\n\n", error));
                        }
                    }
                    *is_processing.borrow_mut() = false;
                    glib::ControlFlow::Break
                }
                Err(crossbeam_channel::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    let mut iter = buffer.end_iter();
                    buffer.insert(&mut iter, "Error: Request failed\n\n");
                    *is_processing.borrow_mut() = false;
                    glib::ControlFlow::Break
                }
            });
        }
    }

    /// Build Explain mode UI (Copilot /explain style)
    fn build_explain_mode(is_processing: Rc<RefCell<bool>>) -> (Box, Entry, TextBuffer) {
        let page = Box::new(Orientation::Vertical, 0);

        // Instructions
        let hint = Label::new(Some("Paste a command or error message to understand it"));
        hint.add_css_class("dim-label");
        hint.set_margins(8);
        hint.set_xalign(0.0);
        page.append(&hint);

        // Input (multi-line for errors)
        let input = Entry::new();
        input.set_placeholder_text(Some("ls -la | grep '.txt' | wc -l"));
        input.set_margin_start(8);
        input.set_margin_end(8);
        input.add_css_class("monospace");
        page.append(&input);

        // Explain button
        let btn_box = Box::new(Orientation::Horizontal, 0);
        btn_box.set_margins(8);
        let explain_btn = Button::with_label("Explain");
        explain_btn.add_css_class("suggested-action");
        btn_box.append(&explain_btn);
        page.append(&btn_box);

        page.append(&Separator::new(Orientation::Horizontal));

        // Result area
        let result_buffer = TextBuffer::new(None::<&gtk4::TextTagTable>);
        let result_view = TextView::with_buffer(&result_buffer);
        result_view.set_editable(false);
        result_view.set_wrap_mode(gtk4::WrapMode::Word);
        result_view.set_margins(8);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&result_view));
        page.append(&scrolled);

        // Connect explain button
        let input_for_btn = input.clone();
        let result_for_btn = result_buffer.clone();
        let processing = is_processing.clone();

        let on_explain = move || {
            let text = input_for_btn.text().to_string();
            if text.is_empty() || *processing.borrow() {
                return;
            }

            *processing.borrow_mut() = true;

            // Clear and show loading
            let start = result_for_btn.start_iter();
            let end = result_for_btn.end_iter();
            result_for_btn.delete(&mut start.clone(), &mut end.clone());

            let mut iter = result_for_btn.end_iter();
            result_for_btn.insert(&mut iter, "Analyzing...\n");

            Self::explain_command(&text, result_for_btn.clone(), processing.clone());
        };

        let on_explain_for_enter = on_explain.clone();
        input.connect_activate(move |_| {
            on_explain_for_enter();
        });

        explain_btn.connect_clicked(move |_| {
            on_explain();
        });

        (page, input, result_buffer)
    }

    fn explain_command(command: &str, buffer: TextBuffer, is_processing: Rc<RefCell<bool>>) {
        if let Some(am) = ai_manager() {
            let system = "You are a shell command expert. Explain the given command in detail:\n\
                         1. What does this command do overall?\n\
                         2. Break down each part/flag\n\
                         3. What are potential risks or side effects?\n\
                         4. Suggest any safer alternatives if applicable\n\
                         Be thorough but clear for beginners.";

            let messages = vec![
                Message {
                    role: Role::System,
                    content: system.to_string(),
                },
                Message {
                    role: Role::User,
                    content: format!("Explain this command: {}", command),
                },
            ];

            let (sender, receiver) = crossbeam_channel::unbounded::<Result<String, String>>();

            let ai_manager = am.clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = sender.send(Err(e.to_string()));
                        return;
                    }
                };

                rt.block_on(async {
                    let ai_mgr = ai_manager.read();
                    if let Some(provider) = ai_mgr.default_provider() {
                        match provider.complete(&messages).await {
                            Ok(resp) => {
                                let _ = sender.send(Ok(resp.content));
                            }
                            Err(e) => {
                                let _ = sender.send(Err(e.to_string()));
                            }
                        }
                    } else {
                        let _ = sender.send(Err("No AI provider".to_string()));
                    }
                });
            });

            glib::timeout_add_local(std::time::Duration::from_millis(50), move || match receiver
                .try_recv()
            {
                Ok(result) => {
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    buffer.delete(&mut start.clone(), &mut end.clone());

                    let mut iter = buffer.end_iter();
                    match result {
                        Ok(explanation) => {
                            buffer.insert(&mut iter, &explanation);
                        }
                        Err(error) => {
                            buffer.insert(&mut iter, &format!("Error: {}", error));
                        }
                    }
                    *is_processing.borrow_mut() = false;
                    glib::ControlFlow::Break
                }
                Err(crossbeam_channel::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    buffer.delete(&mut start.clone(), &mut end.clone());
                    let mut iter = buffer.end_iter();
                    buffer.insert(&mut iter, "Request failed");
                    *is_processing.borrow_mut() = false;
                    glib::ControlFlow::Break
                }
            });
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the current mode
    pub fn set_mode(&self, new_mode: AiPanelMode) {
        *self.mode.borrow_mut() = new_mode;
        self.stack.set_visible_child_name(new_mode.as_str());
    }

    /// Get current mode
    pub fn current_mode(&self) -> AiPanelMode {
        *self.mode.borrow()
    }

    /// Add a message to the chat
    pub fn add_message(&self, role: &str, content: &str) {
        let mut end = self.chat_buffer.end_iter();
        self.chat_buffer
            .insert(&mut end, &format!("{}: {}\n\n", role, content));
    }

    /// Clear the chat history
    pub fn clear_chat(&self) {
        let start = self.chat_buffer.start_iter();
        let end = self.chat_buffer.end_iter();
        self.chat_buffer
            .delete(&mut start.clone(), &mut end.clone());

        let mut iter = self.chat_buffer.start_iter();
        self.chat_buffer
            .insert(&mut iter, "Chat cleared. How can I help?\n\n");
    }

    /// Focus the input field for current mode
    pub fn focus_input(&self) {
        match *self.mode.borrow() {
            AiPanelMode::Command => {
                self.command_input.grab_focus();
            }
            AiPanelMode::Chat => {
                self.chat_input.grab_focus();
            }
            AiPanelMode::Explain => {
                self.explain_input.grab_focus();
            }
        }
    }

    /// Set command input (for natural language bar integration)
    pub fn set_command_query(&self, query: &str) {
        self.set_mode(AiPanelMode::Command);
        self.command_input.set_text(query);
        self.command_input.grab_focus();
    }

    /// Set explain input (for terminal selection)
    pub fn set_explain_text(&self, text: &str) {
        self.set_mode(AiPanelMode::Explain);
        self.explain_input.set_text(text);
        self.explain_input.grab_focus();
    }

    /// Get the last generated command (for execution)
    pub fn last_generated_command(&self) -> Option<String> {
        self.generated_command
            .borrow()
            .as_ref()
            .map(|c| c.command.clone())
    }

    /// Set the callback for executing commands
    /// This should be called by the parent window to wire up command execution
    pub fn set_execute_callback<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        *self.execute_callback.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Set the callback for mode switching
    /// Used internally to wire up Explain button in Command mode
    fn set_mode_switch_callback<F>(&self, callback: F)
    where
        F: Fn(AiPanelMode, &str) + 'static,
    {
        *self.mode_switch_callback.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Wire up internal mode switching (called during construction)
    fn setup_mode_switching(&self) {
        let stack = self.stack.clone();
        let explain_input = self.explain_input.clone();

        self.set_mode_switch_callback(move |mode, text| {
            // Switch to the requested mode
            stack.set_visible_child_name(mode.as_str());

            // If switching to Explain, populate the input
            if mode == AiPanelMode::Explain {
                explain_input.set_text(text);
                explain_input.grab_focus();
                // Trigger explanation automatically
                explain_input.emit_activate();
            }
        });
    }
}

impl Default for AiPanel {
    fn default() -> Self {
        Self::new()
    }
}
