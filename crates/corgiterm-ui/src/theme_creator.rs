//! Visual Theme Creator with live preview
//!
//! A comprehensive theme editor that allows users to create custom color schemes
//! with real-time preview of how the theme looks in practice.

use corgiterm_config::themes::Theme;
use gtk4::prelude::*;
use gtk4::{
    Box, ColorDialog, ColorDialogButton, Label, Orientation, ScrolledWindow, TextView, Window,
};
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Color utility to convert between formats
fn hex_to_rgba(hex: &str) -> gtk4::gdk::RGBA {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
    gtk4::gdk::RGBA::new(r, g, b, 1.0)
}

fn rgba_to_hex(rgba: &gtk4::gdk::RGBA) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (rgba.red() * 255.0) as u8,
        (rgba.green() * 255.0) as u8,
        (rgba.blue() * 255.0) as u8
    )
}

/// Calculate relative luminance for contrast checking
fn relative_luminance(rgba: &gtk4::gdk::RGBA) -> f32 {
    let r = if rgba.red() <= 0.03928 {
        rgba.red() / 12.92
    } else {
        ((rgba.red() + 0.055) / 1.055).powf(2.4)
    };

    let g = if rgba.green() <= 0.03928 {
        rgba.green() / 12.92
    } else {
        ((rgba.green() + 0.055) / 1.055).powf(2.4)
    };

    let b = if rgba.blue() <= 0.03928 {
        rgba.blue() / 12.92
    } else {
        ((rgba.blue() + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate contrast ratio between two colors
fn contrast_ratio(c1: &gtk4::gdk::RGBA, c2: &gtk4::gdk::RGBA) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);

    if l1 > l2 {
        (l1 + 0.05) / (l2 + 0.05)
    } else {
        (l2 + 0.05) / (l1 + 0.05)
    }
}

/// Theme being edited
#[derive(Clone)]
struct ThemeEditor {
    theme: Rc<RefCell<Theme>>,
}

impl ThemeEditor {
    fn new() -> Self {
        Self {
            theme: Rc::new(RefCell::new(Theme::corgi_dark())),
        }
    }

    #[allow(dead_code)]
    fn from_theme(theme: Theme) -> Self {
        Self {
            theme: Rc::new(RefCell::new(theme)),
        }
    }

    fn get_theme(&self) -> Theme {
        self.theme.borrow().clone()
    }

    fn set_name(&self, name: String) {
        self.theme.borrow_mut().name = name;
    }

    fn set_author(&self, author: String) {
        self.theme.borrow_mut().author = Some(author);
    }

    fn set_description(&self, desc: String) {
        self.theme.borrow_mut().description = Some(desc);
    }

    fn set_is_dark(&self, is_dark: bool) {
        self.theme.borrow_mut().is_dark = is_dark;
    }

    // Color setters
    fn set_foreground(&self, color: String) {
        self.theme.borrow_mut().colors.foreground = color;
    }

    fn set_background(&self, color: String) {
        self.theme.borrow_mut().colors.background = color;
    }

    fn set_cursor(&self, color: String) {
        self.theme.borrow_mut().cursor.cursor = color;
    }

    fn set_cursor_text(&self, color: String) {
        self.theme.borrow_mut().cursor.cursor_text = color;
    }

    fn set_selection_bg(&self, color: String) {
        self.theme.borrow_mut().selection.background = color;
    }

    // ANSI color setters
    fn set_ansi_color(&self, index: usize, color: String) {
        let mut theme = self.theme.borrow_mut();
        match index {
            0 => theme.colors.black = color,
            1 => theme.colors.red = color,
            2 => theme.colors.green = color,
            3 => theme.colors.yellow = color,
            4 => theme.colors.blue = color,
            5 => theme.colors.magenta = color,
            6 => theme.colors.cyan = color,
            7 => theme.colors.white = color,
            8 => theme.colors.bright_black = color,
            9 => theme.colors.bright_red = color,
            10 => theme.colors.bright_green = color,
            11 => theme.colors.bright_yellow = color,
            12 => theme.colors.bright_blue = color,
            13 => theme.colors.bright_magenta = color,
            14 => theme.colors.bright_cyan = color,
            15 => theme.colors.bright_white = color,
            _ => {}
        }
    }

    // UI color setters
    fn set_accent(&self, color: String) {
        self.theme.borrow_mut().ui.accent = color;
    }

    fn set_success(&self, color: String) {
        self.theme.borrow_mut().ui.success = color;
    }

    fn set_warning(&self, color: String) {
        self.theme.borrow_mut().ui.warning = color;
    }

    fn set_error(&self, color: String) {
        self.theme.borrow_mut().ui.error = color;
    }
}

/// Create a color picker row
fn create_color_row(
    label: &str,
    initial_color: &str,
    editor: &ThemeEditor,
    setter: impl Fn(&ThemeEditor, String) + 'static,
    preview_buffer: &gtk4::TextBuffer,
    css_provider: &Rc<RefCell<gtk4::CssProvider>>,
    contrast_label: &Rc<RefCell<Label>>,
) -> libadwaita::ActionRow {
    let row = libadwaita::ActionRow::builder().title(label).build();

    // Use modern ColorDialogButton instead of deprecated ColorButton
    let color_dialog = ColorDialog::builder().with_alpha(false).build();
    let color_button = ColorDialogButton::builder()
        .dialog(&color_dialog)
        .rgba(&hex_to_rgba(initial_color))
        .valign(gtk4::Align::Center)
        .build();

    // Add hex label
    let hex_label = Label::new(Some(initial_color));
    hex_label.add_css_class("monospace");
    hex_label.add_css_class("dim-label");
    hex_label.set_margin_end(12);

    let editor_clone = editor.clone();
    let preview_buffer_clone = preview_buffer.clone();
    let hex_label_clone = hex_label.clone();
    let css_provider_clone = css_provider.clone();
    let contrast_label_clone = contrast_label.clone();

    // ColorDialogButton uses notify::rgba signal
    color_button.connect_rgba_notify(move |button| {
        let rgba = button.rgba();
        let hex = rgba_to_hex(&rgba);
        hex_label_clone.set_text(&hex);
        setter(&editor_clone, hex);
        update_preview(&preview_buffer_clone, &editor_clone);

        // Update CSS (use .theme-preview class for the preview widget)
        let theme = editor_clone.get_theme();
        let css = format!(
            ".theme-preview {{
                background-color: {};
                color: {};
                font-family: 'Source Code Pro', monospace;
                font-size: 11pt;
                padding: 8px;
            }}",
            &theme.colors.background, &theme.colors.foreground,
        );
        css_provider_clone.borrow().load_from_string(&css);

        // Update contrast ratio
        let bg_rgba = hex_to_rgba(&theme.colors.background);
        let fg_rgba = hex_to_rgba(&theme.colors.foreground);
        let ratio = contrast_ratio(&bg_rgba, &fg_rgba);
        let status = if ratio >= 7.0 {
            "✅ AAA (Excellent)"
        } else if ratio >= 4.5 {
            "✅ AA (Good)"
        } else {
            "⚠️ Below recommended (consider adjusting)"
        };
        contrast_label_clone
            .borrow()
            .set_text(&format!("Contrast Ratio: {:.2}:1 - {}", ratio, status));
    });

    row.add_suffix(&hex_label);
    row.add_suffix(&color_button);
    row
}

/// Update the live preview with current theme
fn update_preview(buffer: &gtk4::TextBuffer, _editor: &ThemeEditor) {
    // Note: _editor could be used for color-aware text styling in future
    // Create sample terminal content
    let preview_text = r#"user@corgiterm:~/projects/corgiterm$ ls -la
total 128
drwxr-xr-x  12 user group  4096 Nov 29 10:30 ./
drwxr-xr-x   8 user group  4096 Nov 29 09:15 ../
-rw-r--r--   1 user group  2048 Nov 29 10:28 config.toml
drwxr-xr-x   3 user group  4096 Nov 29 10:25 src/

user@corgiterm:~/projects/corgiterm$ cargo build
   Compiling corgiterm v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s

user@corgiterm:~/projects/corgiterm$ echo "Testing colors!"
Black  Red  Green  Yellow  Blue  Magenta  Cyan  White

Bright colors:
Black  Red  Green  Yellow  Blue  Magenta  Cyan  White

user@corgiterm:~/projects/corgiterm$ █"#;

    buffer.set_text(preview_text);
}

/// Create the live preview panel
fn create_preview_panel(
    editor: &ThemeEditor,
) -> (
    Box,
    gtk4::TextBuffer,
    Rc<RefCell<gtk4::CssProvider>>,
    Rc<RefCell<Label>>,
) {
    let panel = Box::new(Orientation::Vertical, 12);
    panel.set_margin_start(12);
    panel.set_margin_end(12);
    panel.set_margin_top(12);
    panel.set_margin_bottom(12);

    // Header
    let header = Label::new(Some("Live Preview"));
    header.add_css_class("title-2");
    header.set_halign(gtk4::Align::Start);
    panel.append(&header);

    // Preview area with terminal styling
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .min_content_height(500)
        .build();

    let text_view = TextView::builder()
        .editable(false)
        .monospace(true)
        .left_margin(12)
        .right_margin(12)
        .top_margin(12)
        .bottom_margin(12)
        .build();

    let buffer = text_view.buffer();

    // Set initial preview
    update_preview(&buffer, editor);

    // Apply theme colors to the preview widget
    let theme = editor.get_theme();

    // Create CSS for preview styling using widget-specific class
    let css_provider = gtk4::CssProvider::new();
    let css = format!(
        ".theme-preview {{
            background-color: {};
            color: {};
            font-family: 'Source Code Pro', monospace;
            font-size: 11pt;
            padding: 8px;
        }}",
        &theme.colors.background, &theme.colors.foreground,
    );
    css_provider.load_from_string(&css);

    // Add CSS class and register provider with display (modern GTK4 4.10+ approach)
    text_view.add_css_class("theme-preview");
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    scrolled.set_child(Some(&text_view));
    panel.append(&scrolled);

    // Contrast check
    let contrast_label = Label::new(None);
    contrast_label.set_halign(gtk4::Align::Start);
    contrast_label.add_css_class("dim-label");

    let bg_rgba = hex_to_rgba(&theme.colors.background);
    let fg_rgba = hex_to_rgba(&theme.colors.foreground);
    let ratio = contrast_ratio(&bg_rgba, &fg_rgba);
    let status = if ratio >= 7.0 {
        "✅ AAA (Excellent)"
    } else if ratio >= 4.5 {
        "✅ AA (Good)"
    } else {
        "⚠️ Below recommended (consider adjusting)"
    };

    contrast_label.set_text(&format!("Contrast Ratio: {:.2}:1 - {}", ratio, status));
    panel.append(&contrast_label);

    (
        panel,
        buffer,
        Rc::new(RefCell::new(css_provider)),
        Rc::new(RefCell::new(contrast_label)),
    )
}

/// Show the Theme Creator dialog
pub fn show_theme_creator<W: IsA<Window> + IsA<gtk4::Widget>>(parent: &W) {
    let editor = ThemeEditor::new();

    let dialog = libadwaita::Dialog::builder()
        .title("Theme Creator")
        .content_width(1200)
        .content_height(700)
        .follows_content_size(false)
        .build();

    // Main horizontal split
    let main_box = Box::new(Orientation::Horizontal, 0);

    // Left panel - Theme editor
    let left_panel = Box::new(Orientation::Vertical, 0);
    left_panel.set_width_request(550);

    let scrolled_left = ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build();

    let editor_box = Box::new(Orientation::Vertical, 0);

    // Theme metadata
    let meta_page = libadwaita::PreferencesGroup::builder()
        .title("Theme Information")
        .build();

    let name_row = libadwaita::EntryRow::builder()
        .title("Theme Name")
        .text(&editor.get_theme().name)
        .build();

    let editor_clone = editor.clone();
    name_row.connect_changed(move |row| {
        editor_clone.set_name(row.text().to_string());
    });
    meta_page.add(&name_row);

    let author_row = libadwaita::EntryRow::builder()
        .title("Author")
        .text(editor.get_theme().author.as_deref().unwrap_or(""))
        .build();

    let editor_clone = editor.clone();
    author_row.connect_changed(move |row| {
        editor_clone.set_author(row.text().to_string());
    });
    meta_page.add(&author_row);

    let desc_row = libadwaita::EntryRow::builder()
        .title("Description")
        .text(editor.get_theme().description.as_deref().unwrap_or(""))
        .build();

    let editor_clone = editor.clone();
    desc_row.connect_changed(move |row| {
        editor_clone.set_description(row.text().to_string());
    });
    meta_page.add(&desc_row);

    let dark_row = libadwaita::SwitchRow::builder()
        .title("Dark Theme")
        .subtitle("Is this a dark theme?")
        .active(editor.get_theme().is_dark)
        .build();

    let editor_clone = editor.clone();
    dark_row.connect_active_notify(move |row| {
        editor_clone.set_is_dark(row.is_active());
    });
    meta_page.add(&dark_row);

    editor_box.append(&meta_page);

    // Create preview panel early so we can pass its components to color rows
    let (right_panel, preview_buffer, css_provider, contrast_label_rc) =
        create_preview_panel(&editor);

    // Background & Foreground colors
    let bg_fg_group = libadwaita::PreferencesGroup::builder()
        .title("Background & Foreground")
        .build();

    let theme = editor.get_theme();
    bg_fg_group.add(&create_color_row(
        "Background",
        &theme.colors.background,
        &editor,
        |e, c| e.set_background(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    bg_fg_group.add(&create_color_row(
        "Foreground",
        &theme.colors.foreground,
        &editor,
        |e, c| e.set_foreground(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    editor_box.append(&bg_fg_group);

    // Normal ANSI colors
    let normal_group = libadwaita::PreferencesGroup::builder()
        .title("Normal Colors (ANSI 0-7)")
        .build();

    let normal_colors = [
        ("Black", &theme.colors.black, 0),
        ("Red", &theme.colors.red, 1),
        ("Green", &theme.colors.green, 2),
        ("Yellow", &theme.colors.yellow, 3),
        ("Blue", &theme.colors.blue, 4),
        ("Magenta", &theme.colors.magenta, 5),
        ("Cyan", &theme.colors.cyan, 6),
        ("White", &theme.colors.white, 7),
    ];

    for (label, color, idx) in normal_colors {
        normal_group.add(&create_color_row(
            label,
            color,
            &editor,
            move |e, c| e.set_ansi_color(idx, c),
            &preview_buffer,
            &css_provider,
            &contrast_label_rc,
        ));
    }

    editor_box.append(&normal_group);

    // Bright ANSI colors
    let bright_group = libadwaita::PreferencesGroup::builder()
        .title("Bright Colors (ANSI 8-15)")
        .build();

    let bright_colors = [
        ("Bright Black", &theme.colors.bright_black, 8),
        ("Bright Red", &theme.colors.bright_red, 9),
        ("Bright Green", &theme.colors.bright_green, 10),
        ("Bright Yellow", &theme.colors.bright_yellow, 11),
        ("Bright Blue", &theme.colors.bright_blue, 12),
        ("Bright Magenta", &theme.colors.bright_magenta, 13),
        ("Bright Cyan", &theme.colors.bright_cyan, 14),
        ("Bright White", &theme.colors.bright_white, 15),
    ];

    for (label, color, idx) in bright_colors {
        bright_group.add(&create_color_row(
            label,
            color,
            &editor,
            move |e, c| e.set_ansi_color(idx, c),
            &preview_buffer,
            &css_provider,
            &contrast_label_rc,
        ));
    }

    editor_box.append(&bright_group);

    // Cursor colors
    let cursor_group = libadwaita::PreferencesGroup::builder()
        .title("Cursor")
        .build();

    cursor_group.add(&create_color_row(
        "Cursor Color",
        &theme.cursor.cursor,
        &editor,
        |e, c| e.set_cursor(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    cursor_group.add(&create_color_row(
        "Cursor Text",
        &theme.cursor.cursor_text,
        &editor,
        |e, c| e.set_cursor_text(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    editor_box.append(&cursor_group);

    // Selection colors
    let selection_group = libadwaita::PreferencesGroup::builder()
        .title("Selection")
        .build();

    selection_group.add(&create_color_row(
        "Selection Background",
        &theme.selection.background,
        &editor,
        |e, c| e.set_selection_bg(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    editor_box.append(&selection_group);

    // UI colors
    let ui_group = libadwaita::PreferencesGroup::builder()
        .title("UI Colors")
        .build();

    ui_group.add(&create_color_row(
        "Accent",
        &theme.ui.accent,
        &editor,
        |e, c| e.set_accent(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    ui_group.add(&create_color_row(
        "Success",
        &theme.ui.success,
        &editor,
        |e, c| e.set_success(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    ui_group.add(&create_color_row(
        "Warning",
        &theme.ui.warning,
        &editor,
        |e, c| e.set_warning(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    ui_group.add(&create_color_row(
        "Error",
        &theme.ui.error,
        &editor,
        |e, c| e.set_error(c),
        &preview_buffer,
        &css_provider,
        &contrast_label_rc,
    ));

    editor_box.append(&ui_group);

    scrolled_left.set_child(Some(&editor_box));
    left_panel.append(&scrolled_left);

    // Toolbar with actions
    let toolbar = gtk4::Box::new(Orientation::Horizontal, 6);
    toolbar.set_margin_start(12);
    toolbar.set_margin_end(12);
    toolbar.set_margin_top(6);
    toolbar.set_margin_bottom(6);
    toolbar.add_css_class("toolbar");

    // Load preset button
    let preset_btn = gtk4::MenuButton::builder().label("Load Preset").build();

    let preset_menu = gtk4::gio::Menu::new();
    preset_menu.append(Some("Corgi Dark"), Some("theme.preset.corgi-dark"));
    preset_menu.append(Some("Corgi Light"), Some("theme.preset.corgi-light"));
    preset_menu.append(Some("Corgi Sunset"), Some("theme.preset.corgi-sunset"));
    preset_menu.append(Some("Pembroke"), Some("theme.preset.pembroke"));
    preset_btn.set_menu_model(Some(&preset_menu));

    toolbar.append(&preset_btn);

    // Export button
    let export_btn = gtk4::Button::with_label("Export");
    let editor_clone = editor.clone();
    let parent_clone = parent.clone();
    export_btn.connect_clicked(move |_| {
        export_theme(&parent_clone, &editor_clone);
    });
    toolbar.append(&export_btn);

    // Import button
    let import_btn = gtk4::Button::with_label("Import");
    let editor_clone = editor.clone();
    let parent_clone = parent.clone();
    import_btn.connect_clicked(move |_| {
        import_theme(&parent_clone, &editor_clone);
    });
    toolbar.append(&import_btn);

    // Spacer
    let spacer = gtk4::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    // Save button
    let save_btn = gtk4::Button::with_label("Save Theme");
    save_btn.add_css_class("suggested-action");
    let editor_clone = editor.clone();
    let dialog_clone = dialog.clone();
    save_btn.connect_clicked(move |_| {
        save_theme(&editor_clone);
        dialog_clone.close();
    });
    toolbar.append(&save_btn);

    left_panel.append(&toolbar);

    // right_panel was created earlier with create_preview_panel
    right_panel.set_hexpand(true);

    main_box.append(&left_panel);

    let separator = gtk4::Separator::new(Orientation::Vertical);
    main_box.append(&separator);

    main_box.append(&right_panel);

    dialog.set_child(Some(&main_box));
    dialog.present(Some(parent));
}

/// Export theme to TOML file
fn export_theme<W: IsA<gtk4::Widget>>(parent: &W, editor: &ThemeEditor) {
    let theme = editor.get_theme();

    let file_dialog = gtk4::FileDialog::builder()
        .title("Export Theme")
        .initial_name(format!(
            "{}.toml",
            theme.name.to_lowercase().replace(' ', "-")
        ))
        .build();

    let editor_clone = editor.clone();
    file_dialog.save(
        parent
            .root()
            .and_then(|r| r.downcast::<Window>().ok())
            .as_ref(),
        None::<&gtk4::gio::Cancellable>,
        move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let theme = editor_clone.get_theme();
                    if let Ok(toml_string) = toml::to_string_pretty(&theme) {
                        if let Err(e) = std::fs::write(&path, toml_string) {
                            tracing::error!("Failed to export theme: {}", e);
                        } else {
                            tracing::info!("Theme exported to: {:?}", path);
                        }
                    }
                }
            }
        },
    );
}

/// Import theme from TOML file
fn import_theme<W: IsA<gtk4::Widget>>(parent: &W, editor: &ThemeEditor) {
    let file_dialog = gtk4::FileDialog::builder().title("Import Theme").build();

    let editor_clone = editor.clone();
    file_dialog.open(
        parent
            .root()
            .and_then(|r| r.downcast::<Window>().ok())
            .as_ref(),
        None::<&gtk4::gio::Cancellable>,
        move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(theme) = toml::from_str::<Theme>(&content) {
                            *editor_clone.theme.borrow_mut() = theme;
                            tracing::info!("Theme imported from: {:?}", path);
                        } else {
                            tracing::error!("Failed to parse theme file");
                        }
                    }
                }
            }
        },
    );
}

/// Save theme to user's themes directory
fn save_theme(editor: &ThemeEditor) {
    let theme = editor.get_theme();
    let config_dir = corgiterm_config::config_dir();
    let themes_dir = config_dir.join("themes");

    if let Err(e) = std::fs::create_dir_all(&themes_dir) {
        tracing::error!("Failed to create themes directory: {}", e);
        return;
    }

    let filename = format!("{}.toml", theme.name.to_lowercase().replace(' ', "-"));
    let path = themes_dir.join(filename);

    if let Ok(toml_string) = toml::to_string_pretty(&theme) {
        if let Err(e) = std::fs::write(&path, toml_string) {
            tracing::error!("Failed to save theme: {}", e);
        } else {
            tracing::info!("Theme saved to: {:?}", path);
        }
    }
}
