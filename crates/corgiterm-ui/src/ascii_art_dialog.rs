//! # ASCII Art Generator Dialog
//!
//! GTK4 dialog for generating ASCII art from images and text.
//!
//! Features:
//! - Image to ASCII conversion with live preview
//! - Text to ASCII art
//! - Built-in Corgi art collection
//! - Copy to clipboard or insert into terminal

use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, DropDown, FileDialog, StringList,
    Label, Notebook, Orientation, Scale, ScrolledWindow, TextView, Window,
};
use libadwaita::{prelude::*, ActionRow, HeaderBar, PreferencesGroup};

use corgiterm_core::ascii_art::{
    AsciiArtConfig, AsciiArtGenerator, CharacterSet, CorgiArt, FONT_SMALL, FONT_STANDARD,
};

use std::cell::RefCell;
use std::rc::Rc;

/// Callback type for inserting ASCII art into terminal
pub type InsertCallback = std::boxed::Box<dyn Fn(&str)>;

/// ASCII Art Generator Dialog
pub struct AsciiArtDialog {
    dialog: libadwaita::Window,
    notebook: Notebook,

    // Image tab
    image_preview: TextView,
    image_width_scale: Scale,
    image_charset_dropdown: DropDown,
    image_colored_check: gtk4::CheckButton,
    image_inverted_check: gtk4::CheckButton,
    current_image: Rc<RefCell<Option<image::DynamicImage>>>,

    // Text tab
    text_input: gtk4::Entry,
    text_font_dropdown: DropDown,
    text_preview: TextView,

    // Corgi tab (corgi_dropdown stored but accessed via connect_selected_notify callback)
    #[allow(dead_code)]
    corgi_dropdown: DropDown,
    corgi_preview: TextView,

    // Result
    result: Rc<RefCell<Option<String>>>,

    // Callback for inserting to terminal
    insert_callback: Rc<RefCell<Option<InsertCallback>>>,
}

impl AsciiArtDialog {
    /// Create a new ASCII Art Generator dialog
    pub fn new<W: IsA<Window> + IsA<gtk4::Widget>>(parent: &W) -> Self {
        let dialog = libadwaita::Window::builder()
            .transient_for(parent)
            .modal(true)
            .default_width(800)
            .default_height(600)
            .title("ASCII Art Generator")
            .build();

        // Header bar
        let header = HeaderBar::new();

        let cancel_btn = Button::with_label("Close");
        let copy_btn = Button::with_label("Copy");
        let insert_btn = Button::with_label("Insert");

        copy_btn.add_css_class("suggested-action");
        insert_btn.add_css_class("suggested-action");

        header.pack_start(&cancel_btn);
        header.pack_end(&insert_btn);
        header.pack_end(&copy_btn);

        // Notebook for tabs
        let notebook = Notebook::new();
        notebook.set_vexpand(true);

        // Image tab
        let (image_tab, image_preview, image_width_scale, image_charset_dropdown,
             image_colored_check, image_inverted_check) = Self::create_image_tab();
        notebook.append_page(&image_tab, Some(&Label::new(Some("From Image"))));

        // Text tab
        let (text_tab, text_input, text_font_dropdown, text_preview) = Self::create_text_tab();
        notebook.append_page(&text_tab, Some(&Label::new(Some("From Text"))));

        // Corgi tab
        let (corgi_tab, corgi_dropdown, corgi_preview) = Self::create_corgi_tab();
        notebook.append_page(&corgi_tab, Some(&Label::new(Some("🐕 Corgi Art"))));

        // Layout
        let content = Box::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&notebook);

        dialog.set_content(Some(&content));

        let current_image = Rc::new(RefCell::new(None));
        let result = Rc::new(RefCell::new(None));
        let insert_callback: Rc<RefCell<Option<InsertCallback>>> = Rc::new(RefCell::new(None));

        let mut ascii_dialog = Self {
            dialog,
            notebook,
            image_preview,
            image_width_scale,
            image_charset_dropdown,
            image_colored_check,
            image_inverted_check,
            current_image,
            text_input,
            text_font_dropdown,
            text_preview,
            corgi_dropdown,
            corgi_preview,
            result,
            insert_callback,
        };

        ascii_dialog.connect_signals(cancel_btn, copy_btn, insert_btn);
        ascii_dialog
    }

    /// Create the image tab
    fn create_image_tab() -> (Box, TextView, Scale, DropDown, gtk4::CheckButton, gtk4::CheckButton) {
        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        // File picker button
        let file_btn = Button::with_label("Choose Image...");
        file_btn.set_halign(Align::Start);
        vbox.append(&file_btn);

        // Settings
        let settings_group = PreferencesGroup::new();
        settings_group.set_title("Settings");

        // Width slider
        let width_row = ActionRow::new();
        width_row.set_title("Width (characters)");
        let width_scale = Scale::with_range(Orientation::Horizontal, 20.0, 200.0, 5.0);
        width_scale.set_value(80.0);
        width_scale.set_draw_value(true);
        width_scale.set_hexpand(true);
        width_row.add_suffix(&width_scale);
        settings_group.add(&width_row);

        // Character set - use modern DropDown
        let charset_row = ActionRow::new();
        charset_row.set_title("Character Set");
        let charset_names: Vec<&str> = CharacterSet::all().iter().map(|s| s.name()).collect();
        let charset_model = StringList::new(&charset_names);
        let charset_dropdown = DropDown::builder()
            .model(&charset_model)
            .selected(0)
            .build();
        charset_row.add_suffix(&charset_dropdown);
        settings_group.add(&charset_row);

        // Colored checkbox
        let colored_row = ActionRow::new();
        colored_row.set_title("ANSI Colors");
        let colored_check = gtk4::CheckButton::new();
        colored_row.add_suffix(&colored_check);
        colored_row.set_activatable_widget(Some(&colored_check));
        settings_group.add(&colored_row);

        // Inverted checkbox
        let inverted_row = ActionRow::new();
        inverted_row.set_title("Invert (for white backgrounds)");
        let inverted_check = gtk4::CheckButton::new();
        inverted_row.add_suffix(&inverted_check);
        inverted_row.set_activatable_widget(Some(&inverted_check));
        settings_group.add(&inverted_row);

        vbox.append(&settings_group);

        // Preview
        let preview_label = Label::new(Some("Preview:"));
        preview_label.set_halign(Align::Start);
        vbox.append(&preview_label);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        let preview = TextView::new();
        preview.set_editable(false);
        preview.set_monospace(true);
        let buffer = preview.buffer();
        buffer.set_text("Select an image to preview...");
        scrolled.set_child(Some(&preview));
        vbox.append(&scrolled);

        (vbox, preview, width_scale, charset_dropdown, colored_check, inverted_check)
    }

    /// Create the text tab
    fn create_text_tab() -> (Box, gtk4::Entry, DropDown, TextView) {
        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        // Text input
        let input_label = Label::new(Some("Enter text:"));
        input_label.set_halign(Align::Start);
        vbox.append(&input_label);

        let text_input = gtk4::Entry::new();
        text_input.set_placeholder_text(Some("Enter text here..."));
        text_input.set_max_length(20); // ASCII art gets wide quickly
        vbox.append(&text_input);

        // Font selection - use modern DropDown
        let font_group = PreferencesGroup::new();
        font_group.set_title("Font");

        let font_row = ActionRow::new();
        font_row.set_title("Font Style");
        let font_model = StringList::new(&["Standard", "Small"]);
        let font_dropdown = DropDown::builder()
            .model(&font_model)
            .selected(0)
            .build();
        font_row.add_suffix(&font_dropdown);
        font_group.add(&font_row);

        vbox.append(&font_group);

        // Preview
        let preview_label = Label::new(Some("Preview:"));
        preview_label.set_halign(Align::Start);
        vbox.append(&preview_label);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        let preview = TextView::new();
        preview.set_editable(false);
        preview.set_monospace(true);
        let buffer = preview.buffer();
        buffer.set_text("Enter text to preview...");
        scrolled.set_child(Some(&preview));
        vbox.append(&scrolled);

        (vbox, text_input, font_dropdown, preview)
    }

    /// Create the corgi tab
    fn create_corgi_tab() -> (Box, DropDown, TextView) {
        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let label = Label::new(Some("Choose a corgi:"));
        label.set_halign(Align::Start);
        vbox.append(&label);

        // Use modern DropDown instead of ComboBoxText
        let corgi_names: Vec<&str> = CorgiArt::all().iter().map(|(name, _)| *name).collect();
        let corgi_model = StringList::new(&corgi_names);
        let corgi_dropdown = DropDown::builder()
            .model(&corgi_model)
            .selected(0)
            .build();
        vbox.append(&corgi_dropdown);

        let random_btn = Button::with_label("🎲 Random Corgi");
        random_btn.set_halign(Align::Start);
        vbox.append(&random_btn);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        let preview = TextView::new();
        preview.set_editable(false);
        preview.set_monospace(true);
        let buffer = preview.buffer();
        buffer.set_text(CorgiArt::all()[0].1);
        scrolled.set_child(Some(&preview));
        vbox.append(&scrolled);

        // Connect random button
        let preview_clone = preview.clone();
        random_btn.connect_clicked(move |_| {
            let art = CorgiArt::random();
            let buffer = preview_clone.buffer();
            buffer.set_text(art);
        });

        // Connect corgi selection using DropDown's selected_notify
        let preview_clone = preview.clone();
        corgi_dropdown.connect_selected_notify(move |dropdown| {
            let idx = dropdown.selected() as usize;
            let arts = CorgiArt::all();
            if let Some((_, art)) = arts.get(idx) {
                let buffer = preview_clone.buffer();
                buffer.set_text(art);
            }
        });

        (vbox, corgi_dropdown, preview)
    }

    /// Connect all signals
    fn connect_signals(&mut self, cancel_btn: Button, copy_btn: Button, insert_btn: Button) {
        // Cancel button
        let dialog_clone = self.dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        // Copy button
        let result_clone = self.result.clone();
        let dialog_clone = self.dialog.clone();
        copy_btn.connect_clicked(move |_| {
            if let Some(text) = result_clone.borrow().as_ref() {
                let clipboard = dialog_clone.clipboard();
                clipboard.set_text(text);
            }
        });

        // Insert button - sends ASCII art to terminal
        let result_clone = self.result.clone();
        let dialog_clone = self.dialog.clone();
        let insert_cb_clone = self.insert_callback.clone();
        insert_btn.connect_clicked(move |_| {
            if let Some(text) = result_clone.borrow().as_ref() {
                // Call the callback to insert text into terminal
                if let Some(ref callback) = *insert_cb_clone.borrow() {
                    callback(text);
                    tracing::info!("Inserted ASCII art into terminal ({} bytes)", text.len());
                } else {
                    tracing::warn!("Insert button clicked but no callback set");
                }
                dialog_clone.close();
            }
        });

        // Image file picker
        self.connect_image_picker();

        // Image settings changes
        self.connect_image_settings();

        // Text input changes
        self.connect_text_input();

        // Connect corgi tab result storage
        self.connect_corgi_result_storage();
    }

    /// Connect corgi tab preview changes to result storage
    fn connect_corgi_result_storage(&self) {
        let preview = self.corgi_preview.clone();
        let result = self.result.clone();

        // Initialize with first corgi art
        let buffer = preview.buffer();
        let initial_art = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
        *result.borrow_mut() = Some(initial_art.to_string());

        // Update result whenever buffer changes
        let result_for_notify = result.clone();
        buffer.connect_changed(move |buf| {
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
            *result_for_notify.borrow_mut() = Some(text.to_string());
        });
    }

    /// Connect image file picker
    fn connect_image_picker(&self) {
        let dialog = self.dialog.clone();
        let current_image = self.current_image.clone();
        let preview = self.image_preview.clone();
        let width_scale = self.image_width_scale.clone();
        let charset_dropdown = self.image_charset_dropdown.clone();
        let colored_check = self.image_colored_check.clone();
        let inverted_check = self.image_inverted_check.clone();

        // Find the file button in image tab
        if let Some(notebook_page) = self.notebook.nth_page(Some(0)) {
            if let Some(vbox) = notebook_page.downcast_ref::<Box>() {
                if let Some(file_btn) = vbox.first_child() {
                    if let Some(button) = file_btn.downcast_ref::<Button>() {
                        button.connect_clicked(move |_btn| {
                            // Use modern FileDialog instead of deprecated FileChooserDialog
                            let file_dialog = FileDialog::builder()
                                .title("Choose Image")
                                .modal(true)
                                .build();

                            let current_image_clone = current_image.clone();
                            let preview_clone = preview.clone();
                            let width_scale_clone = width_scale.clone();
                            let charset_dropdown_clone = charset_dropdown.clone();
                            let colored_check_clone = colored_check.clone();
                            let inverted_check_clone = inverted_check.clone();
                            let dialog_clone = dialog.clone();

                            file_dialog.open(
                                Some(&dialog_clone),
                                None::<&gtk4::gio::Cancellable>,
                                move |result| {
                                    if let Ok(file) = result {
                                        if let Some(path) = file.path() {
                                            if let Ok(img) = image::open(&path) {
                                                *current_image_clone.borrow_mut() = Some(img.clone());
                                                Self::update_image_preview(
                                                    &img,
                                                    &preview_clone,
                                                    &width_scale_clone,
                                                    &charset_dropdown_clone,
                                                    &colored_check_clone,
                                                    &inverted_check_clone,
                                                );
                                            }
                                        }
                                    }
                                },
                            );
                        });
                    }
                }
            }
        }
    }

    /// Connect image settings changes
    fn connect_image_settings(&self) {
        let current_image = self.current_image.clone();
        let preview = self.image_preview.clone();
        let width_scale = self.image_width_scale.clone();
        let charset_dropdown = self.image_charset_dropdown.clone();
        let colored_check = self.image_colored_check.clone();
        let inverted_check = self.image_inverted_check.clone();
        let result = self.result.clone();

        // Width change
        let current_image_clone = current_image.clone();
        let preview_clone = preview.clone();
        let charset_dropdown_clone = charset_dropdown.clone();
        let colored_check_clone = colored_check.clone();
        let inverted_check_clone = inverted_check.clone();
        let result_clone = result.clone();
        width_scale.connect_value_changed(move |scale| {
            if let Some(img) = current_image_clone.borrow().as_ref() {
                Self::update_image_preview(
                    img,
                    &preview_clone,
                    scale,
                    &charset_dropdown_clone,
                    &colored_check_clone,
                    &inverted_check_clone,
                );
                // Store result
                let buffer = preview_clone.buffer();
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                *result_clone.borrow_mut() = Some(text.to_string());
            }
        });

        // Charset change - use connect_selected_notify for DropDown
        let current_image_clone = current_image.clone();
        let preview_clone = preview.clone();
        let width_scale_clone = width_scale.clone();
        let colored_check_clone = colored_check.clone();
        let inverted_check_clone = inverted_check.clone();
        let result_clone = result.clone();
        charset_dropdown.connect_selected_notify(move |dropdown| {
            if let Some(img) = current_image_clone.borrow().as_ref() {
                Self::update_image_preview(
                    img,
                    &preview_clone,
                    &width_scale_clone,
                    dropdown,
                    &colored_check_clone,
                    &inverted_check_clone,
                );
                // Store result
                let buffer = preview_clone.buffer();
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                *result_clone.borrow_mut() = Some(text.to_string());
            }
        });

        // Color checkbox
        let current_image_clone = current_image.clone();
        let preview_clone = preview.clone();
        let width_scale_clone = width_scale.clone();
        let charset_dropdown_clone = charset_dropdown.clone();
        let inverted_check_clone = inverted_check.clone();
        let result_clone = result.clone();
        colored_check.connect_toggled(move |check| {
            if let Some(img) = current_image_clone.borrow().as_ref() {
                Self::update_image_preview(
                    img,
                    &preview_clone,
                    &width_scale_clone,
                    &charset_dropdown_clone,
                    check,
                    &inverted_check_clone,
                );
                // Store result
                let buffer = preview_clone.buffer();
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                *result_clone.borrow_mut() = Some(text.to_string());
            }
        });

        // Inverted checkbox
        let current_image_clone = current_image;
        let preview_clone = preview;
        let width_scale_clone = width_scale;
        let charset_dropdown_clone = charset_dropdown;
        let colored_check_clone = colored_check;
        inverted_check.connect_toggled(move |check| {
            if let Some(img) = current_image_clone.borrow().as_ref() {
                Self::update_image_preview(
                    img,
                    &preview_clone,
                    &width_scale_clone,
                    &charset_dropdown_clone,
                    &colored_check_clone,
                    check,
                );
                // Store result
                let buffer = preview_clone.buffer();
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                *result.borrow_mut() = Some(text.to_string());
            }
        });
    }

    /// Update image preview
    fn update_image_preview(
        img: &image::DynamicImage,
        preview: &TextView,
        width_scale: &Scale,
        charset_dropdown: &DropDown,
        colored_check: &gtk4::CheckButton,
        inverted_check: &gtk4::CheckButton,
    ) {
        let width = width_scale.value() as usize;
        let charset_idx = charset_dropdown.selected() as usize;
        let charset = CharacterSet::all()[charset_idx];
        let colored = colored_check.is_active();
        let inverted = inverted_check.is_active();

        let config = AsciiArtConfig {
            width: Some(width),
            charset,
            colored,
            inverted,
            aspect_ratio: 0.5,
        };

        let generator = AsciiArtGenerator::with_config(config);
        if let Ok(art) = generator.from_image(img) {
            let buffer = preview.buffer();
            buffer.set_text(&art);
        }
    }

    /// Update result storage when preview changes
    #[allow(dead_code)]
    fn store_preview_result(&self, text: &str) {
        *self.result.borrow_mut() = Some(text.to_string());
    }

    /// Connect text input changes
    fn connect_text_input(&self) {
        let text_input = self.text_input.clone();
        let font_dropdown = self.text_font_dropdown.clone();
        let preview = self.text_preview.clone();
        let result = self.result.clone();

        // Text changed
        let font_dropdown_clone = font_dropdown.clone();
        let preview_clone = preview.clone();
        let result_clone = result.clone();
        text_input.connect_changed(move |entry| {
            let text = entry.text();
            Self::update_text_preview(&text, &font_dropdown_clone, &preview_clone);
            // Store result
            let buffer = preview_clone.buffer();
            let art = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            if !text.is_empty() && !art.as_str().starts_with("Enter text") {
                *result_clone.borrow_mut() = Some(art.to_string());
            }
        });

        // Font changed - use connect_selected_notify for DropDown
        let text_input_clone = text_input.clone();
        let preview_clone = preview.clone();
        let result_clone = result.clone();
        font_dropdown.connect_selected_notify(move |dropdown| {
            let text = text_input_clone.text();
            Self::update_text_preview(&text, dropdown, &preview_clone);
            // Store result
            let buffer = preview_clone.buffer();
            let art = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            if !text.is_empty() && !art.as_str().starts_with("Enter text") {
                *result_clone.borrow_mut() = Some(art.to_string());
            }
        });
    }

    /// Update text preview
    fn update_text_preview(text: &str, font_dropdown: &DropDown, preview: &TextView) {
        if text.is_empty() {
            let buffer = preview.buffer();
            buffer.set_text("Enter text to preview...");
            return;
        }

        let font = match font_dropdown.selected() {
            0 => &FONT_STANDARD,
            1 => &FONT_SMALL,
            _ => &FONT_STANDARD,
        };

        let generator = AsciiArtGenerator::new();
        if let Ok(art) = generator.from_text(text, font) {
            let buffer = preview.buffer();
            buffer.set_text(&art);
        }
    }

    /// Show the dialog
    pub fn show(&self) {
        self.dialog.present();
    }

    /// Get the generated ASCII art result
    pub fn result(&self) -> Option<String> {
        self.result.borrow().clone()
    }

    /// Set callback for inserting ASCII art into terminal
    pub fn set_insert_callback<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        *self.insert_callback.borrow_mut() = Some(std::boxed::Box::new(callback));
    }
}

/// Show ASCII Art Generator dialog
pub fn show_ascii_art_dialog<W: IsA<Window> + IsA<gtk4::Widget>>(parent: &W) {
    let dialog = AsciiArtDialog::new(parent);
    dialog.show();
}
