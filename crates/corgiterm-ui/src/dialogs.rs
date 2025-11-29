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
    let (restore_sessions, show_welcome, confirm_close, check_updates, telemetry) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.general.restore_sessions,
            config.general.show_welcome,
            config.general.confirm_close,
            config.general.check_updates,
            config.general.telemetry,
        )
    } else {
        (true, true, true, true, false)
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

    // Confirm close
    let confirm_row = libadwaita::SwitchRow::builder()
        .title("Confirm Before Closing")
        .subtitle("Ask before closing with active sessions")
        .active(confirm_close)
        .build();
    startup_group.add(&confirm_row);

    confirm_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.general.confirm_close = active;
            });
            let _ = config_manager.read().save();
        }
    });

    general_page.add(&startup_group);

    // Updates and Privacy group
    let updates_group = libadwaita::PreferencesGroup::builder()
        .title("Updates and Privacy")
        .build();

    let updates_row = libadwaita::SwitchRow::builder()
        .title("Check for Updates")
        .subtitle("Automatically check for new versions")
        .active(check_updates)
        .build();
    updates_group.add(&updates_row);

    updates_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.general.check_updates = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let telemetry_row = libadwaita::SwitchRow::builder()
        .title("Send Anonymous Usage Data")
        .subtitle("Help improve CorgiTerm with usage statistics")
        .active(telemetry)
        .build();
    updates_group.add(&telemetry_row);

    telemetry_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.general.telemetry = active;
            });
            let _ = config_manager.read().save();
        }
    });

    general_page.add(&updates_group);
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

    // Theme Creator button
    let creator_group = libadwaita::PreferencesGroup::builder()
        .title("Custom Themes")
        .build();

    let creator_row = libadwaita::ActionRow::builder()
        .title("Theme Creator")
        .subtitle("Create and customize your own color themes")
        .activatable(true)
        .build();

    // Add icon
    let creator_icon = gtk4::Image::from_icon_name("applications-graphics-symbolic");
    creator_row.add_prefix(&creator_icon);

    // Add arrow
    let arrow = gtk4::Image::from_icon_name("go-next-symbolic");
    creator_row.add_suffix(&arrow);

    // Connect to theme creator
    let parent_ref = parent.clone();
    creator_row.connect_activated(move |_| {
        crate::theme_creator::show_theme_creator(&parent_ref);
    });

    creator_group.add(&creator_row);
    appearance_page.add(&creator_group);

    // Font settings group
    let font_group = libadwaita::PreferencesGroup::builder()
        .title("Font")
        .description("Configure terminal font appearance")
        .build();

    // Get current config values
    let (current_font, current_size, line_height, cursor_blink_rate) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.appearance.font_family.clone(),
            config.appearance.font_size,
            config.appearance.line_height,
            config.appearance.cursor_blink_rate,
        )
    } else {
        ("Source Code Pro".to_string(), 11.0, 1.2, 530)
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

    // Line height spin row
    let line_adj = gtk4::Adjustment::new(line_height as f64, 1.0, 2.0, 0.1, 0.2, 0.0);
    let line_row = libadwaita::SpinRow::builder()
        .title("Line Height")
        .subtitle("Spacing between lines (1.0 = tight, 2.0 = double)")
        .adjustment(&line_adj)
        .digits(1)
        .build();
    font_group.add(&line_row);

    line_row.connect_changed(move |row| {
        let height = row.value() as f32;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.line_height = height;
            });
            let _ = config_manager.read().save();
        }
    });

    appearance_page.add(&font_group);

    // Window appearance group
    let window_group = libadwaita::PreferencesGroup::builder()
        .title("Window")
        .description("Window appearance settings")
        .build();

    // Get current window settings
    let (opacity, ligatures) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (config.appearance.opacity, config.appearance.ligatures)
    } else {
        (1.0, true)
    };

    // Opacity
    let opacity_adj = gtk4::Adjustment::new(opacity as f64, 0.3, 1.0, 0.05, 0.1, 0.0);
    let opacity_row = libadwaita::SpinRow::builder()
        .title("Background Opacity")
        .subtitle("Window transparency (1.0 = opaque)")
        .adjustment(&opacity_adj)
        .digits(2)
        .build();
    window_group.add(&opacity_row);

    opacity_row.connect_changed(move |row| {
        let opacity = row.value() as f32;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.opacity = opacity;
            });
            let _ = config_manager.read().save();
            tracing::info!("Opacity changed to: {}", opacity);
        }
    });

    // Ligatures
    let ligatures_row = libadwaita::SwitchRow::builder()
        .title("Font Ligatures")
        .subtitle("Enable programming font ligatures (e.g., -> becomes →)")
        .active(ligatures)
        .build();
    window_group.add(&ligatures_row);

    ligatures_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.ligatures = active;
            });
            let _ = config_manager.read().save();
        }
    });

    appearance_page.add(&window_group);
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

    // TERM environment variable
    let current_term = if let Some(config_manager) = get_config() {
        config_manager.read().config().terminal.term.clone()
    } else {
        "xterm-256color".to_string()
    };

    let term_row = libadwaita::ComboRow::builder()
        .title("TERM Variable")
        .subtitle("Terminal type for compatibility")
        .build();
    let term_types = ["xterm-256color", "xterm", "vt100", "screen-256color", "tmux-256color"];
    term_row.set_model(Some(&gtk4::StringList::new(&term_types)));
    if let Some(pos) = term_types.iter().position(|&t| t == current_term) {
        term_row.set_selected(pos as u32);
    }
    shell_group.add(&term_row);

    term_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if selected < term_types.len() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.terminal.term = term_types[selected].to_string();
                });
                let _ = config_manager.read().save();
                tracing::info!("TERM changed to: {}", term_types[selected]);
            }
        }
    });

    terminal_page.add(&shell_group);

    // Terminal behavior group
    let behavior_group = libadwaita::PreferencesGroup::builder()
        .title("Behavior")
        .description("Terminal behavior settings")
        .build();

    // Get current terminal settings
    let (scrollback, copy_on_select, scroll_on_output, scroll_on_keystroke, mouse_reporting,
         paste_middle_click, hyperlinks, close_on_exit_idx) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        let close_idx = match config.terminal.close_on_exit {
            corgiterm_config::CloseOnExit::Always => 0,
            corgiterm_config::CloseOnExit::Never => 1,
            corgiterm_config::CloseOnExit::IfClean => 2,
        };
        (
            config.terminal.scrollback_lines,
            config.terminal.copy_on_select,
            config.terminal.scroll_on_output,
            config.terminal.scroll_on_keystroke,
            config.terminal.mouse_reporting,
            config.terminal.paste_on_middle_click,
            config.terminal.hyperlinks,
            close_idx,
        )
    } else {
        (10000, false, false, true, true, true, true, 2u32)
    };

    // Scrollback lines
    let scrollback_adj = gtk4::Adjustment::new(scrollback as f64, 100.0, 100000.0, 100.0, 1000.0, 0.0);
    let scrollback_row = libadwaita::SpinRow::builder()
        .title("Scrollback Lines")
        .subtitle("Number of lines to keep in history")
        .adjustment(&scrollback_adj)
        .build();
    behavior_group.add(&scrollback_row);

    scrollback_row.connect_changed(move |row| {
        let lines = row.value() as usize;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.scrollback_lines = lines;
            });
            let _ = config_manager.read().save();
            tracing::info!("Scrollback lines changed to: {}", lines);
        }
    });

    // Copy on select
    let copy_select_row = libadwaita::SwitchRow::builder()
        .title("Copy on Select")
        .subtitle("Automatically copy selected text")
        .active(copy_on_select)
        .build();
    behavior_group.add(&copy_select_row);

    copy_select_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.copy_on_select = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Scroll on output
    let scroll_output_row = libadwaita::SwitchRow::builder()
        .title("Scroll on Output")
        .subtitle("Auto-scroll when new output appears")
        .active(scroll_on_output)
        .build();
    behavior_group.add(&scroll_output_row);

    scroll_output_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.scroll_on_output = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Scroll on keystroke
    let scroll_key_row = libadwaita::SwitchRow::builder()
        .title("Scroll on Keystroke")
        .subtitle("Auto-scroll when typing")
        .active(scroll_on_keystroke)
        .build();
    behavior_group.add(&scroll_key_row);

    scroll_key_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.scroll_on_keystroke = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Mouse reporting
    let mouse_row = libadwaita::SwitchRow::builder()
        .title("Mouse Reporting")
        .subtitle("Send mouse events to applications")
        .active(mouse_reporting)
        .build();
    behavior_group.add(&mouse_row);

    mouse_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.mouse_reporting = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Paste on middle click
    let paste_row = libadwaita::SwitchRow::builder()
        .title("Paste on Middle Click")
        .subtitle("Paste clipboard on middle mouse button")
        .active(paste_middle_click)
        .build();
    behavior_group.add(&paste_row);

    paste_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.paste_on_middle_click = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Hyperlinks
    let hyperlinks_row = libadwaita::SwitchRow::builder()
        .title("Clickable Hyperlinks")
        .subtitle("Detect and highlight URLs for clicking")
        .active(hyperlinks)
        .build();
    behavior_group.add(&hyperlinks_row);

    hyperlinks_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.hyperlinks = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Close on exit
    let close_exit_row = libadwaita::ComboRow::builder()
        .title("Close Tab on Exit")
        .subtitle("When to close tab after shell exits")
        .build();
    let close_options = ["Always", "Never", "If Clean Exit (code 0)"];
    close_exit_row.set_model(Some(&gtk4::StringList::new(&close_options)));
    close_exit_row.set_selected(close_on_exit_idx);
    behavior_group.add(&close_exit_row);

    close_exit_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.close_on_exit = match selected {
                    0 => corgiterm_config::CloseOnExit::Always,
                    1 => corgiterm_config::CloseOnExit::Never,
                    2 => corgiterm_config::CloseOnExit::IfClean,
                    _ => corgiterm_config::CloseOnExit::IfClean,
                };
            });
            let _ = config_manager.read().save();
        }
    });

    // Bell style
    let bell_style_idx = if let Some(config_manager) = get_config() {
        match config_manager.read().config().terminal.bell_style {
            corgiterm_config::BellStyle::None => 0,
            corgiterm_config::BellStyle::Visual => 1,
            corgiterm_config::BellStyle::Audible => 2,
            corgiterm_config::BellStyle::Both => 3,
        }
    } else {
        1 // Visual by default
    };

    let bell_row = libadwaita::ComboRow::builder()
        .title("Bell Style")
        .subtitle("How to notify on terminal bell")
        .build();
    let bell_styles = ["None", "Visual", "Audible", "Both"];
    bell_row.set_model(Some(&gtk4::StringList::new(&bell_styles)));
    bell_row.set_selected(bell_style_idx);
    behavior_group.add(&bell_row);

    bell_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.terminal.bell_style = match selected {
                    0 => corgiterm_config::BellStyle::None,
                    1 => corgiterm_config::BellStyle::Visual,
                    2 => corgiterm_config::BellStyle::Audible,
                    3 => corgiterm_config::BellStyle::Both,
                    _ => corgiterm_config::BellStyle::Visual,
                };
            });
            let _ = config_manager.read().save();
            tracing::info!("Bell style changed to: {}", bell_styles[selected]);
        }
    });

    terminal_page.add(&behavior_group);

    // Cursor group
    let cursor_group = libadwaita::PreferencesGroup::builder()
        .title("Cursor")
        .description("Cursor appearance settings")
        .build();

    // Get current cursor settings
    let (cursor_style_idx, cursor_blink) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        let idx = match config.appearance.cursor_style {
            corgiterm_config::CursorStyle::Block => 0,
            corgiterm_config::CursorStyle::Underline => 1,
            corgiterm_config::CursorStyle::Bar => 2,
            corgiterm_config::CursorStyle::Hollow => 3,
        };
        (idx, config.appearance.cursor_blink)
    } else {
        (0, true)
    };

    // Cursor style
    let cursor_row = libadwaita::ComboRow::builder()
        .title("Cursor Style")
        .subtitle("Shape of the cursor")
        .build();
    let cursor_styles = ["Block", "Underline", "Bar", "Hollow"];
    cursor_row.set_model(Some(&gtk4::StringList::new(&cursor_styles)));
    cursor_row.set_selected(cursor_style_idx);
    cursor_group.add(&cursor_row);

    cursor_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.cursor_style = match selected {
                    0 => corgiterm_config::CursorStyle::Block,
                    1 => corgiterm_config::CursorStyle::Underline,
                    2 => corgiterm_config::CursorStyle::Bar,
                    3 => corgiterm_config::CursorStyle::Hollow,
                    _ => corgiterm_config::CursorStyle::Block,
                };
            });
            let _ = config_manager.read().save();
            tracing::info!("Cursor style changed to: {}", cursor_styles[selected]);
        }
    });

    // Cursor blink
    let blink_row = libadwaita::SwitchRow::builder()
        .title("Cursor Blink")
        .subtitle("Animate the cursor")
        .active(cursor_blink)
        .build();
    cursor_group.add(&blink_row);

    blink_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.cursor_blink = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Cursor blink rate
    let blink_rate_adj = gtk4::Adjustment::new(cursor_blink_rate as f64, 200.0, 1500.0, 50.0, 100.0, 0.0);
    let blink_rate_row = libadwaita::SpinRow::builder()
        .title("Blink Rate")
        .subtitle("Milliseconds between blink cycles")
        .adjustment(&blink_rate_adj)
        .build();
    cursor_group.add(&blink_rate_row);

    blink_rate_row.connect_changed(move |row| {
        let rate = row.value() as u32;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.appearance.cursor_blink_rate = rate;
            });
            let _ = config_manager.read().save();
        }
    });

    terminal_page.add(&cursor_group);
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

    // AI Provider group
    let provider_group = libadwaita::PreferencesGroup::builder()
        .title("Provider")
        .description("Configure AI provider and API keys")
        .build();

    // Get current provider settings
    let (default_provider, claude_key, claude_model, openai_key, openai_model,
         gemini_key, gemini_model, local_enabled, local_endpoint, local_model, auto_suggest) =
        if let Some(config_manager) = get_config() {
            let config = config_manager.read().config();
            (
                config.ai.default_provider.clone(),
                config.ai.claude.api_key.clone().unwrap_or_default(),
                config.ai.claude.model.clone(),
                config.ai.openai.api_key.clone().unwrap_or_default(),
                config.ai.openai.model.clone(),
                config.ai.gemini.api_key.clone().unwrap_or_default(),
                config.ai.gemini.model.clone(),
                config.ai.local.enabled,
                config.ai.local.endpoint.clone(),
                config.ai.local.model.clone(),
                config.ai.auto_suggest,
            )
        } else {
            ("claude".to_string(), String::new(), "claude-sonnet-4-20250514".to_string(),
             String::new(), "gpt-4o".to_string(), String::new(), "gemini-1.5-pro".to_string(),
             false, "http://localhost:11434".to_string(), "llama3".to_string(), true)
        };

    // Provider selection
    let provider_row = libadwaita::ComboRow::builder()
        .title("Default Provider")
        .subtitle("AI provider to use for commands")
        .build();
    let providers = ["claude", "openai", "gemini", "local"];
    provider_row.set_model(Some(&gtk4::StringList::new(&providers)));
    if let Some(pos) = providers.iter().position(|&p| p == default_provider) {
        provider_row.set_selected(pos as u32);
    }
    provider_group.add(&provider_row);

    provider_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if selected < providers.len() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ai.default_provider = providers[selected].to_string();
                });
                let _ = config_manager.read().save();
                tracing::info!("AI provider changed to: {}", providers[selected]);
            }
        }
    });

    // Auto-suggest toggle
    let suggest_row = libadwaita::SwitchRow::builder()
        .title("Auto-Suggest")
        .subtitle("Show AI suggestions while typing")
        .active(auto_suggest)
        .build();
    provider_group.add(&suggest_row);

    suggest_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.auto_suggest = active;
            });
            let _ = config_manager.read().save();
        }
    });

    ai_page.add(&provider_group);

    // Claude group
    let claude_group = libadwaita::PreferencesGroup::builder()
        .title("Claude (Anthropic)")
        .build();

    let claude_key_row = libadwaita::PasswordEntryRow::builder()
        .title("API Key")
        .text(&claude_key)
        .build();
    claude_group.add(&claude_key_row);

    claude_key_row.connect_changed(move |row| {
        let key = row.text().to_string();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.claude.api_key = if key.is_empty() { None } else { Some(key) };
            });
            let _ = config_manager.read().save();
        }
    });

    let claude_model_row = libadwaita::EntryRow::builder()
        .title("Model")
        .text(&claude_model)
        .build();
    claude_group.add(&claude_model_row);

    claude_model_row.connect_changed(move |row| {
        let model = row.text().to_string();
        if !model.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ai.claude.model = model;
                });
                let _ = config_manager.read().save();
            }
        }
    });

    ai_page.add(&claude_group);

    // OpenAI group
    let openai_group = libadwaita::PreferencesGroup::builder()
        .title("OpenAI")
        .build();

    let openai_key_row = libadwaita::PasswordEntryRow::builder()
        .title("API Key")
        .text(&openai_key)
        .build();
    openai_group.add(&openai_key_row);

    openai_key_row.connect_changed(move |row| {
        let key = row.text().to_string();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.openai.api_key = if key.is_empty() { None } else { Some(key) };
            });
            let _ = config_manager.read().save();
        }
    });

    let openai_model_row = libadwaita::EntryRow::builder()
        .title("Model")
        .text(&openai_model)
        .build();
    openai_group.add(&openai_model_row);

    openai_model_row.connect_changed(move |row| {
        let model = row.text().to_string();
        if !model.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ai.openai.model = model;
                });
                let _ = config_manager.read().save();
            }
        }
    });

    ai_page.add(&openai_group);

    // Gemini group
    let gemini_group = libadwaita::PreferencesGroup::builder()
        .title("Google Gemini")
        .build();

    let gemini_key_row = libadwaita::PasswordEntryRow::builder()
        .title("API Key")
        .text(&gemini_key)
        .build();
    gemini_group.add(&gemini_key_row);

    gemini_key_row.connect_changed(move |row| {
        let key = row.text().to_string();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.gemini.api_key = if key.is_empty() { None } else { Some(key) };
            });
            let _ = config_manager.read().save();
        }
    });

    let gemini_model_row = libadwaita::EntryRow::builder()
        .title("Model")
        .text(&gemini_model)
        .build();
    gemini_group.add(&gemini_model_row);

    gemini_model_row.connect_changed(move |row| {
        let model = row.text().to_string();
        if !model.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ai.gemini.model = model;
                });
                let _ = config_manager.read().save();
            }
        }
    });

    ai_page.add(&gemini_group);

    // Local LLM group
    let local_group = libadwaita::PreferencesGroup::builder()
        .title("Local LLM")
        .description("Use a local language model (e.g., Ollama)")
        .build();

    let local_enabled_row = libadwaita::SwitchRow::builder()
        .title("Enable Local LLM")
        .subtitle("Use local AI instead of cloud providers")
        .active(local_enabled)
        .build();
    local_group.add(&local_enabled_row);

    local_enabled_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.local.enabled = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let endpoint_row = libadwaita::EntryRow::builder()
        .title("Endpoint URL")
        .text(&local_endpoint)
        .build();
    local_group.add(&endpoint_row);

    endpoint_row.connect_changed(move |row| {
        let endpoint = row.text().to_string();
        if !endpoint.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ai.local.endpoint = endpoint;
                });
                let _ = config_manager.read().save();
            }
        }
    });

    let local_model_row = libadwaita::EntryRow::builder()
        .title("Model Name")
        .text(&local_model)
        .build();
    local_group.add(&local_model_row);

    local_model_row.connect_changed(move |row| {
        let model = row.text().to_string();
        if !model.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ai.local.model = model;
                });
                let _ = config_manager.read().save();
            }
        }
    });

    ai_page.add(&local_group);
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
    let (safe_is_enabled, preview_all, preview_dangerous, ai_explanations) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.safe_mode.enabled,
            config.safe_mode.preview_all,
            config.safe_mode.preview_dangerous_only,
            config.safe_mode.ai_explanations,
        )
    } else {
        (false, false, true, true)
    };

    let safe_enabled = libadwaita::SwitchRow::builder()
        .title("Enable Safe Mode")
        .subtitle("Preview commands before execution")
        .active(safe_is_enabled)
        .build();
    safe_group.add(&safe_enabled);

    safe_enabled.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.safe_mode.enabled = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let preview_all_row = libadwaita::SwitchRow::builder()
        .title("Preview All Commands")
        .subtitle("Show preview for every command")
        .active(preview_all)
        .build();
    safe_group.add(&preview_all_row);

    preview_all_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.safe_mode.preview_all = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let preview_danger_row = libadwaita::SwitchRow::builder()
        .title("Preview Dangerous Only")
        .subtitle("Only preview potentially destructive commands")
        .active(preview_dangerous)
        .build();
    safe_group.add(&preview_danger_row);

    preview_danger_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.safe_mode.preview_dangerous_only = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let ai_explain_row = libadwaita::SwitchRow::builder()
        .title("AI Explanations")
        .subtitle("Show AI-generated command explanations")
        .active(ai_explanations)
        .build();
    safe_group.add(&ai_explain_row);

    ai_explain_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.safe_mode.ai_explanations = active;
            });
            let _ = config_manager.read().save();
        }
    });

    safe_page.add(&safe_group);
    dialog.add(&safe_page);

    // Sessions page
    let sessions_page = libadwaita::PreferencesPage::builder()
        .title("Sessions")
        .icon_name("tab-new-symbolic")
        .build();

    let sessions_group = libadwaita::PreferencesGroup::builder()
        .title("Session Behavior")
        .description("Configure tab and session settings")
        .build();

    // Get session settings
    let (default_name, auto_rename, show_process, show_cwd, warn_close) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.sessions.default_name.clone(),
            config.sessions.auto_rename,
            config.sessions.show_process,
            config.sessions.show_cwd,
            config.sessions.warn_multiple_close,
        )
    } else {
        ("Terminal".to_string(), true, true, true, true)
    };

    let name_row = libadwaita::EntryRow::builder()
        .title("Default Tab Name")
        .text(&default_name)
        .build();
    sessions_group.add(&name_row);

    name_row.connect_changed(move |row| {
        let name = row.text().to_string();
        if !name.is_empty() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.sessions.default_name = name;
                });
                let _ = config_manager.read().save();
            }
        }
    });

    let auto_rename_row = libadwaita::SwitchRow::builder()
        .title("Auto-Rename Tabs")
        .subtitle("Update tab name based on running command")
        .active(auto_rename)
        .build();
    sessions_group.add(&auto_rename_row);

    auto_rename_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.sessions.auto_rename = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let show_process_row = libadwaita::SwitchRow::builder()
        .title("Show Process in Title")
        .subtitle("Display current process name")
        .active(show_process)
        .build();
    sessions_group.add(&show_process_row);

    show_process_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.sessions.show_process = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let show_cwd_row = libadwaita::SwitchRow::builder()
        .title("Show Directory in Title")
        .subtitle("Display current working directory")
        .active(show_cwd)
        .build();
    sessions_group.add(&show_cwd_row);

    show_cwd_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.sessions.show_cwd = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let warn_close_row = libadwaita::SwitchRow::builder()
        .title("Warn Before Multiple Close")
        .subtitle("Confirm when closing multiple tabs")
        .active(warn_close)
        .build();
    sessions_group.add(&warn_close_row);

    warn_close_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.sessions.warn_multiple_close = active;
            });
            let _ = config_manager.read().save();
        }
    });

    sessions_page.add(&sessions_group);
    dialog.add(&sessions_page);

    // Performance page
    let perf_page = libadwaita::PreferencesPage::builder()
        .title("Performance")
        .icon_name("speedometer-symbolic")
        .build();

    let perf_group = libadwaita::PreferencesGroup::builder()
        .title("Rendering")
        .description("Graphics and performance settings")
        .build();

    // Get performance settings
    let (gpu_rendering, vsync, target_fps) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.performance.gpu_rendering,
            config.performance.vsync,
            config.performance.target_fps,
        )
    } else {
        (true, true, 60)
    };

    let gpu_row = libadwaita::SwitchRow::builder()
        .title("GPU Rendering")
        .subtitle("Use GPU acceleration for drawing")
        .active(gpu_rendering)
        .build();
    perf_group.add(&gpu_row);

    gpu_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.performance.gpu_rendering = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let vsync_row = libadwaita::SwitchRow::builder()
        .title("VSync")
        .subtitle("Synchronize with display refresh rate")
        .active(vsync)
        .build();
    perf_group.add(&vsync_row);

    vsync_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.performance.vsync = active;
            });
            let _ = config_manager.read().save();
        }
    });

    let fps_adj = gtk4::Adjustment::new(target_fps as f64, 30.0, 240.0, 10.0, 30.0, 0.0);
    let fps_row = libadwaita::SpinRow::builder()
        .title("Target FPS")
        .subtitle("Maximum frame rate")
        .adjustment(&fps_adj)
        .build();
    perf_group.add(&fps_row);

    fps_row.connect_changed(move |row| {
        let fps = row.value() as u32;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.performance.target_fps = fps;
            });
            let _ = config_manager.read().save();
        }
    });

    perf_page.add(&perf_group);
    dialog.add(&perf_page);

    // Accessibility page
    let a11y_page = libadwaita::PreferencesPage::builder()
        .title("Accessibility")
        .icon_name("preferences-desktop-accessibility-symbolic")
        .build();

    let a11y_group = libadwaita::PreferencesGroup::builder()
        .title("Accessibility Features")
        .description("Improve usability for all users")
        .build();

    // Get current accessibility settings
    let (reduce_motion, high_contrast, focus_indicators, screen_reader, min_font_size, announce_notifications) =
        if let Some(config_manager) = get_config() {
            let config = config_manager.read().config();
            (
                config.accessibility.reduce_motion,
                config.accessibility.high_contrast,
                config.accessibility.focus_indicators,
                config.accessibility.screen_reader,
                config.accessibility.min_font_size,
                config.accessibility.announce_notifications,
            )
        } else {
            (false, false, true, false, 10.0, true)
        };

    // Reduce motion
    let motion_row = libadwaita::SwitchRow::builder()
        .title("Reduce Motion")
        .subtitle("Minimize animations")
        .active(reduce_motion)
        .build();
    a11y_group.add(&motion_row);

    motion_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.accessibility.reduce_motion = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // High contrast
    let contrast_row = libadwaita::SwitchRow::builder()
        .title("High Contrast")
        .subtitle("Increase color contrast for better visibility")
        .active(high_contrast)
        .build();
    a11y_group.add(&contrast_row);

    contrast_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.accessibility.high_contrast = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Focus indicators
    let focus_row = libadwaita::SwitchRow::builder()
        .title("Focus Indicators")
        .subtitle("Show keyboard focus outlines")
        .active(focus_indicators)
        .build();
    a11y_group.add(&focus_row);

    focus_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.accessibility.focus_indicators = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Screen reader support
    let screen_reader_row = libadwaita::SwitchRow::builder()
        .title("Screen Reader Support")
        .subtitle("Enhance compatibility with screen readers")
        .active(screen_reader)
        .build();
    a11y_group.add(&screen_reader_row);

    screen_reader_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.accessibility.screen_reader = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Minimum font size
    let min_font_adj = gtk4::Adjustment::new(min_font_size as f64, 8.0, 24.0, 1.0, 2.0, 0.0);
    let min_font_row = libadwaita::SpinRow::builder()
        .title("Minimum Font Size")
        .subtitle("Smallest allowed font size")
        .adjustment(&min_font_adj)
        .build();
    a11y_group.add(&min_font_row);

    min_font_row.connect_changed(move |row| {
        let size = row.value() as f32;
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.accessibility.min_font_size = size;
            });
            let _ = config_manager.read().save();
        }
    });

    // Announce notifications
    let announce_row = libadwaita::SwitchRow::builder()
        .title("Announce Notifications")
        .subtitle("Speak notifications to screen reader")
        .active(announce_notifications)
        .build();
    a11y_group.add(&announce_row);

    announce_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.accessibility.announce_notifications = active;
            });
            let _ = config_manager.read().save();
        }
    });

    a11y_page.add(&a11y_group);
    dialog.add(&a11y_page);

    // Keybindings page
    let keybindings_page = libadwaita::PreferencesPage::builder()
        .title("Keybindings")
        .icon_name("input-keyboard-symbolic")
        .build();

    let keybindings_group = libadwaita::PreferencesGroup::builder()
        .title("Keyboard Shortcuts")
        .description("View and customize keyboard shortcuts")
        .build();

    // Show current shortcuts as informational rows
    let shortcuts_info = [
        ("New Tab", "Ctrl+T", "Create a new terminal tab"),
        ("Close Tab", "Ctrl+W", "Close the current tab"),
        ("Next Tab", "Ctrl+Tab", "Switch to next tab"),
        ("Previous Tab", "Ctrl+Shift+Tab", "Switch to previous tab"),
        ("Quick Switcher", "Ctrl+K", "Open the quick switcher"),
        ("Toggle AI Panel", "Ctrl+Shift+A", "Show/hide AI assistant"),
        ("New Document", "Ctrl+O", "Open a new document tab"),
        ("Open File", "Ctrl+Shift+O", "Open a file in a new tab"),
        ("Quit", "Ctrl+Q", "Close the application"),
    ];

    for (name, shortcut, description) in shortcuts_info {
        let row = libadwaita::ActionRow::builder()
            .title(name)
            .subtitle(description)
            .build();

        // Add shortcut label as suffix
        let shortcut_label = gtk4::Label::new(Some(shortcut));
        shortcut_label.add_css_class("dim-label");
        shortcut_label.add_css_class("monospace");
        row.add_suffix(&shortcut_label);

        keybindings_group.add(&row);
    }

    // Terminal shortcuts
    let terminal_keybindings_group = libadwaita::PreferencesGroup::builder()
        .title("Terminal Shortcuts")
        .build();

    let terminal_shortcuts = [
        ("Copy", "Ctrl+Shift+C", "Copy selection to clipboard"),
        ("Paste", "Ctrl+Shift+V", "Paste from clipboard"),
        ("Select All", "Ctrl+Shift+A", "Select all terminal content"),
        ("Zoom In", "Ctrl++", "Increase font size"),
        ("Zoom Out", "Ctrl+-", "Decrease font size"),
        ("Reset Zoom", "Ctrl+0", "Reset to default font size"),
    ];

    for (name, shortcut, description) in terminal_shortcuts {
        let row = libadwaita::ActionRow::builder()
            .title(name)
            .subtitle(description)
            .build();

        let shortcut_label = gtk4::Label::new(Some(shortcut));
        shortcut_label.add_css_class("dim-label");
        shortcut_label.add_css_class("monospace");
        row.add_suffix(&shortcut_label);

        terminal_keybindings_group.add(&row);
    }

    // Custom keybindings note
    let custom_note = libadwaita::PreferencesGroup::builder()
        .title("Custom Keybindings")
        .description("Custom keybindings can be set in ~/.config/corgiterm/config.toml under [keybindings]")
        .build();

    keybindings_page.add(&keybindings_group);
    keybindings_page.add(&terminal_keybindings_group);
    keybindings_page.add(&custom_note);
    dialog.add(&keybindings_page);

    // Plugins page
    let plugins_page = libadwaita::PreferencesPage::builder()
        .title("Plugins")
        .icon_name("application-x-addon-symbolic")
        .build();

    let plugins_group = libadwaita::PreferencesGroup::builder()
        .title("Installed Plugins")
        .description("Manage CorgiTerm extensions")
        .build();

    // Get plugin list
    if let Some(pm) = crate::app::plugin_manager() {
        let plugin_mgr = pm.read();
        let plugins = plugin_mgr.plugins();

        if plugins.is_empty() {
            let empty_row = libadwaita::ActionRow::builder()
                .title("No plugins installed")
                .subtitle("Put plugins in ~/.config/corgiterm/plugins/")
                .build();
            plugins_group.add(&empty_row);
        } else {
            for plugin in plugins {
                let row = libadwaita::SwitchRow::builder()
                    .title(&plugin.manifest.name)
                    .subtitle(plugin.manifest.description.as_deref().unwrap_or("No description"))
                    .active(plugin.enabled)
                    .build();

                // Add version as suffix
                let version_label = gtk4::Label::new(Some(&format!("v{}", plugin.manifest.version)));
                version_label.add_css_class("dim-label");
                row.add_suffix(&version_label);

                plugins_group.add(&row);
            }
        }
    } else {
        let error_row = libadwaita::ActionRow::builder()
            .title("Plugin system not initialized")
            .build();
        plugins_group.add(&error_row);
    }

    // Plugin development info
    let dev_group = libadwaita::PreferencesGroup::builder()
        .title("Plugin Development")
        .description("Create your own plugins")
        .build();

    let dev_info = libadwaita::ActionRow::builder()
        .title("Plugin Types")
        .subtitle("WASM (sandboxed) or Lua (lightweight)")
        .build();
    dev_group.add(&dev_info);

    let docs_row = libadwaita::ActionRow::builder()
        .title("Documentation")
        .subtitle("https://corgiterm.dev/plugins")
        .activatable(true)
        .build();

    // Open documentation link when clicked
    docs_row.connect_activated(|row| {
        if let Some(root) = row.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                let launcher = gtk4::UriLauncher::new("https://corgiterm.dev/plugins");
                launcher.launch(Some(window), None::<&gtk4::gio::Cancellable>, |result| {
                    if let Err(e) = result {
                        tracing::warn!("Failed to open docs: {}", e);
                    }
                });
            }
        }
    });
    dev_group.add(&docs_row);

    plugins_page.add(&plugins_group);
    plugins_page.add(&dev_group);
    dialog.add(&plugins_page);

    // Advanced page
    let advanced_page = libadwaita::PreferencesPage::builder()
        .title("Advanced")
        .icon_name("applications-utilities-symbolic")
        .build();

    let advanced_group = libadwaita::PreferencesGroup::builder()
        .title("Developer Options")
        .description("Advanced settings for developers")
        .build();

    // Get advanced settings
    let (debug_mode, log_level, experimental) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.advanced.debug,
            config.advanced.log_level.clone(),
            config.advanced.experimental,
        )
    } else {
        (false, "info".to_string(), false)
    };

    let debug_row = libadwaita::SwitchRow::builder()
        .title("Debug Mode")
        .subtitle("Enable debug logging and features")
        .active(debug_mode)
        .build();
    advanced_group.add(&debug_row);

    debug_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.advanced.debug = active;
            });
            let _ = config_manager.read().save();
        }
    });

    // Log level
    let log_row = libadwaita::ComboRow::builder()
        .title("Log Level")
        .subtitle("Verbosity of logging output")
        .build();
    let log_levels = ["error", "warn", "info", "debug", "trace"];
    log_row.set_model(Some(&gtk4::StringList::new(&log_levels)));
    if let Some(pos) = log_levels.iter().position(|&l| l == log_level) {
        log_row.set_selected(pos as u32);
    }
    advanced_group.add(&log_row);

    log_row.connect_selected_notify(move |row| {
        let selected = row.selected() as usize;
        if selected < log_levels.len() {
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.advanced.log_level = log_levels[selected].to_string();
                });
                let _ = config_manager.read().save();
            }
        }
    });

    // Experimental features
    let experimental_row = libadwaita::SwitchRow::builder()
        .title("Experimental Features")
        .subtitle("Enable unstable features (may cause issues)")
        .active(experimental)
        .build();
    advanced_group.add(&experimental_row);

    experimental_row.connect_active_notify(move |row| {
        let active = row.is_active();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.advanced.experimental = active;
            });
            let _ = config_manager.read().save();
        }
    });

    advanced_page.add(&advanced_group);
    dialog.add(&advanced_page);

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
        ("Toggle AI Panel", "<Ctrl><Shift>a"),
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

/// Show ASCII Art Generator dialog with callback for inserting art into terminal
pub fn show_ascii_art_dialog<W, F>(parent: &W, on_insert: F)
where
    W: IsA<Window> + IsA<gtk4::Widget>,
    F: Fn(&str) + 'static,
{
    let dialog = crate::ascii_art_dialog::AsciiArtDialog::new(parent);
    dialog.set_insert_callback(on_insert);
    dialog.show();
}

/// Show Safe Mode preview dialog for a command
/// Returns true if user confirms execution, false if cancelled
pub fn show_safe_mode_preview<F>(
    parent: &impl IsA<gtk4::Widget>,
    preview: &corgiterm_core::CommandPreview,
    on_execute: F,
) where
    F: Fn() + 'static,
{
    use corgiterm_core::RiskLevel;

    let dialog = libadwaita::Dialog::builder()
        .title("🐕 Safe Mode Preview")
        .build();
    dialog.set_follows_content_size(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_width_request(500);

    // Command section
    let cmd_label = gtk4::Label::new(Some("Command:"));
    cmd_label.set_xalign(0.0);
    cmd_label.add_css_class("dim-label");
    main_box.append(&cmd_label);

    let cmd_text = gtk4::Label::new(Some(&preview.command));
    cmd_text.set_xalign(0.0);
    cmd_text.add_css_class("monospace");
    cmd_text.set_selectable(true);
    cmd_text.set_margin_bottom(12);
    main_box.append(&cmd_text);

    // Risk level banner
    let risk_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    risk_box.set_margin_bottom(12);

    let (risk_icon, risk_text, risk_class) = match preview.risk {
        RiskLevel::Safe => ("✅", "SAFE - This command is safe to run", "success"),
        RiskLevel::Caution => ("⚠️", "CAUTION - This will make changes", "warning"),
        RiskLevel::Danger => ("🚨", "DANGER - This could cause data loss", "error"),
        RiskLevel::Unknown => ("❓", "UNKNOWN - Unable to assess risk", "dim-label"),
    };

    let risk_emoji = gtk4::Label::new(Some(risk_icon));
    risk_emoji.set_margin_end(4);
    risk_box.append(&risk_emoji);

    let risk_label = gtk4::Label::new(Some(risk_text));
    risk_label.add_css_class(risk_class);
    risk_label.add_css_class("heading");
    risk_box.append(&risk_label);

    main_box.append(&risk_box);

    // Explanation section
    if !preview.explanation.is_empty() {
        let exp_header = gtk4::Label::new(Some("What it does:"));
        exp_header.set_xalign(0.0);
        exp_header.add_css_class("dim-label");
        main_box.append(&exp_header);

        for exp in &preview.explanation {
            let bullet = gtk4::Label::new(Some(&format!("• {}", exp)));
            bullet.set_xalign(0.0);
            bullet.set_wrap(true);
            main_box.append(&bullet);
        }
    }

    // Affected files
    if let Some(count) = preview.affected_count {
        let size_str = preview.affected_size.map(|s| {
            if s > 1_000_000_000 { format!(" ({:.1} GB)", s as f64 / 1_000_000_000.0) }
            else if s > 1_000_000 { format!(" ({:.1} MB)", s as f64 / 1_000_000.0) }
            else if s > 1_000 { format!(" ({:.1} KB)", s as f64 / 1_000.0) }
            else { format!(" ({} bytes)", s) }
        }).unwrap_or_default();

        let affected = gtk4::Label::new(Some(&format!("• Will affect {} file(s){}", count, size_str)));
        affected.set_xalign(0.0);
        main_box.append(&affected);
    }

    // Network access warning
    if preview.network_access {
        let net_label = gtk4::Label::new(Some("• Requires network access"));
        net_label.set_xalign(0.0);
        main_box.append(&net_label);
    }

    // Sudo warning
    if preview.needs_sudo {
        let sudo_label = gtk4::Label::new(Some("• Requires administrator privileges (sudo)"));
        sudo_label.set_xalign(0.0);
        sudo_label.add_css_class("warning");
        main_box.append(&sudo_label);
    }

    // Undo hint
    if let Some(ref undo) = preview.undo_hint {
        let undo_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        undo_box.set_margin_top(8);
        let undo_prefix = gtk4::Label::new(Some("To undo: "));
        undo_prefix.add_css_class("dim-label");
        undo_box.append(&undo_prefix);
        let undo_cmd = gtk4::Label::new(Some(undo));
        undo_cmd.add_css_class("monospace");
        undo_box.append(&undo_cmd);
        main_box.append(&undo_box);
    }

    // Safer alternatives
    if !preview.alternatives.is_empty() {
        let alt_header = gtk4::Label::new(Some("Safer alternatives:"));
        alt_header.set_xalign(0.0);
        alt_header.add_css_class("dim-label");
        alt_header.set_margin_top(12);
        main_box.append(&alt_header);

        for alt in &preview.alternatives {
            let alt_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            let alt_cmd = gtk4::Label::new(Some(&alt.command));
            alt_cmd.add_css_class("monospace");
            alt_box.append(&alt_cmd);
            let alt_desc = gtk4::Label::new(Some(&format!("- {}", alt.description)));
            alt_desc.add_css_class("dim-label");
            alt_box.append(&alt_desc);
            main_box.append(&alt_box);
        }
    }

    // Buttons
    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    button_box.set_margin_top(20);
    button_box.set_halign(gtk4::Align::End);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("pill");
    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });
    button_box.append(&cancel_btn);

    let execute_btn = gtk4::Button::with_label("Execute");
    execute_btn.add_css_class("pill");
    execute_btn.add_css_class(if preview.risk == RiskLevel::Danger {
        "destructive-action"
    } else {
        "suggested-action"
    });
    let dialog_for_exec = dialog.clone();
    execute_btn.connect_clicked(move |_| {
        on_execute();
        dialog_for_exec.close();
    });
    button_box.append(&execute_btn);

    main_box.append(&button_box);

    dialog.set_child(Some(&main_box));
    dialog.present(Some(parent));
}
