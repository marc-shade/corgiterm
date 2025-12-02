//! # ASCII Art Generator Dialog
//!
//! GTK4 dialog for generating ASCII art from images and text.
//!
//! Features:
//! - Image to ASCII conversion with live preview
//! - Multiple image filters (edge detection, high contrast, posterize, dither)
//! - Brightness and contrast adjustments
//! - Drag-drop image support
//! - Text to ASCII art with 6 font styles
//! - Aspect ratio control
//! - Built-in Corgi art collection
//! - Template gallery
//! - Copy to clipboard or insert into terminal

use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, DropDown, FileDialog, Label, Notebook, Orientation, Scale, ScrolledWindow,
    SelectionMode, StringList, TextView, Window,
};
use libadwaita::{prelude::*, ActionRow, HeaderBar, PreferencesGroup};

use corgiterm_core::ascii_art::{
    all_fonts, AsciiArtConfig, AsciiArtGenerator, CharacterSet, CorgiArt, ImageFilter,
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
    image_filter_dropdown: DropDown,
    image_brightness_scale: Scale,
    image_contrast_scale: Scale,
    image_aspect_scale: Scale,
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

    // History
    history: Rc<RefCell<Vec<String>>>,

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
        let (
            image_tab,
            image_preview,
            image_width_scale,
            image_charset_dropdown,
            image_filter_dropdown,
            image_brightness_scale,
            image_contrast_scale,
            image_aspect_scale,
            image_colored_check,
            image_inverted_check,
        ) = Self::create_image_tab();
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
        let history: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

        let mut ascii_dialog = Self {
            dialog,
            notebook,
            image_preview,
            image_width_scale,
            image_charset_dropdown,
            image_filter_dropdown,
            image_brightness_scale,
            image_contrast_scale,
            image_aspect_scale,
            image_colored_check,
            image_inverted_check,
            current_image,
            text_input,
            text_font_dropdown,
            text_preview,
            corgi_dropdown,
            corgi_preview,
            history,
            result,
            insert_callback,
        };

        ascii_dialog.connect_signals(cancel_btn, copy_btn, insert_btn);

        // History tab
        let history_tab = ascii_dialog.create_history_tab();
        ascii_dialog
            .notebook
            .append_page(&history_tab, Some(&Label::new(Some("History"))));

        ascii_dialog
    }

    fn create_history_tab(&self) -> Box {
        let vbox = Box::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

        let list = gtk4::ListBox::new();
        list.set_selection_mode(SelectionMode::None);
        scrolled.set_child(Some(&list));
        vbox.append(&scrolled);

        // Populate from history
        let history = self.history.clone();
        let insert_cb = self.insert_callback.clone();
        let result_ref = self.result.clone();
        let preview_ref = self.image_preview.clone();
        let list_ref = list.clone();
        gtk4::glib::idle_add_local(move || {
            list_ref.remove_all();
            for entry in history.borrow().iter().rev().take(20) {
                let row = libadwaita::ActionRow::builder()
                    .title("ASCII art")
                    .subtitle(format!("{} chars", entry.len()))
                    .activatable(true)
                    .build();
                let copy_btn = Button::from_icon_name("edit-copy-symbolic");
                copy_btn.add_css_class("flat");
                copy_btn.set_tooltip_text(Some("Copy"));
                let entry_text = entry.clone();
                copy_btn.connect_clicked(move |_| {
                    let _ = gtk4::gdk::Display::default()
                        .map(|d| d.clipboard())
                        .map(|cb| cb.set_text(&entry_text));
                });
                row.add_suffix(&copy_btn);

                let insert_btn = Button::from_icon_name("insert-text-symbolic");
                insert_btn.add_css_class("flat");
                insert_btn.set_tooltip_text(Some("Insert"));
                let entry_text_insert = entry.clone();
                let insert_cb = insert_cb.clone();
                insert_btn.connect_clicked(move |_| {
                    if let Some(cb) = &*insert_cb.borrow() {
                        cb(&entry_text_insert);
                    }
                });
                row.add_suffix(&insert_btn);

                // Preview click
                let entry_text_preview = entry.clone();
                let preview_ref = preview_ref.clone();
                let result_ref = result_ref.clone();
                row.connect_activated(move |_| {
                    let buffer = preview_ref.buffer();
                    buffer.set_text(&entry_text_preview);
                    *result_ref.borrow_mut() = Some(entry_text_preview.clone());
                });

                list_ref.append(&row);
            }
            gtk4::glib::ControlFlow::Break
        });

        vbox
    }

    /// Create the image tab
    #[allow(clippy::type_complexity)]
    fn create_image_tab() -> (
        Box,
        TextView,
        Scale,
        DropDown,
        DropDown,
        Scale,
        Scale,
        Scale,
        gtk4::CheckButton,
        gtk4::CheckButton,
    ) {
        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        // File picker button with drag-drop hint
        let file_box = Box::new(Orientation::Horizontal, 12);
        let file_btn = Button::with_label("Choose Image...");
        file_btn.set_halign(Align::Start);
        file_box.append(&file_btn);

        let drag_label = Label::new(Some("or drag & drop image here"));
        drag_label.add_css_class("dim-label");
        file_box.append(&drag_label);
        vbox.append(&file_box);

        // Settings
        let settings_group = PreferencesGroup::new();
        settings_group.set_title("Image Settings");

        // Width slider
        let width_row = ActionRow::new();
        width_row.set_title("Width (characters)");
        let width_scale = Scale::with_range(Orientation::Horizontal, 20.0, 200.0, 5.0);
        width_scale.set_value(80.0);
        width_scale.set_draw_value(true);
        width_scale.set_hexpand(true);
        width_row.add_suffix(&width_scale);
        settings_group.add(&width_row);

        // Aspect ratio slider
        let aspect_row = ActionRow::new();
        aspect_row.set_title("Aspect Ratio");
        aspect_row.set_subtitle("Adjust for terminal font aspect");
        let aspect_scale = Scale::with_range(Orientation::Horizontal, 0.3, 1.0, 0.05);
        aspect_scale.set_value(0.5);
        aspect_scale.set_draw_value(true);
        aspect_scale.set_hexpand(true);
        aspect_row.add_suffix(&aspect_scale);
        settings_group.add(&aspect_row);

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

        // Image Filter dropdown
        let filter_row = ActionRow::new();
        filter_row.set_title("Filter");
        filter_row.set_subtitle("Apply image processing");
        let filter_names: Vec<&str> = ImageFilter::all().iter().map(|f| f.name()).collect();
        let filter_model = StringList::new(&filter_names);
        let filter_dropdown = DropDown::builder().model(&filter_model).selected(0).build();
        filter_row.add_suffix(&filter_dropdown);
        settings_group.add(&filter_row);

        vbox.append(&settings_group);

        // Adjustments group
        let adjust_group = PreferencesGroup::new();
        adjust_group.set_title("Adjustments");

        // Brightness slider
        let brightness_row = ActionRow::new();
        brightness_row.set_title("Brightness");
        let brightness_scale = Scale::with_range(Orientation::Horizontal, -100.0, 100.0, 5.0);
        brightness_scale.set_value(0.0);
        brightness_scale.set_draw_value(true);
        brightness_scale.set_hexpand(true);
        brightness_row.add_suffix(&brightness_scale);
        adjust_group.add(&brightness_row);

        // Contrast slider
        let contrast_row = ActionRow::new();
        contrast_row.set_title("Contrast");
        let contrast_scale = Scale::with_range(Orientation::Horizontal, -100.0, 100.0, 5.0);
        contrast_scale.set_value(0.0);
        contrast_scale.set_draw_value(true);
        contrast_scale.set_hexpand(true);
        contrast_row.add_suffix(&contrast_scale);
        adjust_group.add(&contrast_row);

        // Colored checkbox
        let colored_row = ActionRow::new();
        colored_row.set_title("ANSI Colors");
        colored_row.set_subtitle("Include terminal colors");
        let colored_check = gtk4::CheckButton::new();
        colored_row.add_suffix(&colored_check);
        colored_row.set_activatable_widget(Some(&colored_check));
        adjust_group.add(&colored_row);

        // Inverted checkbox
        let inverted_row = ActionRow::new();
        inverted_row.set_title("Invert");
        inverted_row.set_subtitle("For light backgrounds");
        let inverted_check = gtk4::CheckButton::new();
        inverted_row.add_suffix(&inverted_check);
        inverted_row.set_activatable_widget(Some(&inverted_check));
        adjust_group.add(&inverted_row);

        vbox.append(&adjust_group);

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
        buffer.set_text("Select an image or drag & drop to preview...");
        scrolled.set_child(Some(&preview));
        vbox.append(&scrolled);

        (
            vbox,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
        )
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

        // Font selection - use all 6 fonts
        let font_group = PreferencesGroup::new();
        font_group.set_title("Font");

        let font_row = ActionRow::new();
        font_row.set_title("Font Style");
        font_row.set_subtitle("Figlet-style ASCII fonts");
        let font_names: Vec<&str> = all_fonts().iter().map(|f| f.name).collect();
        let font_model = StringList::new(&font_names);
        let font_dropdown = DropDown::builder().model(&font_model).selected(0).build();
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
        let corgi_dropdown = DropDown::builder().model(&corgi_model).selected(0).build();
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
        let history_clone = self.history.clone();
        let dialog_clone = self.dialog.clone();
        copy_btn.connect_clicked(move |_| {
            if let Some(text) = result_clone.borrow().as_ref() {
                let clipboard = dialog_clone.clipboard();
                clipboard.set_text(text);
                history_clone.borrow_mut().push(text.clone());
            }
        });

        // Insert button - sends ASCII art to terminal
        let result_clone = self.result.clone();
        let dialog_clone = self.dialog.clone();
        let insert_cb_clone = self.insert_callback.clone();
        let history_clone = self.history.clone();
        insert_btn.connect_clicked(move |_| {
            if let Some(text) = result_clone.borrow().as_ref() {
                // Call the callback to insert text into terminal
                if let Some(ref callback) = *insert_cb_clone.borrow() {
                    callback(text);
                    tracing::info!("Inserted ASCII art into terminal ({} bytes)", text.len());
                    history_clone.borrow_mut().push(text.clone());
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

    /// Connect image file picker and drag-drop
    fn connect_image_picker(&self) {
        let dialog = self.dialog.clone();
        let current_image = self.current_image.clone();
        let preview = self.image_preview.clone();
        let width_scale = self.image_width_scale.clone();
        let charset_dropdown = self.image_charset_dropdown.clone();
        let filter_dropdown = self.image_filter_dropdown.clone();
        let brightness_scale = self.image_brightness_scale.clone();
        let contrast_scale = self.image_contrast_scale.clone();
        let aspect_scale = self.image_aspect_scale.clone();
        let colored_check = self.image_colored_check.clone();
        let inverted_check = self.image_inverted_check.clone();
        let result = self.result.clone();

        // Find the file button in image tab
        if let Some(notebook_page) = self.notebook.nth_page(Some(0)) {
            if let Some(vbox) = notebook_page.downcast_ref::<Box>() {
                // Setup drag-drop on the vbox
                let drop_target =
                    gtk4::DropTarget::new(gtk4::gio::File::static_type(), gdk::DragAction::COPY);
                let current_image_drop = current_image.clone();
                let preview_drop = preview.clone();
                let width_drop = width_scale.clone();
                let charset_drop = charset_dropdown.clone();
                let filter_drop = filter_dropdown.clone();
                let brightness_drop = brightness_scale.clone();
                let contrast_drop = contrast_scale.clone();
                let aspect_drop = aspect_scale.clone();
                let colored_drop = colored_check.clone();
                let inverted_drop = inverted_check.clone();
                let result_drop = result.clone();

                drop_target.connect_drop(move |_target, value, _x, _y| {
                    if let Ok(file) = value.get::<gtk4::gio::File>() {
                        if let Some(path) = file.path() {
                            if let Ok(img) = image::open(&path) {
                                *current_image_drop.borrow_mut() = Some(img.clone());
                                Self::update_image_preview(
                                    &img,
                                    &preview_drop,
                                    &width_drop,
                                    &charset_drop,
                                    &filter_drop,
                                    &brightness_drop,
                                    &contrast_drop,
                                    &aspect_drop,
                                    &colored_drop,
                                    &inverted_drop,
                                );
                                // Store result
                                let buffer = preview_drop.buffer();
                                let text =
                                    buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                                *result_drop.borrow_mut() = Some(text.to_string());
                                return true;
                            }
                        }
                    }
                    false
                });
                vbox.add_controller(drop_target);

                // Connect file button
                if let Some(file_box) = vbox.first_child() {
                    if let Some(file_btn_box) = file_box.downcast_ref::<Box>() {
                        if let Some(button) = file_btn_box.first_child() {
                            if let Some(button) = button.downcast_ref::<Button>() {
                                let current_image_btn = current_image.clone();
                                let preview_btn = preview.clone();
                                let width_btn = width_scale.clone();
                                let charset_btn = charset_dropdown.clone();
                                let filter_btn = filter_dropdown.clone();
                                let brightness_btn = brightness_scale.clone();
                                let contrast_btn = contrast_scale.clone();
                                let aspect_btn = aspect_scale.clone();
                                let colored_btn = colored_check.clone();
                                let inverted_btn = inverted_check.clone();
                                let dialog_btn = dialog.clone();
                                let result_btn = result.clone();

                                button.connect_clicked(move |_btn| {
                                    let file_dialog = FileDialog::builder()
                                        .title("Choose Image")
                                        .modal(true)
                                        .build();

                                    let current_image_clone = current_image_btn.clone();
                                    let preview_clone = preview_btn.clone();
                                    let width_clone = width_btn.clone();
                                    let charset_clone = charset_btn.clone();
                                    let filter_clone = filter_btn.clone();
                                    let brightness_clone = brightness_btn.clone();
                                    let contrast_clone = contrast_btn.clone();
                                    let aspect_clone = aspect_btn.clone();
                                    let colored_clone = colored_btn.clone();
                                    let inverted_clone = inverted_btn.clone();
                                    let dialog_clone = dialog_btn.clone();
                                    let result_clone = result_btn.clone();

                                    file_dialog.open(
                                        Some(&dialog_clone),
                                        None::<&gtk4::gio::Cancellable>,
                                        move |result| {
                                            if let Ok(file) = result {
                                                if let Some(path) = file.path() {
                                                    if let Ok(img) = image::open(&path) {
                                                        *current_image_clone.borrow_mut() =
                                                            Some(img.clone());
                                                        Self::update_image_preview(
                                                            &img,
                                                            &preview_clone,
                                                            &width_clone,
                                                            &charset_clone,
                                                            &filter_clone,
                                                            &brightness_clone,
                                                            &contrast_clone,
                                                            &aspect_clone,
                                                            &colored_clone,
                                                            &inverted_clone,
                                                        );
                                                        // Store result
                                                        let buffer = preview_clone.buffer();
                                                        let text = buffer.text(
                                                            &buffer.start_iter(),
                                                            &buffer.end_iter(),
                                                            false,
                                                        );
                                                        *result_clone.borrow_mut() =
                                                            Some(text.to_string());
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
        }
    }

    /// Connect image settings changes
    fn connect_image_settings(&self) {
        // Helper macro to reduce repetition
        macro_rules! connect_image_control {
            ($control:expr, $signal:ident, $current_image:expr, $preview:expr,
             $width:expr, $charset:expr, $filter:expr, $brightness:expr,
             $contrast:expr, $aspect:expr, $colored:expr, $inverted:expr, $result:expr) => {{
                let current_image = $current_image.clone();
                let preview = $preview.clone();
                let width = $width.clone();
                let charset = $charset.clone();
                let filter = $filter.clone();
                let brightness = $brightness.clone();
                let contrast = $contrast.clone();
                let aspect = $aspect.clone();
                let colored = $colored.clone();
                let inverted = $inverted.clone();
                let result = $result.clone();
                $control.$signal(move |_| {
                    if let Some(img) = current_image.borrow().as_ref() {
                        Self::update_image_preview(
                            img,
                            &preview,
                            &width,
                            &charset,
                            &filter,
                            &brightness,
                            &contrast,
                            &aspect,
                            &colored,
                            &inverted,
                        );
                        let buffer = preview.buffer();
                        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                        *result.borrow_mut() = Some(text.to_string());
                    }
                });
            }};
        }

        let current_image = &self.current_image;
        let preview = &self.image_preview;
        let width_scale = &self.image_width_scale;
        let charset_dropdown = &self.image_charset_dropdown;
        let filter_dropdown = &self.image_filter_dropdown;
        let brightness_scale = &self.image_brightness_scale;
        let contrast_scale = &self.image_contrast_scale;
        let aspect_scale = &self.image_aspect_scale;
        let colored_check = &self.image_colored_check;
        let inverted_check = &self.image_inverted_check;
        let result = &self.result;

        // Width change
        connect_image_control!(
            width_scale,
            connect_value_changed,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Charset change
        connect_image_control!(
            charset_dropdown,
            connect_selected_notify,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Filter change
        connect_image_control!(
            filter_dropdown,
            connect_selected_notify,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Brightness change
        connect_image_control!(
            brightness_scale,
            connect_value_changed,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Contrast change
        connect_image_control!(
            contrast_scale,
            connect_value_changed,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Aspect change
        connect_image_control!(
            aspect_scale,
            connect_value_changed,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Color checkbox
        connect_image_control!(
            colored_check,
            connect_toggled,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );

        // Inverted checkbox
        connect_image_control!(
            inverted_check,
            connect_toggled,
            current_image,
            preview,
            width_scale,
            charset_dropdown,
            filter_dropdown,
            brightness_scale,
            contrast_scale,
            aspect_scale,
            colored_check,
            inverted_check,
            result
        );
    }

    /// Update image preview
    fn update_image_preview(
        img: &image::DynamicImage,
        preview: &TextView,
        width_scale: &Scale,
        charset_dropdown: &DropDown,
        filter_dropdown: &DropDown,
        brightness_scale: &Scale,
        contrast_scale: &Scale,
        aspect_scale: &Scale,
        colored_check: &gtk4::CheckButton,
        inverted_check: &gtk4::CheckButton,
    ) {
        let width = width_scale.value() as usize;
        let charset_idx = charset_dropdown.selected() as usize;
        let charset = CharacterSet::all()[charset_idx];
        let filter_idx = filter_dropdown.selected() as usize;
        let filter = ImageFilter::all()[filter_idx];
        let brightness = brightness_scale.value() as i32;
        let contrast = contrast_scale.value() as i32;
        let aspect = aspect_scale.value() as f32;
        let colored = colored_check.is_active();
        let inverted = inverted_check.is_active();

        let config = AsciiArtConfig {
            width: Some(width),
            charset,
            colored,
            inverted,
            aspect_ratio: aspect,
            filter,
            brightness,
            contrast,
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

        let fonts = all_fonts();
        let font_idx = font_dropdown.selected() as usize;
        let font = fonts.get(font_idx).unwrap_or(&fonts[0]);

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
