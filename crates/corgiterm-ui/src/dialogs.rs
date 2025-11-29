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
    let (scrollback, copy_on_select, scroll_on_output, scroll_on_keystroke) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.terminal.scrollback_lines,
            config.terminal.copy_on_select,
            config.terminal.scroll_on_output,
            config.terminal.scroll_on_keystroke,
        )
    } else {
        (10000, false, false, true)
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
    let (default_provider, claude_key, openai_key, gemini_key, local_enabled, local_endpoint) =
        if let Some(config_manager) = get_config() {
            let config = config_manager.read().config();
            (
                config.ai.default_provider.clone(),
                config.ai.claude.api_key.clone().unwrap_or_default(),
                config.ai.openai.api_key.clone().unwrap_or_default(),
                config.ai.gemini.api_key.clone().unwrap_or_default(),
                config.ai.local.enabled,
                config.ai.local.endpoint.clone(),
            )
        } else {
            ("claude".to_string(), String::new(), String::new(), String::new(), false, "http://localhost:11434".to_string())
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

    // Claude API key
    let claude_row = libadwaita::PasswordEntryRow::builder()
        .title("Claude API Key")
        .text(&claude_key)
        .build();
    provider_group.add(&claude_row);

    claude_row.connect_changed(move |row| {
        let key = row.text().to_string();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.claude.api_key = if key.is_empty() { None } else { Some(key) };
            });
            let _ = config_manager.read().save();
        }
    });

    // OpenAI API key
    let openai_row = libadwaita::PasswordEntryRow::builder()
        .title("OpenAI API Key")
        .text(&openai_key)
        .build();
    provider_group.add(&openai_row);

    openai_row.connect_changed(move |row| {
        let key = row.text().to_string();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.openai.api_key = if key.is_empty() { None } else { Some(key) };
            });
            let _ = config_manager.read().save();
        }
    });

    // Gemini API key
    let gemini_row = libadwaita::PasswordEntryRow::builder()
        .title("Gemini API Key")
        .text(&gemini_key)
        .build();
    provider_group.add(&gemini_row);

    gemini_row.connect_changed(move |row| {
        let key = row.text().to_string();
        if let Some(config_manager) = get_config() {
            config_manager.read().update(|config| {
                config.ai.gemini.api_key = if key.is_empty() { None } else { Some(key) };
            });
            let _ = config_manager.read().save();
        }
    });

    ai_page.add(&provider_group);

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
        .title("Ollama Endpoint")
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
    let (reduce_motion, high_contrast, focus_indicators) = if let Some(config_manager) = get_config() {
        let config = config_manager.read().config();
        (
            config.accessibility.reduce_motion,
            config.accessibility.high_contrast,
            config.accessibility.focus_indicators,
        )
    } else {
        (false, false, true)
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

    a11y_page.add(&a11y_group);
    dialog.add(&a11y_page);

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
