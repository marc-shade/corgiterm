//! Dialog windows (settings, about, etc.)

use gtk4::prelude::*;
use gtk4::Window;
use libadwaita::prelude::*;
use corgiterm_config::ConfigManager;
use std::sync::Arc;
use parking_lot::RwLock;

/// Global config manager (initialized by app)
static CONFIG: std::sync::OnceLock<Arc<RwLock<ConfigManager>>> = std::sync::OnceLock::new();

/// Initialize the global config manager
pub fn init_config(config: Arc<RwLock<ConfigManager>>) {
    let _ = CONFIG.set(config);
}

/// Get the global config manager
pub fn get_config() -> Option<Arc<RwLock<ConfigManager>>> {
    CONFIG.get().cloned()
}

/// Show the about dialog
pub fn show_about_dialog<W: IsA<Window> + IsA<gtk4::Widget>>(parent: &W) {
    let dialog = libadwaita::AboutDialog::builder()
        .application_name("CorgiTerm")
        .application_icon("dev.corgiterm.CorgiTerm")
        .developers(vec!["CorgiTerm Team".to_string()])
        .version(crate::version())
        .website("https://corgiterm.dev")
        .issue_url("https://github.com/corgiterm/corgiterm/issues")
        .license_type(gtk4::License::MitX11)
        .comments("A next-generation, AI-powered terminal emulator that makes the command line accessible to everyone.")
        .build();

    // Add mascot credit
    dialog.add_credit_section(Some("Mascot"), &["Pixel the Corgi 🐕"]);

    dialog.present(Some(parent));
}

/// Show the preferences dialog
pub fn show_preferences<W: IsA<Window> + IsA<gtk4::Widget>>(parent: &W) {
    let dialog = libadwaita::PreferencesDialog::builder()
        .title("Preferences")
        .build();

    // General page
    let general_page = libadwaita::PreferencesPage::builder()
        .title("General")
        .icon_name("preferences-system-symbolic")
        .build();

    let startup_group = libadwaita::PreferencesGroup::builder()
        .title("Startup")
        .build();

    // Get current startup settings from config
    let (restore_sessions, show_welcome) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (config.general.restore_sessions, config.general.show_welcome)
    } else {
        (true, true)
    };

    let restore_switch = libadwaita::SwitchRow::builder()
        .title("Restore Previous Session")
        .subtitle("Open windows and tabs from last time")
        .active(restore_sessions)
        .build();
    startup_group.add(&restore_switch);

    // Connect restore sessions toggle
    restore_switch.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.general.restore_sessions = active;
            });
            let _ = config_manager.read().save();
            tracing::info!("Restore sessions {}", if active { "enabled" } else { "disabled" });
        }
    });

    let welcome_switch = libadwaita::SwitchRow::builder()
        .title("Show Welcome Screen")
        .subtitle("Display tips for new users")
        .active(show_welcome)
        .build();
    startup_group.add(&welcome_switch);

    // Connect welcome screen toggle
    welcome_switch.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.general.show_welcome = active;
            });
            let _ = config_manager.read().save();
            tracing::info!("Welcome screen {}", if active { "enabled" } else { "disabled" });
        }
    });

    general_page.add(&startup_group);
    dialog.add(&general_page);

    // Appearance page
    let appearance_page = libadwaita::PreferencesPage::builder()
        .title("Appearance")
        .icon_name("applications-graphics-symbolic")
        .build();

    let theme_group = libadwaita::PreferencesGroup::builder()
        .title("Theme")
        .build();

    // Get current theme from config
    let current_theme = if let Some(config_manager) = get_config() {
        config_manager.read().config().appearance.theme.clone()
    } else {
        "Corgi Dark".to_string()
    };

    let theme_row = libadwaita::ComboRow::builder()
        .title("Color Theme")
        .subtitle("Choose your preferred color scheme")
        .build();
    let themes = ["Corgi Dark", "Corgi Light", "Corgi Sunset", "Pembroke"];
    theme_row.set_model(Some(&gtk4::StringList::new(&themes)));

    // Set current theme selection
    if let Some(pos) = themes.iter().position(|&t| t == current_theme) {
        theme_row.set_selected(pos as u32);
    }
    theme_group.add(&theme_row);

    // Connect theme change
    theme_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if selected < themes.len() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.appearance.theme = themes[selected].to_string();
                });
                let _ = config_manager.read().save();
                tracing::info!("Theme changed to: {}", themes[selected]);
            }
        }
    });

    appearance_page.add(&theme_group);

    // Font settings group
    let font_group = libadwaita::PreferencesGroup::builder()
        .title("Font")
        .description("Configure terminal font appearance")
        .build();

    // Get current config values
    let (current_font, current_size) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (config.appearance.font_family.clone(), config.appearance.font_size)
    } else {
        ("Source Code Pro".to_string(), 11.0)
    };

    // Font family dropdown
    let font_row = libadwaita::ComboRow::builder()
        .title("Font Family")
        .subtitle("Monospace font for terminal text")
        .build();
    let fonts = ["Source Code Pro", "DejaVu Sans Mono", "Liberation Mono", "Adwaita Mono", "Monospace"];
    font_row.set_model(Some(&gtk4::StringList::new(&fonts)));
    // Set current font
    if let Some(pos) = fonts.iter().position(|&f| f == current_font) {
        font_row.set_selected(pos as u32);
    }
    font_group.add(&font_row);

    // Connect font change
    font_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if selected < fonts.len() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.appearance.font_family = fonts[selected].to_string();
                });
                let _ = config_manager.read().save();
                tracing::info!("Font changed to: {}", fonts[selected]);
            }
        }
    });

    // Font size spin row
    let size_adj = gtk4::Adjustment::new(current_size as f64, 8.0, 24.0, 1.0, 2.0, 0.0);
    let size_row = libadwaita::SpinRow::builder()
        .title("Font Size")
        .subtitle("Size in points")
        .adjustment(&size_adj)
        .build();
    font_group.add(&size_row);

    // Connect size change
    size_row.connect_changed(move |row| {
        let size = row.value() as f32;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.font_size = size;
            });
            let _ = config_manager.read().save();
            tracing::info!("Font size changed to: {}", size);
        }
    });

    appearance_page.add(&font_group);
    dialog.add(&appearance_page);

    // Terminal page
    let terminal_page = libadwaita::PreferencesPage::builder()
        .title("Terminal")
        .icon_name("utilities-terminal-symbolic")
        .build();

    let shell_group = libadwaita::PreferencesGroup::builder()
        .title("Shell")
        .description("Configure your terminal shell")
        .build();

    // Get current shell from config
    let current_shell = if let Some(config_manager) = get_config() {
        config_manager.read().config().general.shell.clone()
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    };

    let shell_row = libadwaita::EntryRow::builder()
        .title("Default Shell")
        .text(&current_shell)
        .build();
    shell_group.add(&shell_row);

    // Connect shell change
    shell_row.connect_changed(move |row| {
        let shell = row.text().to_string();
        if !shell.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.general.shell = shell.clone();
                });
                let _ = config_manager.read().save();
                tracing::info!("Shell changed to: {}", shell);
            }
        }
    });

    terminal_page.add(&shell_group);
    dialog.add(&terminal_page);

    // AI page
    let ai_page = libadwaita::PreferencesPage::builder()
        .title("AI")
        .icon_name("face-smile-symbolic")
        .build();

    let ai_group = libadwaita::PreferencesGroup::builder()
        .title("AI Features")
        .description("Configure AI-powered assistance")
        .build();

    // Get current AI settings
    let (ai_is_enabled, natural_lang_enabled) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (config.ai.enabled, config.ai.natural_language)
    } else {
        (true, true)
    };

    let ai_enabled = libadwaita::SwitchRow::builder()
        .title("Enable AI Features")
        .subtitle("Use AI to help with commands")
        .active(ai_is_enabled)
        .build();
    ai_group.add(&ai_enabled);

    // Connect AI enabled toggle
    ai_enabled.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.enabled = active;
            });
            let _ = config_manager.read().save();
            tracing::info!("AI features {}", if active { "enabled" } else { "disabled" });
        }
    });

    let natural_lang = libadwaita::SwitchRow::builder()
        .title("Natural Language Input")
        .subtitle("Type commands in plain English")
        .active(natural_lang_enabled)
        .build();
    ai_group.add(&natural_lang);

    // Connect natural language toggle
    natural_lang.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.natural_language = active;
            });
            let _ = config_manager.read().save();
            tracing::info!("Natural language input {}", if active { "enabled" } else { "disabled" });
        }
    });

    ai_page.add(&ai_group);
    dialog.add(&ai_page);

    // Safe Mode page
    let safe_page = libadwaita::PreferencesPage::builder()
        .title("Safe Mode")
        .icon_name("security-high-symbolic")
        .build();

    let safe_group = libadwaita::PreferencesGroup::builder()
        .title("Command Safety")
        .description("Preview commands before execution")
        .build();

    // Get current safe mode settings
    let safe_is_enabled = if let Some(config_manager) = get_config() {
        config_manager.read().config().safe_mode.enabled
    } else {
        false
    };

    let safe_enabled = libadwaita::SwitchRow::builder()
        .title("Enable Safe Mode")
        .subtitle("Show preview for dangerous commands")
        .active(safe_is_enabled)
        .build();
    safe_group.add(&safe_enabled);

    // Connect safe mode toggle
    safe_enabled.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.safe_mode.enabled = active;
            });
            let _ = config_manager.read().save();
            tracing::info!("Safe mode {}", if active { "enabled" } else { "disabled" });
        }
    });

    safe_page.add(&safe_group);
    dialog.add(&safe_page);

    dialog.present(Some(parent));
}

/// Show keyboard shortcuts dialog
pub fn show_shortcuts_dialog<W: IsA<Window> + IsA<gtk4::Widget>>(parent: &W) {
    let dialog = gtk4::ShortcutsWindow::builder()
        .transient_for(parent)
        .modal(true)
        .build();

    // Create shortcuts section for terminal
    let terminal_section = gtk4::ShortcutsSection::builder()
        .section_name("terminal")
        .title("Terminal")
        .build();

    // Tab management group
    let tab_group = gtk4::ShortcutsGroup::builder()
        .title("Tabs")
        .build();

    let shortcuts = [
        ("New Tab", "<Ctrl>t"),
        ("Close Tab", "<Ctrl>w"),
        ("Next Tab", "<Ctrl>Tab"),
        ("Previous Tab", "<Ctrl><Shift>Tab"),
        ("Switch to Tab 1-9", "<Ctrl>1...9"),
    ];

    for (title, accel) in shortcuts {
        let shortcut = gtk4::ShortcutsShortcut::builder()
            .title(title)
            .accelerator(accel)
            .build();
        tab_group.append(&shortcut);
    }
    terminal_section.append(&tab_group);

    // Window group
    let window_group = gtk4::ShortcutsGroup::builder()
        .title("Window")
        .build();

    let window_shortcuts = [
        ("Quick Switcher", "<Ctrl>k"),
        ("Quit", "<Ctrl>q"),
    ];

    for (title, accel) in window_shortcuts {
        let shortcut = gtk4::ShortcutsShortcut::builder()
            .title(title)
            .accelerator(accel)
            .build();
        window_group.append(&shortcut);
    }
    terminal_section.append(&window_group);

    // Terminal group
    let term_group = gtk4::ShortcutsGroup::builder()
        .title("Terminal")
        .build();

    let term_shortcuts = [
        ("Copy", "<Ctrl><Shift>c"),
        ("Paste", "<Ctrl><Shift>v"),
        ("Select All", "<Ctrl><Shift>a"),
        ("Find in Terminal", "<Ctrl><Shift>f"),
        ("Zoom In", "<Ctrl>plus"),
        ("Zoom Out", "<Ctrl>minus"),
        ("Reset Zoom", "<Ctrl>0"),
    ];

    for (title, accel) in term_shortcuts {
        let shortcut = gtk4::ShortcutsShortcut::builder()
            .title(title)
            .accelerator(accel)
            .build();
        term_group.append(&shortcut);
    }
    terminal_section.append(&term_group);

    // Documents group
    let doc_group = gtk4::ShortcutsGroup::builder()
        .title("Documents")
        .build();

    let doc_shortcuts = [
        ("New Document", "<Ctrl>o"),
        ("Open File", "<Ctrl><Shift>o"),
    ];

    for (title, accel) in doc_shortcuts {
        let shortcut = gtk4::ShortcutsShortcut::builder()
            .title(title)
            .accelerator(accel)
            .build();
        doc_group.append(&shortcut);
    }
    terminal_section.append(&doc_group);

    dialog.add_section(&terminal_section);
    dialog.present();
}

/// Show quick switcher (Ctrl+K style) for switching between tabs and actions
pub fn show_quick_switcher<W: IsA<Window> + IsA<gtk4::Widget>>(
    parent: &W,
    tab_view: &libadwaita::TabView,
) {
    use gtk4::{Label, ListBox, ListBoxRow, Orientation, SelectionMode, ScrolledWindow};

    // Create modal dialog
    let dialog = libadwaita::Dialog::builder()
        .title("Quick Switcher")
        .content_width(500)
        .content_height(400)
        .build();

    // Main container
    let main_box = gtk4::Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Search entry
    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Type to search tabs...")
        .hexpand(true)
        .build();
    main_box.append(&search_entry);

    // Scrolled list of tabs
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .min_content_height(300)
        .build();

    let list_box = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .css_classes(vec!["boxed-list"])
        .build();

    // Collect tab info (index, title, icon)
    let tab_view_ref = tab_view.clone();
    let n_pages = tab_view.n_pages();

    for i in 0..n_pages {
        let page = tab_view.nth_page(i);
        let title = page.title().to_string();
        let icon_name = page.icon()
            .and_then(|icon| icon.downcast::<gtk4::gio::ThemedIcon>().ok())
            .and_then(|themed| themed.names().first().map(|s| s.to_string()))
            .unwrap_or_else(|| "utilities-terminal-symbolic".to_string());

        // Create row with icon and title
        let row_box = gtk4::Box::new(Orientation::Horizontal, 12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let icon = gtk4::Image::from_icon_name(&icon_name);
        icon.set_pixel_size(24);
        row_box.append(&icon);

        let title_label = Label::new(Some(&title));
        title_label.set_halign(gtk4::Align::Start);
        title_label.set_hexpand(true);
        row_box.append(&title_label);

        // Tab number hint
        let hint_label = Label::new(Some(&format!("Ctrl+{}", i + 1)));
        hint_label.add_css_class("dim-label");
        row_box.append(&hint_label);

        let row = ListBoxRow::new();
        row.set_child(Some(&row_box));
        // Store the page index in the widget name for retrieval
        row.set_widget_name(&format!("tab-{}", i));
        list_box.append(&row);
    }

    // Select first row by default
    if let Some(first_row) = list_box.row_at_index(0) {
        list_box.select_row(Some(&first_row));
    }

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    // Handle row activation (click or Enter)
    let dialog_ref = dialog.clone();
    let tab_view_for_activate = tab_view_ref.clone();
    list_box.connect_row_activated(move |_, row| {
        let name = row.widget_name();
        if let Some(idx_str) = name.strip_prefix("tab-") {
            if let Ok(idx) = idx_str.parse::<i32>() {
                let page = tab_view_for_activate.nth_page(idx);
                tab_view_for_activate.set_selected_page(&page);
            }
        }
        dialog_ref.close();
    });

    // Handle search filtering
    let list_box_for_filter = list_box.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let mut first_visible: Option<ListBoxRow> = None;

        for i in 0..n_pages {
            if let Some(row) = list_box_for_filter.row_at_index(i) {
                // Get the title from the row's box child
                let visible = if query.is_empty() {
                    true
                } else if let Some(row_box) = row.child().and_then(|c| c.downcast::<gtk4::Box>().ok()) {
                    let mut found = false;
                    let mut child = row_box.first_child();
                    while let Some(widget) = child {
                        if let Ok(label) = widget.clone().downcast::<Label>() {
                            if label.text().to_lowercase().contains(&query) {
                                found = true;
                                break;
                            }
                        }
                        child = widget.next_sibling();
                    }
                    found
                } else {
                    true
                };
                row.set_visible(visible);
                if visible && first_visible.is_none() {
                    first_visible = Some(row);
                }
            }
        }

        // Select first visible row
        if let Some(row) = first_visible {
            list_box_for_filter.select_row(Some(&row));
        }
    });

    // Handle Enter key on search entry
    let list_box_for_enter = list_box.clone();
    let dialog_for_enter = dialog.clone();
    let tab_view_for_enter = tab_view_ref.clone();
    search_entry.connect_activate(move |_| {
        if let Some(selected_row) = list_box_for_enter.selected_row() {
            let name = selected_row.widget_name();
            if let Some(idx_str) = name.strip_prefix("tab-") {
                if let Ok(idx) = idx_str.parse::<i32>() {
                    let page = tab_view_for_enter.nth_page(idx);
                    tab_view_for_enter.set_selected_page(&page);
                }
            }
            dialog_for_enter.close();
        }
    });

    // Handle keyboard navigation in list
    let key_controller = gtk4::EventControllerKey::new();
    let list_box_for_keys = list_box.clone();
    let dialog_for_escape = dialog.clone();
    key_controller.connect_key_pressed(move |_, key, _keycode, _modifier| {
        use gtk4::gdk::Key;

        match key {
            Key::Escape => {
                dialog_for_escape.close();
                gtk4::glib::Propagation::Stop
            }
            Key::Up => {
                if let Some(row) = list_box_for_keys.selected_row() {
                    let idx = row.index();
                    // Find previous visible row
                    for i in (0..idx).rev() {
                        if let Some(prev_row) = list_box_for_keys.row_at_index(i) {
                            if prev_row.is_visible() {
                                list_box_for_keys.select_row(Some(&prev_row));
                                break;
                            }
                        }
                    }
                }
                gtk4::glib::Propagation::Stop
            }
            Key::Down => {
                if let Some(row) = list_box_for_keys.selected_row() {
                    let idx = row.index();
                    // Find next visible row
                    for i in (idx + 1).. {
                        if let Some(next_row) = list_box_for_keys.row_at_index(i) {
                            if next_row.is_visible() {
                                list_box_for_keys.select_row(Some(&next_row));
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                gtk4::glib::Propagation::Stop
            }
            _ => gtk4::glib::Propagation::Proceed
        }
    });
    search_entry.add_controller(key_controller);

    dialog.set_child(Some(&main_box));
    dialog.present(Some(parent));

    // Focus search entry
    search_entry.grab_focus();
}
