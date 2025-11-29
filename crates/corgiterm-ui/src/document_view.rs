//! Document viewer with view/edit mode switching

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, Stack, StackTransitionType, TextView};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// Document view mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DocumentMode {
    View,
    Edit,
}

/// Document viewer with view/edit switching
pub struct DocumentView {
    container: Box,
    stack: Stack,
    mode: Rc<RefCell<DocumentMode>>,
    file_path: Rc<RefCell<Option<PathBuf>>>,
    content: Rc<RefCell<String>>,
    view_label: Label,
    edit_buffer: gtk4::TextBuffer,
}

impl DocumentView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("document-view");

        // Toolbar with mode toggle
        let toolbar = Box::new(Orientation::Horizontal, 8);
        toolbar.set_margin_start(8);
        toolbar.set_margin_end(8);
        toolbar.set_margin_top(4);
        toolbar.set_margin_bottom(4);
        toolbar.add_css_class("toolbar");

        // File path label
        let path_label = Label::new(Some("Untitled"));
        path_label.add_css_class("heading");
        path_label.set_xalign(0.0);
        path_label.set_hexpand(true);
        toolbar.append(&path_label);

        // View/Edit toggle buttons
        let view_btn = Button::with_label("View");
        view_btn.add_css_class("flat");
        view_btn.add_css_class("toggle-active");
        toolbar.append(&view_btn);

        let edit_btn = Button::with_label("Edit");
        edit_btn.add_css_class("flat");
        toolbar.append(&edit_btn);

        // Save button (only visible in edit mode)
        let save_btn = Button::from_icon_name("document-save-symbolic");
        save_btn.set_tooltip_text(Some("Save (Ctrl+S)"));
        save_btn.add_css_class("flat");
        save_btn.set_visible(false);
        toolbar.append(&save_btn);

        container.append(&toolbar);

        // Separator
        let separator = gtk4::Separator::new(Orientation::Horizontal);
        container.append(&separator);

        // Stack for view/edit modes
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(150);
        stack.set_vexpand(true);

        // View mode: read-only label with markdown-like rendering
        let view_scroll = ScrolledWindow::new();
        view_scroll.set_vexpand(true);
        let view_label = Label::new(Some(""));
        view_label.set_wrap(true);
        view_label.set_xalign(0.0);
        view_label.set_yalign(0.0);
        view_label.set_selectable(true);
        view_label.set_margin_start(16);
        view_label.set_margin_end(16);
        view_label.set_margin_top(16);
        view_label.set_margin_bottom(16);
        view_label.add_css_class("document-content");
        view_scroll.set_child(Some(&view_label));
        stack.add_named(&view_scroll, Some("view"));

        // Edit mode: text editor
        let edit_scroll = ScrolledWindow::new();
        edit_scroll.set_vexpand(true);
        let edit_view = TextView::new();
        edit_view.set_monospace(true);
        edit_view.set_wrap_mode(gtk4::WrapMode::WordChar);
        edit_view.set_left_margin(16);
        edit_view.set_right_margin(16);
        edit_view.set_top_margin(16);
        edit_view.set_bottom_margin(16);
        edit_view.add_css_class("document-editor");
        let edit_buffer = edit_view.buffer();
        edit_scroll.set_child(Some(&edit_view));
        stack.add_named(&edit_scroll, Some("edit"));

        container.append(&stack);

        // State
        let mode = Rc::new(RefCell::new(DocumentMode::View));
        let file_path = Rc::new(RefCell::new(None::<PathBuf>));
        let content = Rc::new(RefCell::new(String::new()));

        // Mode toggle handlers
        let mode_for_view = mode.clone();
        let stack_for_view = stack.clone();
        let view_btn_clone = view_btn.clone();
        let edit_btn_clone = edit_btn.clone();
        let save_btn_for_view = save_btn.clone();
        view_btn.connect_clicked(move |_| {
            *mode_for_view.borrow_mut() = DocumentMode::View;
            stack_for_view.set_visible_child_name("view");
            view_btn_clone.add_css_class("toggle-active");
            edit_btn_clone.remove_css_class("toggle-active");
            save_btn_for_view.set_visible(false);
        });

        let mode_for_edit = mode.clone();
        let stack_for_edit = stack.clone();
        let view_btn_clone2 = view_btn.clone();
        let edit_btn_clone2 = edit_btn.clone();
        let save_btn_for_edit = save_btn.clone();
        edit_btn.connect_clicked(move |_| {
            *mode_for_edit.borrow_mut() = DocumentMode::Edit;
            stack_for_edit.set_visible_child_name("edit");
            edit_btn_clone2.add_css_class("toggle-active");
            view_btn_clone2.remove_css_class("toggle-active");
            save_btn_for_edit.set_visible(true);
        });

        // Save handler
        let file_path_for_save = file_path.clone();
        let buffer_for_save = edit_buffer.clone();
        save_btn.connect_clicked(move |_| {
            if let Some(ref path) = *file_path_for_save.borrow() {
                let (start, end) = buffer_for_save.bounds();
                let text = buffer_for_save.text(&start, &end, true);
                if let Err(e) = std::fs::write(path, text.as_str()) {
                    tracing::error!("Failed to save file: {}", e);
                } else {
                    tracing::info!("File saved: {:?}", path);
                }
            }
        });

        Self {
            container,
            stack,
            mode,
            file_path,
            content,
            view_label,
            edit_buffer,
        }
    }

    /// Load a file into the document view
    pub fn load_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(path)?;

        *self.file_path.borrow_mut() = Some(path.to_path_buf());
        *self.content.borrow_mut() = content.clone();

        // Update view mode label
        self.view_label.set_text(&content);

        // Update edit mode buffer
        self.edit_buffer.set_text(&content);

        tracing::info!("Loaded document: {:?}", path);
        Ok(())
    }

    /// Set content directly (for new documents)
    pub fn set_content(&self, text: &str) {
        *self.content.borrow_mut() = text.to_string();
        self.view_label.set_text(text);
        self.edit_buffer.set_text(text);
    }

    /// Get current content
    pub fn get_content(&self) -> String {
        let (start, end) = self.edit_buffer.bounds();
        self.edit_buffer.text(&start, &end, true).to_string()
    }

    /// Switch to view mode
    pub fn set_view_mode(&self) {
        *self.mode.borrow_mut() = DocumentMode::View;
        self.stack.set_visible_child_name("view");

        // Sync content from edit buffer to view label
        let content = self.get_content();
        self.view_label.set_text(&content);
    }

    /// Switch to edit mode
    pub fn set_edit_mode(&self) {
        *self.mode.borrow_mut() = DocumentMode::Edit;
        self.stack.set_visible_child_name("edit");
    }

    /// Get current mode
    pub fn mode(&self) -> DocumentMode {
        *self.mode.borrow()
    }

    /// Get the container widget
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Get file path if set
    pub fn file_path(&self) -> Option<PathBuf> {
        self.file_path.borrow().clone()
    }
}

impl Default for DocumentView {
    fn default() -> Self {
        Self::new()
    }
}
