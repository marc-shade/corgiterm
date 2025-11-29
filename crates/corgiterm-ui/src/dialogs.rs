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

    let restore_switch = libadwaita::SwitchRow::builder()
        .title("Restore Previous Session")
        .subtitle("Open windows and tabs from last time")
        .active(true)
        .build();
    startup_group.add(&restore_switch);

    let welcome_switch = libadwaita::SwitchRow::builder()
        .title("Show Welcome Screen")
        .subtitle("Display tips for new users")
        .active(true)
        .build();
    startup_group.add(&welcome_switch);

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

    let theme_row = libadwaita::ComboRow::builder()
        .title("Color Theme")
        .subtitle("Choose your preferred color scheme")
        .build();
    theme_row.set_model(Some(&gtk4::StringList::new(&[
        "Corgi Dark",
        "Corgi Light",
        "Corgi Sunset",
        "Pembroke",
    ])));
    theme_group.add(&theme_row);

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
        .build();

    let shell_row = libadwaita::EntryRow::builder()
        .title("Default Shell")
        .text("/bin/bash")
        .build();
    shell_group.add(&shell_row);

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

    let ai_enabled = libadwaita::SwitchRow::builder()
        .title("Enable AI Features")
        .active(true)
        .build();
    ai_group.add(&ai_enabled);

    let natural_lang = libadwaita::SwitchRow::builder()
        .title("Natural Language Input")
        .subtitle("Type commands in plain English")
        .active(true)
        .build();
    ai_group.add(&natural_lang);

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

    let safe_enabled = libadwaita::SwitchRow::builder()
        .title("Enable Safe Mode")
        .subtitle("Show preview for dangerous commands")
        .active(false)
        .build();
    safe_group.add(&safe_enabled);

    safe_page.add(&safe_group);
    window.add(&safe_page);

    window.present();
}

/// Show quick switcher (Cmd+K style)
pub fn show_quick_switcher(parent: &impl IsA<Window>) {
    // TODO: Implement quick switcher dialog
}
