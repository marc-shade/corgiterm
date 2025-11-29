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
pub fn show_about_dialog(parent: &impl IsA<Window>) {
    let dialog = libadwaita::AboutWindow::builder()
        .application_name("CorgiTerm")
        .application_icon("dev.corgiterm.CorgiTerm")
        .developer_name("CorgiTerm Team")
        .version(crate::version())
        .website("https://corgiterm.dev")
        .issue_url("https://github.com/corgiterm/corgiterm/issues")
        .license_type(gtk4::License::MitX11)
        .comments("A next-generation, AI-powered terminal emulator that makes the command line accessible to everyone.")
        .transient_for(parent)
        .build();

    // Add mascot credit
    dialog.add_credit_section(Some("Mascot"), &["Pixel the Corgi 🐕"]);

    dialog.present();
}

/// Show the preferences window
pub fn show_preferences(parent: &impl IsA<Window>) {
    let window = libadwaita::PreferencesWindow::builder()
        .title("Preferences")
        .transient_for(parent)
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
    window.add(&general_page);

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
    window.add(&appearance_page);

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
    window.add(&terminal_page);

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
    window.add(&ai_page);

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
    window.add(&safe_page);

    window.present();
}

/// Show quick switcher (Cmd+K style)
pub fn show_quick_switcher(parent: &impl IsA<Window>) {
    // TODO: Implement quick switcher dialog
}
