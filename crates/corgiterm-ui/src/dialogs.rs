//! Dialog windows (settings, about, etc.)

use gtk4::prelude::*;
use gtk4::Window;
use libadwaita::prelude::*;

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
