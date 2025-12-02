//! Main application window

use gtk4::gio::{self, Menu, SimpleAction};
use gtk4::prelude::*;
use gtk4::{
    Application, Box, Button, EventControllerKey, FileDialog, FileFilter, Label, MenuButton,
    Orientation, Paned, Revealer, RevealerTransitionType, Spinner, Widget,
};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, HeaderBar, WindowTitle};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ai_panel::AiPanel;
use crate::dialogs;
use crate::keyboard::{KeyboardShortcuts, ShortcutAction};
use crate::recording_panel::show_recording_dialog;
use crate::sidebar::Sidebar;
use crate::tab_bar::TerminalTabs;
use crate::widgets::natural_language_input::NaturalLanguageInput;
use crate::widgets::safe_mode_preview::SafeModePreviewWidget;
use corgiterm_core::SafeMode;
use std::path::PathBuf;

/// Main application window
pub struct MainWindow {
    window: ApplicationWindow,
    #[allow(dead_code)]
    tabs: Rc<TerminalTabs>,
    #[allow(dead_code)]
    sidebar: Rc<Sidebar>,
    #[allow(dead_code)]
    sidebar_widget: Widget,
    #[allow(dead_code)]
    ai_panel: Rc<RefCell<AiPanel>>,
    #[allow(dead_code)]
    ai_revealer: Revealer,
    #[allow(dead_code)]
    nl_input: Rc<NaturalLanguageInput>,
    #[allow(dead_code)]
    safe_mode_preview: Rc<SafeModePreviewWidget>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        // Create window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("CorgiTerm")
            .default_width(1200)
            .default_height(800)
            .build();

        // Explicitly set size - needed for some X11 window managers
        window.set_size_request(800, 600);
        window.set_default_size(1200, 800);

        // Force proper sizing on realize (fixes X11/libadwaita issues)
        window.connect_realize(|w| {
            // Queue a resize after realization to ensure proper geometry
            w.queue_resize();
            tracing::debug!("Window realized, queued resize");
        });

        // Create components
        let sidebar = Rc::new(Sidebar::new());
        let tabs = Rc::new(TerminalTabs::new());

        // Create header bar
        let header = HeaderBar::new();

        // Window title
        let title = WindowTitle::new("CorgiTerm", "~/");
        header.set_title_widget(Some(&title));

        // Sidebar toggle button
        let sidebar_toggle_btn = Button::from_icon_name("sidebar-show-symbolic");
        sidebar_toggle_btn.set_tooltip_text(Some("Toggle Sidebar (Ctrl+Shift+B)"));
        header.pack_start(&sidebar_toggle_btn);

        // New tab button
        let new_tab_btn = Button::from_icon_name("tab-new-symbolic");
        new_tab_btn.set_tooltip_text(Some("New Tab (Ctrl+T)"));
        let tabs_for_btn = tabs.clone();
        new_tab_btn.connect_clicked(move |_| {
            tabs_for_btn.add_terminal_tab("Terminal", None);
        });
        header.pack_start(&new_tab_btn);

        // Create menu model
        let menu = Menu::new();

        // Tools submenu
        let tools_menu = Menu::new();
        tools_menu.append(Some("_SSH Manager"), Some("win.ssh_manager"));
        tools_menu.append(Some("_ASCII Art Generator"), Some("win.ascii_art"));
        tools_menu.append(Some("_Emojis"), Some("win.emojis"));
        tools_menu.append(Some("_History Search"), Some("win.history_search"));
        tools_menu.append(Some("_Session Recording"), Some("win.session_recording"));
        menu.append_submenu(Some("_Tools"), &tools_menu);

        menu.append(Some("_Preferences"), Some("win.preferences"));
        menu.append(Some("_Keyboard Shortcuts"), Some("win.shortcuts"));
        menu.append(Some("_About CorgiTerm"), Some("win.about"));

        // Menu button with popover
        let menu_btn = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .menu_model(&menu)
            .build();
        header.pack_end(&menu_btn);

        // AI panel toggle button
        let ai_toggle_btn = Button::from_icon_name("user-available-symbolic");
        ai_toggle_btn.set_tooltip_text(Some("Toggle AI Assistant (Ctrl+Shift+A)"));
        header.pack_end(&ai_toggle_btn);

        // Add window actions
        let prefs_action = SimpleAction::new("preferences", None);
        let win_for_prefs = window.clone();
        let tabs_for_prefs = tabs.clone();
        prefs_action.connect_activate(move |_, _| {
            let tabs_clone = tabs_for_prefs.clone();
            dialogs::show_preferences(
                &win_for_prefs,
                Some(std::boxed::Box::new(move || {
                    tabs_clone.queue_redraw_all_terminals();
                })),
            );
        });
        window.add_action(&prefs_action);

        let about_action = SimpleAction::new("about", None);
        let win_for_about = window.clone();
        about_action.connect_activate(move |_, _| {
            dialogs::show_about_dialog(&win_for_about);
        });
        window.add_action(&about_action);

        let shortcuts_action = SimpleAction::new("shortcuts", None);
        let win_for_shortcuts = window.clone();
        shortcuts_action.connect_activate(move |_, _| {
            dialogs::show_shortcuts_dialog(&win_for_shortcuts);
        });
        window.add_action(&shortcuts_action);

        let ascii_art_action = SimpleAction::new("ascii_art", None);
        let win_for_ascii = window.clone();
        let tabs_for_ascii = tabs.clone();
        ascii_art_action.connect_activate(move |_, _| {
            let tabs = tabs_for_ascii.clone();
            dialogs::show_ascii_art_dialog(&win_for_ascii, move |art| {
                if tabs.send_command_to_current(art) {
                    tracing::info!("Inserted ASCII art into terminal ({} bytes)", art.len());
                } else {
                    tracing::warn!("No active terminal for ASCII art insertion");
                }
            });
        });
        window.add_action(&ascii_art_action);

        // SSH Manager action
        let ssh_manager_action = SimpleAction::new("ssh_manager", None);
        let win_for_ssh = window.clone();
        ssh_manager_action.connect_activate(move |_, _| {
            let ssh_manager = crate::ssh_manager::SshManager::new(&win_for_ssh);
            ssh_manager.show(&win_for_ssh);
        });
        window.add_action(&ssh_manager_action);

        // Emojis action
        let emojis_action = SimpleAction::new("emojis", None);
        let win_for_emojis = window.clone();
        let tabs_for_emojis = tabs.clone();
        emojis_action.connect_activate(move |_, _| {
            let tabs = tabs_for_emojis.clone();
            crate::emoji_picker::show_emoji_picker(&win_for_emojis, move |emoji| {
                if tabs.send_text_to_current(&emoji) {
                    tracing::info!("Inserted emoji into terminal: {}", emoji);
                } else {
                    tracing::warn!("No active terminal for emoji insertion");
                }
            });
        });
        window.add_action(&emojis_action);

        // History Search action
        let history_action = SimpleAction::new("history_search", None);
        let win_for_history = window.clone();
        let tabs_for_history = tabs.clone();
        history_action.connect_activate(move |_, _| {
            let tabs = tabs_for_history.clone();
            crate::history_search::show_history_search_dialog(&win_for_history, move |cmd| {
                if tabs.send_command_to_current(cmd) {
                    tracing::info!("History search: inserted command {}", cmd);
                } else {
                    tracing::warn!("No active terminal for history search");
                }
            });
        });
        window.add_action(&history_action);

        // Session Recording action
        let recording_action = SimpleAction::new("session_recording", None);
        let win_for_recording = window.clone();
        recording_action.connect_activate(move |_, _| {
            show_recording_dialog(&win_for_recording);
        });
        window.add_action(&recording_action);

        // Main layout with header + content
        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);

        // Header bar with tab bar integrated
        let header_box = Box::new(Orientation::Vertical, 0);
        header_box.append(&header);
        header_box.append(tabs.tab_bar_widget());
        main_box.append(&header_box);

        // Create natural language input widget
        let nl_input = Rc::new(NaturalLanguageInput::new());

        // NL input area with status indicator
        let nl_container = Box::new(Orientation::Horizontal, 8);
        nl_container.add_css_class("nl-input-container");
        nl_container.set_margin_start(8);
        nl_container.set_margin_end(8);
        nl_container.set_margin_top(4);
        nl_container.set_margin_bottom(4);

        // Spinner for loading state
        let nl_spinner = Spinner::new();
        nl_spinner.set_visible(false);
        nl_container.append(&nl_spinner);

        // Input widget
        nl_container.append(nl_input.widget());
        nl_input.widget().set_hexpand(true);

        // Status label for errors
        let nl_status = Label::new(None);
        nl_status.add_css_class("dim-label");
        nl_status.set_visible(false);
        nl_container.append(&nl_status);

        // Create Safe Mode preview widget and analyzer
        let safe_mode_preview = Rc::new(SafeModePreviewWidget::new());
        let safe_mode = Rc::new(RefCell::new(SafeMode::new()));
        safe_mode.borrow_mut().set_enabled(true); // Enable by default for safety

        // Terminal area (NL input removed - using AI panel on the right instead)
        let terminal_area = Box::new(Orientation::Vertical, 0);
        terminal_area.set_hexpand(true); // Expand horizontally to fill available space
        terminal_area.append(tabs.tab_view_widget());
        terminal_area.append(safe_mode_preview.widget()); // Safe mode preview at bottom
                                                          // nl_container removed - AI interaction now handled by AI panel
        tabs.tab_view_widget().set_vexpand(true);
        tabs.tab_view_widget().set_hexpand(true); // Ensure tabs expand horizontally

        // Wire safe mode execute callback
        let tabs_for_safe_exec = tabs.clone();
        safe_mode_preview.set_on_execute(move |command| {
            if tabs_for_safe_exec.send_command_to_current(&command) {
                tracing::info!("Safe mode: Executed command: {}", command);
            }
        });

        // Wire safe mode cancel callback (no-op, just for logging)
        safe_mode_preview.set_on_cancel(|| {
            tracing::info!("Safe mode: Command cancelled");
        });

        // Sidebar setup with Paned layout (original working approach)
        let sidebar_widget = sidebar.widget().clone();
        sidebar_widget.set_width_request(220);

        // Content paned: sidebar | terminal area
        let content_paned = Paned::new(Orientation::Horizontal);
        content_paned.set_start_child(Some(&sidebar_widget));
        content_paned.set_end_child(Some(&terminal_area));
        content_paned.set_resize_start_child(false);
        content_paned.set_shrink_start_child(false);
        content_paned.set_resize_end_child(true);
        content_paned.set_shrink_end_child(false);
        content_paned.set_position(220);
        content_paned.set_vexpand(true);

        // Sidebar visibility state (tracked separately to avoid is_visible() issues)
        let sidebar_visible = Rc::new(RefCell::new(true));

        // Connect sidebar toggle button - only adjust paned position (avoids visibility toggle issues)
        let sidebar_visible_for_btn = sidebar_visible.clone();
        let paned_for_btn = content_paned.clone();
        sidebar_toggle_btn.connect_clicked(move |_| {
            let is_visible = *sidebar_visible_for_btn.borrow();
            *sidebar_visible_for_btn.borrow_mut() = !is_visible;
            // Just change paned position - don't toggle visibility
            if is_visible {
                paned_for_btn.set_position(0);
            } else {
                paned_for_btn.set_position(220);
            }
        });

        // Connect NL input to AI translation and terminal execution
        let tabs_for_nl = tabs.clone();
        let nl_input_for_activate = nl_input.clone();
        let nl_spinner_ref = nl_spinner.clone();
        let nl_status_ref = nl_status.clone();

        // Store current translation for use in activate handler
        let current_translation: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let translation_for_change = current_translation.clone();
        let translation_for_activate = current_translation.clone();

        // On text change: try pattern matching first, then AI fallback
        let nl_spinner_for_change = nl_spinner.clone();
        nl_input.connect_changed(move |text| {
            if text.is_empty() {
                nl_input_for_activate.hide_suggestion();
                *translation_for_change.borrow_mut() = None;
                nl_spinner_for_change.set_spinning(false);
                nl_spinner_for_change.set_visible(false);
                return;
            }

            // Try quick pattern-based translation first
            let suggestion = quick_translate(text);
            if let Some(cmd) = &suggestion {
                nl_input_for_activate.show_suggestion(cmd);
                *translation_for_change.borrow_mut() = suggestion;
                nl_spinner_for_change.set_spinning(false);
                nl_spinner_for_change.set_visible(false);
                return;
            }

            // No pattern match - will use AI on activation
            // Show hint that AI will be used
            nl_input_for_activate.show_suggestion("(Press Enter to translate with AI...)");
            *translation_for_change.borrow_mut() = None;
        });

        // On Enter: execute the translated command (with safe mode check)
        // If no pattern matched, use AI to translate first
        let nl_input_for_exec = nl_input.clone();
        let safe_mode_for_activate = safe_mode.clone();
        let safe_mode_preview_for_activate = safe_mode_preview.clone();
        nl_input.connect_activate(move || {
            let translation = translation_for_activate.borrow().clone();
            let user_text = nl_input_for_exec.text();

            if user_text.is_empty() {
                return;
            }

            if let Some(cmd) = translation {
                // We have a pattern-matched command - execute it
                execute_command(&cmd, &user_text, &tabs_for_nl, &safe_mode_for_activate,
                    &safe_mode_preview_for_activate, &nl_input_for_exec, &nl_status_ref);
            } else {
                // No pattern match - use AI to translate
                // Show spinner
                nl_spinner_ref.set_spinning(true);
                nl_spinner_ref.set_visible(true);

                // Check if AI is available
                if let Some(am) = crate::app::ai_manager() {
                    let providers = {
                        let ai_mgr = am.read();
                        ai_mgr.list_providers().iter().map(|s| s.to_string()).collect::<Vec<_>>()
                    };

                    if providers.is_empty() {
                        nl_status_ref.set_text("No AI providers configured");
                        nl_status_ref.set_visible(true);
                        nl_spinner_ref.set_spinning(false);
                        nl_spinner_ref.set_visible(false);
                        gtk4::glib::timeout_add_local_once(std::time::Duration::from_secs(3), {
                            let status = nl_status_ref.clone();
                            move || status.set_visible(false)
                        });
                        return;
                    }

                    // Build translation prompt
                    let system_prompt = format!(
                        "You are a shell command expert. Convert this natural language request into a shell command.\n\
                        Current shell: {}\n\
                        OS: {}\n\n\
                        Rules:\n\
                        1. Output ONLY the command, nothing else\n\
                        2. Use safe, standard commands\n\
                        3. No explanations or markdown",
                        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
                        std::env::consts::OS
                    );

                    let messages = vec![
                        corgiterm_ai::Message { role: corgiterm_ai::Role::System, content: system_prompt },
                        corgiterm_ai::Message { role: corgiterm_ai::Role::User, content: user_text.clone() },
                    ];

                    // Clone refs for async handler
                    let tabs = tabs_for_nl.clone();
                    let safe_mode = safe_mode_for_activate.clone();
                    let safe_mode_preview = safe_mode_preview_for_activate.clone();
                    let nl_input = nl_input_for_exec.clone();
                    let nl_status = nl_status_ref.clone();
                    let spinner = nl_spinner_ref.clone();
                    let user_text_for_log = user_text.clone();

                    // Use crossbeam channel for async AI call
                    let (sender, receiver) = crossbeam_channel::unbounded::<Result<String, String>>();

                    std::thread::spawn(move || {
                        let rt = match tokio::runtime::Runtime::new() {
                            Ok(rt) => rt,
                            Err(e) => {
                                let _ = sender.send(Err(e.to_string()));
                                return;
                            }
                        };

                        rt.block_on(async {
                            let ai_mgr = am.read();
                            if let Some(provider) = ai_mgr.default_provider() {
                                match provider.complete(&messages).await {
                                    Ok(response) => {
                                        // Clean up the response - remove any markdown or extra text
                                        let cmd = response.content.trim()
                                            .trim_start_matches("```")
                                            .trim_start_matches("bash")
                                            .trim_start_matches("sh")
                                            .trim_end_matches("```")
                                            .trim()
                                            .to_string();
                                        let _ = sender.send(Ok(cmd));
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

                    // Poll for AI response
                    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                        match receiver.try_recv() {
                            Ok(Ok(cmd)) => {
                                spinner.set_spinning(false);
                                spinner.set_visible(false);

                                // Execute the AI-generated command
                                execute_command(&cmd, &user_text_for_log, &tabs, &safe_mode,
                                    &safe_mode_preview, &nl_input, &nl_status);

                                gtk4::glib::ControlFlow::Break
                            }
                            Ok(Err(error)) => {
                                spinner.set_spinning(false);
                                spinner.set_visible(false);
                                nl_status.set_text(&format!("AI error: {}", error));
                                nl_status.set_visible(true);
                                gtk4::glib::timeout_add_local_once(std::time::Duration::from_secs(3), {
                                    let status = nl_status.clone();
                                    move || status.set_visible(false)
                                });
                                gtk4::glib::ControlFlow::Break
                            }
                            Err(crossbeam_channel::TryRecvError::Empty) => {
                                gtk4::glib::ControlFlow::Continue
                            }
                            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                                spinner.set_spinning(false);
                                spinner.set_visible(false);
                                nl_status.set_text("AI request failed");
                                nl_status.set_visible(true);
                                gtk4::glib::ControlFlow::Break
                            }
                        }
                    });
                } else {
                    nl_status_ref.set_text("AI not initialized");
                    nl_status_ref.set_visible(true);
                    nl_spinner_ref.set_spinning(false);
                    nl_spinner_ref.set_visible(false);
                }
            }
        });

        // Create AI panel with slide-out revealer
        let ai_panel = Rc::new(RefCell::new(AiPanel::new()));

        // Wire AI panel execute button to send commands to the active terminal
        let tabs_for_ai = tabs.clone();
        ai_panel.borrow().set_execute_callback(move |command| {
            if tabs_for_ai.send_command_to_current(command) {
                tracing::info!("AI panel executed command: {}", command);
            } else {
                tracing::warn!(
                    "AI panel: No active terminal to execute command: {}",
                    command
                );
            }
        });

        // Set up AI panel revealer - slides in from the right
        // Terminal will resize when panel opens/closes (handled by debounced resize)
        let ai_revealer = Revealer::new();
        ai_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
        ai_revealer.set_transition_duration(150);
        ai_revealer.set_reveal_child(false);
        ai_revealer.set_child(Some(ai_panel.borrow().widget()));
        ai_panel.borrow().widget().set_width_request(350);

        // Connect AI toggle button
        let revealer_for_toggle = ai_revealer.clone();
        ai_toggle_btn.connect_clicked(move |_| {
            let currently_revealed = revealer_for_toggle.reveals_child();
            revealer_for_toggle.set_reveal_child(!currently_revealed);
        });

        // Connect to child-revealed to detect when animation completes
        // This triggers a resize after the Revealer animation finishes
        let tabs_for_reveal = tabs.clone();
        ai_revealer.connect_notify_local(Some("child-revealed"), move |revealer, _| {
            // Animation has completed when child-revealed matches reveals-child
            let revealed = revealer.is_child_revealed();
            let target = revealer.reveals_child();
            if revealed == target {
                // Animation complete - force full resize of terminal tabs
                // queue_resize() forces GTK to recalculate all widget sizes
                tabs_for_reveal.tab_view_widget().queue_resize();
            }
        });

        // Use horizontal Box so terminal resizes when AI panel opens/closes
        let content_with_ai = Box::new(Orientation::Horizontal, 0);
        content_with_ai.set_hexpand(true);
        content_with_ai.set_vexpand(true);
        content_paned.set_hexpand(true);
        content_paned.set_vexpand(true);
        content_with_ai.append(&content_paned); // Main content (sidebar + terminal)
        content_with_ai.append(&ai_revealer); // AI panel on the right

        // Revealer should NOT expand - let the paned take all available space
        ai_revealer.set_hexpand(false);

        main_box.append(&content_with_ai);

        window.set_content(Some(&main_box));

        // Force size after content is set (critical for X11/libadwaita)
        window.set_default_size(1200, 800);

        // Also force size in idle callback after main loop starts
        let window_for_idle = window.clone();
        gtk4::glib::idle_add_local_once(move || {
            window_for_idle.set_default_size(1200, 800);
            // Try direct surface sizing if available
            if window_for_idle.surface().is_some() {
                // Request specific size from compositor
                tracing::debug!("Requesting surface size 1200x800");
            }
            window_for_idle.queue_resize();
        });

        // Connect sidebar project folder clicks to tab creation
        let tabs_for_session = tabs.clone();
        sidebar.set_on_session_click(move |name, path| {
            // Open terminal in the selected folder
            tabs_for_session.add_terminal_tab(name, Some(path));
            tracing::info!("Opened terminal in: {}", path);
        });

        // Connect sidebar file shortcut clicks to document tab creation
        let tabs_for_file = tabs.clone();
        sidebar.set_on_file_click(move |name, path| {
            // Open file in document editor tab
            let file_path = std::path::PathBuf::from(path);
            tabs_for_file.add_document_tab(name, Some(&file_path));
            tracing::info!("Opened file: {}", path);
        });

        // Load keyboard shortcuts from configuration
        let shortcuts = if let Some(cm) = crate::app::config_manager() {
            let config = cm.read().config();
            KeyboardShortcuts::from_config(&config.keybindings.shortcuts)
        } else {
            KeyboardShortcuts::default()
        };
        let shortcuts = Rc::new(shortcuts);

        // Set up keyboard shortcuts
        let key_controller = EventControllerKey::new();
        let tabs_for_keys = tabs.clone();
        let window_for_keys = window.clone();
        let ai_revealer_for_keys = ai_revealer.clone();
        let sidebar_visible_for_keys = sidebar_visible.clone();
        let paned_for_keys = content_paned.clone();
        let safe_mode_preview_for_keys = safe_mode_preview.clone();
        let shortcuts_for_keys = shortcuts.clone();
        key_controller.connect_key_pressed(move |_, key, _keycode, modifier| {
            use gtk4::gdk::Key;

            // Handle Escape to close safe mode preview
            if key == Key::Escape && safe_mode_preview_for_keys.is_visible() {
                safe_mode_preview_for_keys.cancel();
                return gtk4::glib::Propagation::Stop;
            }

            // Check configured shortcuts

            // Tab management
            if shortcuts_for_keys.matches(ShortcutAction::NewTab, key, modifier) {
                tabs_for_keys.add_terminal_tab("Terminal", None);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::CloseTab, key, modifier) {
                tabs_for_keys.close_current_tab();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::NextTab, key, modifier) {
                tabs_for_keys.select_next_tab();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::PrevTab, key, modifier) {
                tabs_for_keys.select_previous_tab();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::NewDocumentTab, key, modifier) {
                tabs_for_keys.add_document_tab("Document", None);
                return gtk4::glib::Propagation::Stop;
            }

            // Tab switching
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab1, key, modifier) {
                tabs_for_keys.select_tab_by_index(0);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab2, key, modifier) {
                tabs_for_keys.select_tab_by_index(1);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab3, key, modifier) {
                tabs_for_keys.select_tab_by_index(2);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab4, key, modifier) {
                tabs_for_keys.select_tab_by_index(3);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab5, key, modifier) {
                tabs_for_keys.select_tab_by_index(4);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab6, key, modifier) {
                tabs_for_keys.select_tab_by_index(5);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab7, key, modifier) {
                tabs_for_keys.select_tab_by_index(6);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab8, key, modifier) {
                tabs_for_keys.select_tab_by_index(7);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SwitchToTab9, key, modifier) {
                tabs_for_keys.select_tab_by_index(8);
                return gtk4::glib::Propagation::Stop;
            }

            // Pane management
            if shortcuts_for_keys.matches(ShortcutAction::SplitHorizontal, key, modifier) {
                tabs_for_keys.split_current_horizontal();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SplitVertical, key, modifier) {
                tabs_for_keys.split_current_vertical();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::ClosePane, key, modifier) {
                tabs_for_keys.close_focused_pane();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::FocusNextPane, key, modifier) {
                tabs_for_keys.focus_next_pane();
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::FocusPrevPane, key, modifier) {
                tabs_for_keys.focus_prev_pane();
                return gtk4::glib::Propagation::Stop;
            }

            // UI features
            if shortcuts_for_keys.matches(ShortcutAction::ToggleAi, key, modifier) {
                let currently_revealed = ai_revealer_for_keys.reveals_child();
                ai_revealer_for_keys.set_reveal_child(!currently_revealed);
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::ToggleSidebar, key, modifier) {
                let is_visible = *sidebar_visible_for_keys.borrow();
                *sidebar_visible_for_keys.borrow_mut() = !is_visible;
                // Just change paned position - don't toggle visibility
                if is_visible {
                    paned_for_keys.set_position(0);
                } else {
                    paned_for_keys.set_position(220);
                }
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::QuickSwitcher, key, modifier) {
                dialogs::show_quick_switcher(&window_for_keys, tabs_for_keys.tab_view_widget());
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::SshManager, key, modifier) {
                gtk4::prelude::ActionGroupExt::activate_action(
                    &window_for_keys,
                    "ssh_manager",
                    None,
                );
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::Snippets, key, modifier) {
                let win = window_for_keys.clone();
                let tabs = tabs_for_keys.clone();
                crate::snippets::show_snippets_dialog(&win, move |snippet| {
                    tabs.send_command_to_current(&snippet);
                });
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::AsciiArt, key, modifier) {
                let win = window_for_keys.clone();
                let tabs = tabs_for_keys.clone();
                dialogs::show_ascii_art_dialog(&win, move |art| {
                    tabs.send_command_to_current(&format!("echo '{}'\n", art));
                });
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::HistorySearch, key, modifier) {
                let win = window_for_keys.clone();
                let tabs = tabs_for_keys.clone();
                crate::history_search::show_history_search_dialog(&win, move |cmd| {
                    if tabs.send_command_to_current(cmd) {
                        tracing::info!("History search: executed {}", cmd);
                    }
                });
                return gtk4::glib::Propagation::Stop;
            }
            if shortcuts_for_keys.matches(ShortcutAction::OpenFile, key, modifier) {
                let tabs = tabs_for_keys.clone();
                let win = window_for_keys.clone();

                // Create file filter for text files
                let filter = FileFilter::new();
                filter.set_name(Some("Text Files"));
                filter.add_mime_type("text/*");
                filter.add_suffix("txt");
                filter.add_suffix("md");
                filter.add_suffix("rs");
                filter.add_suffix("py");
                filter.add_suffix("js");
                filter.add_suffix("ts");
                filter.add_suffix("json");
                filter.add_suffix("toml");
                filter.add_suffix("yaml");
                filter.add_suffix("yml");

                let filters = gio::ListStore::new::<FileFilter>();
                filters.append(&filter);

                let dialog = FileDialog::builder()
                    .title("Open File")
                    .modal(true)
                    .filters(&filters)
                    .build();

                dialog.open(Some(&win), None::<&gio::Cancellable>, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let filename = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Document");
                            let page = tabs.add_document_tab(filename, Some(&path));
                            tracing::info!("Opened file: {:?}", path);
                            let _ = page; // Use the page if needed
                        }
                    }
                });
                return gtk4::glib::Propagation::Stop;
            }

            // Application
            if shortcuts_for_keys.matches(ShortcutAction::Quit, key, modifier) {
                window_for_keys.close();
                return gtk4::glib::Propagation::Stop;
            }

            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        // Set up periodic timer to update tab titles based on working directory
        // Poll every 500ms to check if the current directory has changed
        let tabs_for_title_update = tabs.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            tabs_for_title_update.update_tab_titles();
            gtk4::glib::ControlFlow::Continue
        });

        Self {
            window,
            tabs,
            sidebar,
            sidebar_widget: sidebar_widget.into(),
            ai_panel,
            ai_revealer,
            nl_input,
            safe_mode_preview,
        }
    }

    pub fn present(&self) {
        // On X11/libadwaita, show() helps ensure proper realization before present()
        self.window.show();
        self.window.present();

        // WORKAROUND: On X11/Budgie, GTK4/libadwaita sometimes fails to set initial window size/position
        // Use xdotool as a fallback to force window to be visible and properly sized.
        // Spawn thread directly (GTK timeout callbacks don't reliably fire on some X11 setups)
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        if session_type != "wayland" {
            std::thread::spawn(move || {
                // Wait for window to be fully mapped by X11/WM
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Get OUR process PID to find only our windows
                // NOTE: Only use --pid filter (NOT --name) to avoid stale window cache issues
                let our_pid = std::process::id();

                if let Ok(output) = std::process::Command::new("xdotool")
                    .args(["search", "--pid", &our_pid.to_string()])
                    .output()
                {
                    let window_ids: Vec<String> = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    eprintln!(
                        "X11 workaround: found {} windows for pid {}",
                        window_ids.len(),
                        our_pid
                    );

                    // First pass: find the main window (larger than 1x1) and any small placeholder windows
                    let mut main_window: Option<String> = None;
                    let mut small_windows: Vec<String> = Vec::new();

                    for wid in &window_ids {
                        if let Ok(geo_output) = std::process::Command::new("xdotool")
                            .args(["getwindowgeometry", wid])
                            .output()
                        {
                            if geo_output.status.success() {
                                let geo_str = String::from_utf8_lossy(&geo_output.stdout);
                                // Parse geometry to check if it's a tiny placeholder window
                                // Format: "Window XXXXX\n  Position: X,Y\n  Geometry: WxH"
                                let is_small = geo_str.lines().any(|line| {
                                    if line.trim().starts_with("Geometry:") {
                                        let size = line.trim().strip_prefix("Geometry:").unwrap_or("").trim();
                                        // Consider windows <= 10x10 as placeholders
                                        if let Some((w, h)) = size.split_once('x') {
                                            if let (Ok(width), Ok(height)) = (w.parse::<u32>(), h.parse::<u32>()) {
                                                return width <= 10 || height <= 10;
                                            }
                                        }
                                    }
                                    false
                                });

                                if is_small {
                                    small_windows.push(wid.clone());
                                } else if main_window.is_none() {
                                    main_window = Some(wid.clone());
                                }
                            }
                        }
                    }

                    // Minimize small placeholder windows to get them out of the way
                    for wid in &small_windows {
                        eprintln!("X11 workaround: minimizing placeholder window {}", wid);
                        let _ = std::process::Command::new("xdotool")
                            .args(["windowminimize", wid])
                            .output();
                    }

                    // Fix and show the main window
                    if let Some(wid) = main_window {
                        // Detect primary monitor offset using xrandr
                        let mut primary_x = 0;
                        if let Ok(xrandr_output) = std::process::Command::new("xrandr")
                            .arg("--query")
                            .output()
                        {
                            let xrandr_str = String::from_utf8_lossy(&xrandr_output.stdout);
                            // Look for "primary WIDTHxHEIGHT+X+Y" pattern
                            for line in xrandr_str.lines() {
                                if line.contains(" primary ") {
                                    // Parse: "DVI-D-0 connected primary 1920x1200+1920+0"
                                    if let Some(pos) = line.find('+') {
                                        let after_plus = &line[pos + 1..];
                                        if let Some(end) = after_plus.find('+') {
                                            if let Ok(x) = after_plus[..end].parse::<i32>() {
                                                primary_x = x;
                                                eprintln!("X11 workaround: primary monitor at x={}", primary_x);
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }

                        eprintln!("X11 workaround: fixing main window {} via xdotool", wid);
                        // Move to visible position on PRIMARY monitor
                        let target_x = (primary_x + 100).to_string();
                        let _ = std::process::Command::new("xdotool")
                            .args(["windowmove", "--sync", &wid, &target_x, "100"])
                            .output();
                        // Set proper size
                        let _ = std::process::Command::new("xdotool")
                            .args(["windowsize", "--sync", &wid, "1200", "800"])
                            .output();
                        // Activate, focus, and raise the window
                        let _ = std::process::Command::new("xdotool")
                            .args(["windowactivate", "--sync", &wid])
                            .output();
                        let _ = std::process::Command::new("xdotool")
                            .args(["windowfocus", &wid])
                            .output();
                        let _ = std::process::Command::new("xdotool")
                            .args(["windowraise", &wid])
                            .output();
                    } else {
                        eprintln!("X11 workaround: no main window found to fix");
                    }
                }
            });
        }
    }

    pub fn widget(&self) -> &ApplicationWindow {
        &self.window
    }
}

/// Execute a command with safe mode checking
fn execute_command(
    cmd: &str,
    user_text: &str,
    tabs: &Rc<TerminalTabs>,
    safe_mode: &Rc<RefCell<SafeMode>>,
    safe_mode_preview: &Rc<crate::widgets::safe_mode_preview::SafeModePreviewWidget>,
    nl_input: &Rc<NaturalLanguageInput>,
    nl_status: &Label,
) {
    // Check if safe mode is enabled
    let analyzer = safe_mode.borrow();
    if analyzer.enabled {
        // Get current working directory from terminal (or use home)
        let cwd = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()));
        let preview = analyzer.analyze(cmd, &cwd);

        // Show preview for risky commands
        if preview.risk != corgiterm_core::RiskLevel::Safe {
            drop(analyzer); // Drop borrow before showing preview
            safe_mode_preview.show_preview(&preview);
            nl_input.clear();
            return;
        }
    }
    drop(analyzer);

    // Safe command - execute directly
    if tabs.send_command_to_current(cmd) {
        tracing::info!("Executed NL command: {} -> {}", user_text, cmd);
        nl_input.clear();
    } else {
        nl_status.set_text("No terminal tab selected");
        nl_status.set_visible(true);
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_secs(2), {
            let status = nl_status.clone();
            move || status.set_visible(false)
        });
    }
}

/// Quick pattern-based translation for common commands (no AI needed)
/// Returns None if AI translation is needed
fn quick_translate(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();

    // File listing patterns
    if input_lower.contains("list") || input_lower.contains("show") {
        if input_lower.contains("file") {
            if input_lower.contains("hidden") {
                return Some("ls -la".to_string());
            }
            if input_lower.contains("big")
                || input_lower.contains("large")
                || input_lower.contains("size")
            {
                // Extract size if specified
                if let Some(size) = extract_size(&input_lower) {
                    return Some(format!("find . -size +{} -type f", size));
                }
                return Some("find . -size +100M -type f".to_string());
            }
            return Some("ls -l".to_string());
        }
        if input_lower.contains("director") {
            return Some("ls -d */".to_string());
        }
        if input_lower.contains("process") {
            return Some("ps aux".to_string());
        }
    }

    // Disk usage
    if input_lower.contains("disk")
        && (input_lower.contains("usage") || input_lower.contains("space"))
    {
        return Some("df -h".to_string());
    }
    if input_lower.contains("folder") && input_lower.contains("size") {
        return Some("du -sh */".to_string());
    }

    // Git patterns
    if input_lower.contains("git") || input_lower.starts_with("commit") {
        if input_lower.contains("status") {
            return Some("git status".to_string());
        }
        if input_lower.contains("log") || input_lower.contains("history") {
            return Some("git log --oneline -10".to_string());
        }
        if input_lower.contains("branch") {
            return Some("git branch -a".to_string());
        }
        if input_lower.contains("diff") {
            return Some("git diff".to_string());
        }
    }

    // Network patterns
    if input_lower.contains("port") {
        if let Some(port) = extract_port(&input_lower) {
            return Some(format!("lsof -i :{}", port));
        }
        return Some("ss -tlnp".to_string());
    }
    if input_lower.contains("network") || input_lower.contains("ip address") {
        return Some("ip addr".to_string());
    }

    // Process patterns
    if (input_lower.contains("kill") || input_lower.contains("stop"))
        && input_lower.contains("process")
    {
        return Some("# Use: kill <PID> or killall <name>".to_string());
    }
    if input_lower.contains("running") && input_lower.contains("process") {
        return Some("ps aux | head -20".to_string());
    }

    // System info
    if input_lower.contains("memory") || input_lower.contains("ram") {
        return Some("free -h".to_string());
    }
    if input_lower.contains("cpu") && input_lower.contains("usage") {
        return Some("top -bn1 | head -15".to_string());
    }

    // Directory navigation
    if input_lower.contains("go to") || input_lower.contains("change to") {
        if input_lower.contains("home") {
            return Some("cd ~".to_string());
        }
        if input_lower.contains("parent") || input_lower.contains("up") {
            return Some("cd ..".to_string());
        }
    }

    // Current directory
    if input_lower.contains("where am i")
        || input_lower.contains("current director")
        || input_lower.contains("pwd")
    {
        return Some("pwd".to_string());
    }

    // Delete patterns
    if input_lower.contains("delete") || input_lower.contains("remove") {
        if input_lower.contains("node_module") {
            return Some("rm -rf node_modules".to_string());
        }
        if input_lower.contains("cache") {
            return Some("rm -rf .cache".to_string());
        }
    }

    // Docker patterns
    if input_lower.contains("docker") {
        if input_lower.contains("container")
            && (input_lower.contains("list") || input_lower.contains("show"))
        {
            return Some("docker ps -a".to_string());
        }
        if input_lower.contains("image") {
            return Some("docker images".to_string());
        }
    }

    // Search patterns
    if input_lower.contains("find") || input_lower.contains("search") {
        if input_lower.contains("file") && input_lower.contains("named") {
            // TODO: Extract filename from input
            return Some("find . -name '*PATTERN*'".to_string());
        }
        if input_lower.contains("text") || input_lower.contains("content") {
            return Some("grep -r 'PATTERN' .".to_string());
        }
    }

    // If no pattern matches, return None to indicate AI translation needed
    None
}

/// Extract size specification from text (e.g., "1GB", "500MB")
fn extract_size(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+)\s*(gb|mb|kb|g|m|k)").ok()?;
    if let Some(caps) = re.captures(text) {
        let num = caps.get(1)?.as_str();
        let unit = caps.get(2)?.as_str().to_uppercase();
        let unit_char = match unit.as_str() {
            "GB" | "G" => "G",
            "MB" | "M" => "M",
            "KB" | "K" => "k",
            _ => "M",
        };
        return Some(format!("{}{}", num, unit_char));
    }
    None
}

/// Extract port number from text
fn extract_port(text: &str) -> Option<u16> {
    let re = regex::Regex::new(r"port\s*(\d+)").ok()?;
    if let Some(caps) = re.captures(text) {
        return caps.get(1)?.as_str().parse().ok();
    }
    // Try just finding a 4-5 digit number that looks like a port
    let re2 = regex::Regex::new(r"\b(\d{4,5})\b").ok()?;
    if let Some(caps) = re2.captures(text) {
        let port: u16 = caps.get(1)?.as_str().parse().ok()?;
        if port >= 1024 {
            return Some(port);
        }
    }
    None
}
