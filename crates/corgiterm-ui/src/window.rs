//! Main application window

use gtk4::prelude::*;
use gtk4::{Application, Box, Button, EventControllerKey, FileDialog, FileFilter, Label, MenuButton, Orientation, Paned, Revealer, RevealerTransitionType, Spinner};
use gtk4::gdk::ModifierType;
use gtk4::gio::{self, Menu, SimpleAction};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, HeaderBar, WindowTitle};
use std::cell::RefCell;
use std::rc::Rc;

use crate::dialogs;
use crate::sidebar::Sidebar;
use crate::tab_bar::TerminalTabs;
use crate::ai_panel::AiPanel;
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

        // Create components
        let sidebar = Rc::new(Sidebar::new());
        let tabs = Rc::new(TerminalTabs::new());

        // Create header bar
        let header = HeaderBar::new();

        // Window title
        let title = WindowTitle::new("CorgiTerm", "~/");
        header.set_title_widget(Some(&title));

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
        prefs_action.connect_activate(move |_, _| {
            dialogs::show_preferences(&win_for_prefs);
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

        // Main layout with header + content
        let main_box = Box::new(Orientation::Vertical, 0);

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

        // Terminal area with NL input at bottom
        let terminal_area = Box::new(Orientation::Vertical, 0);
        terminal_area.append(tabs.tab_view_widget());
        terminal_area.append(safe_mode_preview.widget()); // Safe mode preview between terminal and input
        terminal_area.append(&nl_container);
        tabs.tab_view_widget().set_vexpand(true);

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

        // Content: sidebar + terminal area
        let content_paned = Paned::new(Orientation::Horizontal);
        content_paned.set_start_child(Some(sidebar.widget()));
        content_paned.set_end_child(Some(&terminal_area));
        content_paned.set_position(220);
        content_paned.set_shrink_start_child(false);
        content_paned.set_shrink_end_child(false);
        content_paned.set_vexpand(true);

        // Connect NL input to AI translation and terminal execution
        let tabs_for_nl = tabs.clone();
        let nl_input_for_activate = nl_input.clone();
        let nl_spinner_ref = nl_spinner.clone();
        let nl_status_ref = nl_status.clone();

        // Store current translation for use in activate handler
        let current_translation: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let translation_for_change = current_translation.clone();
        let translation_for_activate = current_translation.clone();

        // On text change: debounce and translate with AI
        nl_input.connect_changed(move |text| {
            if text.is_empty() {
                nl_input_for_activate.hide_suggestion();
                *translation_for_change.borrow_mut() = None;
                return;
            }

            // TODO: Add debounce (300ms) before AI call
            // For now, just show a placeholder suggestion for common patterns
            let suggestion = quick_translate(text);
            if let Some(cmd) = &suggestion {
                nl_input_for_activate.show_suggestion(cmd);
            }
            *translation_for_change.borrow_mut() = suggestion;
        });

        // On Enter: execute the translated command (with safe mode check)
        let nl_input_for_exec = nl_input.clone();
        let safe_mode_for_activate = safe_mode.clone();
        let safe_mode_preview_for_activate = safe_mode_preview.clone();
        nl_input.connect_activate(move || {
            let translation = translation_for_activate.borrow();
            if let Some(ref cmd) = *translation {
                // Check if safe mode is enabled
                let analyzer = safe_mode_for_activate.borrow();
                if analyzer.enabled {
                    // Get current working directory from terminal (or use home)
                    let cwd = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()));
                    let preview = analyzer.analyze(cmd, &cwd);

                    // Show preview for risky commands
                    if preview.risk != corgiterm_core::RiskLevel::Safe {
                        safe_mode_preview_for_activate.show_preview(&preview);
                        nl_input_for_exec.clear();
                        return;
                    }
                }

                // Safe command - execute directly
                if tabs_for_nl.send_command_to_current(cmd) {
                    tracing::info!("Executed NL command: {} -> {}", nl_input_for_exec.text(), cmd);
                    nl_input_for_exec.clear();
                } else {
                    nl_status_ref.set_text("No terminal tab selected");
                    nl_status_ref.set_visible(true);
                    gtk4::glib::timeout_add_local_once(std::time::Duration::from_secs(2), {
                        let status = nl_status_ref.clone();
                        move || status.set_visible(false)
                    });
                }
            }
            // Hide spinner if it was showing
            nl_spinner_ref.set_spinning(false);
            nl_spinner_ref.set_visible(false);
        });

        // Create AI panel with slide-out revealer
        let ai_panel = Rc::new(RefCell::new(AiPanel::new()));
        let ai_revealer = Revealer::new();
        ai_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
        ai_revealer.set_transition_duration(200);
        ai_revealer.set_reveal_child(false);
        ai_revealer.set_child(Some(ai_panel.borrow().widget()));
        ai_panel.borrow().widget().set_width_request(350);

        // Connect AI toggle button
        let revealer_for_toggle = ai_revealer.clone();
        ai_toggle_btn.connect_clicked(move |_| {
            let currently_revealed = revealer_for_toggle.reveals_child();
            revealer_for_toggle.set_reveal_child(!currently_revealed);
        });

        // Horizontal box for content + AI panel
        let content_box = Box::new(Orientation::Horizontal, 0);
        content_box.append(&content_paned);
        content_box.append(&ai_revealer);
        content_box.set_vexpand(true);

        main_box.append(&content_box);

        window.set_content(Some(&main_box));

        // Connect sidebar project folder clicks to tab creation
        let tabs_for_session = tabs.clone();
        sidebar.set_on_session_click(move |name, path| {
            // Open terminal in the selected folder
            tabs_for_session.add_terminal_tab(name, Some(path));
            tracing::info!("Opened terminal in: {}", path);
        });

        // Connect sidebar AI actions
        let tabs_for_ai = tabs.clone();
        sidebar.set_on_ai_action(move |action| {
            match action {
                "chat" => {
                    tabs_for_ai.add_document_tab("AI Chat", None);
                    tracing::info!("Opening AI chat");
                }
                "command" => {
                    tracing::info!("Opening AI command mode");
                }
                _ => {}
            }
        });

        // Set up keyboard shortcuts
        let key_controller = EventControllerKey::new();
        let tabs_for_keys = tabs.clone();
        let window_for_keys = window.clone();
        let ai_revealer_for_keys = ai_revealer.clone();
        let safe_mode_preview_for_keys = safe_mode_preview.clone();
        key_controller.connect_key_pressed(move |_, key, _keycode, modifier| {
            use gtk4::gdk::Key;

            // Handle Escape to close safe mode preview
            if key == Key::Escape {
                if safe_mode_preview_for_keys.is_visible() {
                    safe_mode_preview_for_keys.cancel();
                    return gtk4::glib::Propagation::Stop;
                }
            }

            let ctrl = modifier.contains(ModifierType::CONTROL_MASK);
            let shift = modifier.contains(ModifierType::SHIFT_MASK);

            if ctrl && !shift {
                match key {
                    Key::t | Key::T => {
                        // Ctrl+T: New tab
                        tabs_for_keys.add_terminal_tab("Terminal", None);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::w | Key::W => {
                        // Ctrl+W: Close tab
                        tabs_for_keys.close_current_tab();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::q | Key::Q => {
                        // Ctrl+Q: Quit
                        window_for_keys.close();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_1 => {
                        // Ctrl+1: Switch to tab 1
                        tabs_for_keys.select_tab_by_index(0);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_2 => {
                        // Ctrl+2: Switch to tab 2
                        tabs_for_keys.select_tab_by_index(1);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_3 => {
                        // Ctrl+3: Switch to tab 3
                        tabs_for_keys.select_tab_by_index(2);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_4 => {
                        // Ctrl+4: Switch to tab 4
                        tabs_for_keys.select_tab_by_index(3);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_5 => {
                        // Ctrl+5: Switch to tab 5
                        tabs_for_keys.select_tab_by_index(4);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_6 => {
                        // Ctrl+6: Switch to tab 6
                        tabs_for_keys.select_tab_by_index(5);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_7 => {
                        // Ctrl+7: Switch to tab 7
                        tabs_for_keys.select_tab_by_index(6);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_8 => {
                        // Ctrl+8: Switch to tab 8
                        tabs_for_keys.select_tab_by_index(7);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_9 => {
                        // Ctrl+9: Switch to tab 9
                        tabs_for_keys.select_tab_by_index(8);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::o | Key::O => {
                        // Ctrl+O: New document tab
                        tabs_for_keys.add_document_tab("Document", None);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::Tab => {
                        // Ctrl+Tab: Next tab
                        tabs_for_keys.select_next_tab();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::k | Key::K => {
                        // Ctrl+K: Quick switcher
                        dialogs::show_quick_switcher(&window_for_keys, tabs_for_keys.tab_view_widget());
                        return gtk4::glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            if ctrl && shift {
                match key {
                    Key::Tab | Key::ISO_Left_Tab => {
                        // Ctrl+Shift+Tab: Previous tab
                        tabs_for_keys.select_previous_tab();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::h | Key::H => {
                        // Ctrl+Shift+H: Split pane horizontally (side by side)
                        tabs_for_keys.split_current_horizontal();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::d | Key::D => {
                        // Ctrl+Shift+D: Split pane vertically (top/bottom)
                        tabs_for_keys.split_current_vertical();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::a | Key::A => {
                        // Ctrl+Shift+A: Toggle AI panel
                        let currently_revealed = ai_revealer_for_keys.reveals_child();
                        ai_revealer_for_keys.set_reveal_child(!currently_revealed);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::o | Key::O => {
                        // Ctrl+Shift+O: Open file dialog
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
                                    let filename = path.file_name()
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
                    _ => {}
                }
            }

            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        Self {
            window,
            tabs,
            sidebar,
            ai_panel,
            ai_revealer,
            nl_input,
            safe_mode_preview,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn widget(&self) -> &ApplicationWindow {
        &self.window
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
            if input_lower.contains("big") || input_lower.contains("large") || input_lower.contains("size") {
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
    if input_lower.contains("disk") && (input_lower.contains("usage") || input_lower.contains("space")) {
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
    if input_lower.contains("kill") || input_lower.contains("stop") {
        if input_lower.contains("process") {
            return Some("# Use: kill <PID> or killall <name>".to_string());
        }
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
    if input_lower.contains("where am i") || input_lower.contains("current director") || input_lower.contains("pwd") {
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
        if input_lower.contains("container") && (input_lower.contains("list") || input_lower.contains("show")) {
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
