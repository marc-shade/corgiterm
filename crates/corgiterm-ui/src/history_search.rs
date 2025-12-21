//! Searchable command history dialog
//!
//! Provides Ctrl+R style fuzzy search through command history

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::history_store;

/// History search popup/dialog
pub struct HistorySearch {
    container: Box,
    entry: Entry,
    results_list: ListBox,
    selected_command: Rc<RefCell<Option<String>>>,
    on_select: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>>,
}

impl HistorySearch {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 8);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.add_css_class("history-search");

        // Header
        let header = Label::new(Some("Search Command History"));
        header.add_css_class("title-4");
        header.set_margin_bottom(8);
        container.append(&header);

        // Search entry
        let entry = Entry::new();
        entry.set_placeholder_text(Some("Type to search..."));
        entry.add_css_class("monospace");
        entry.set_hexpand(true);
        container.append(&entry);

        // Results list
        let results_list = ListBox::new();
        results_list.add_css_class("boxed-list");
        results_list.set_selection_mode(gtk4::SelectionMode::Single);

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&results_list));
        scrolled.set_vexpand(true);
        scrolled.set_min_content_height(300);
        scrolled.set_max_content_height(400);
        container.append(&scrolled);

        // Help text
        let help = Label::new(Some("↑/↓ to navigate • Enter to select • Esc to cancel"));
        help.add_css_class("dim-label");
        help.add_css_class("caption");
        container.append(&help);

        let selected_command = Rc::new(RefCell::new(None));
        let on_select: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>> =
            Rc::new(RefCell::new(None));

        let search = Self {
            container,
            entry,
            results_list,
            selected_command,
            on_select,
        };

        // Wire up search
        search.setup_search();

        // Initial population with recent commands
        search.update_results("");

        search
    }

    fn setup_search(&self) {
        let results_list = self.results_list.clone();
        let selected_cmd = self.selected_command.clone();
        let on_select_cb = self.on_select.clone();

        // Update results on text change
        self.entry.connect_changed({
            let results_list = results_list.clone();
            let selected_cmd = selected_cmd.clone();
            move |entry| {
                let query = entry.text().to_string();
                Self::update_results_static(&results_list, &query, &selected_cmd);
            }
        });

        // Handle selection on Enter
        self.entry.connect_activate({
            let results_list = results_list.clone();
            let on_select_cb = on_select_cb.clone();
            move |entry| {
                if let Some(row) = results_list.selected_row() {
                    if let Some(label) = row
                        .child()
                        .and_then(|c| c.downcast::<Box>().ok())
                        .and_then(|b| b.first_child())
                        .and_then(|c| c.downcast::<Label>().ok())
                    {
                        let cmd = label.text().to_string();
                        let callback = on_select_cb.borrow();
                        if let Some(ref cb) = *callback {
                            cb(&cmd);
                        }
                        // Close the popup (will be handled by parent)
                        if let Some(window) = entry.root().and_then(|r| r.downcast::<Window>().ok())
                        {
                            window.close();
                        }
                    }
                }
            }
        });

        // Handle row activation (click)
        self.results_list.connect_row_activated({
            let on_select_cb = on_select_cb.clone();
            move |_list, row| {
                if let Some(label) = row
                    .child()
                    .and_then(|c| c.downcast::<Box>().ok())
                    .and_then(|b| b.first_child())
                    .and_then(|c| c.downcast::<Label>().ok())
                {
                    let cmd = label.text().to_string();
                    let callback = on_select_cb.borrow();
                    if let Some(ref cb) = *callback {
                        cb(&cmd);
                    }
                    // Close the popup
                    if let Some(window) = row.root().and_then(|r| r.downcast::<Window>().ok()) {
                        window.close();
                    }
                }
            }
        });
    }

    fn update_results(&self, query: &str) {
        Self::update_results_static(&self.results_list, query, &self.selected_command);
    }

    fn update_results_static(
        results_list: &ListBox,
        query: &str,
        selected_cmd: &Rc<RefCell<Option<String>>>,
    ) {
        // Clear existing results
        while let Some(row) = results_list.first_child() {
            results_list.remove(&row);
        }

        if let Some(store) = history_store() {
            let store = store.read();
            let results = store.fuzzy_search(query, 20);

            for (entry, score) in results {
                let row_box = Box::new(Orientation::Vertical, 4);
                row_box.set_margin_top(8);
                row_box.set_margin_bottom(8);
                row_box.set_margin_start(8);
                row_box.set_margin_end(8);

                // Command
                let cmd_label = Label::new(Some(&entry.command));
                cmd_label.add_css_class("monospace");
                cmd_label.set_xalign(0.0);
                cmd_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                row_box.append(&cmd_label);

                // Directory and timestamp info
                let info_text = format!("{} • score: {:.1}", entry.directory, score);
                let info_label = Label::new(Some(&info_text));
                info_label.add_css_class("dim-label");
                info_label.add_css_class("caption");
                info_label.set_xalign(0.0);
                row_box.append(&info_label);

                let row = ListBoxRow::new();
                row.set_child(Some(&row_box));
                results_list.append(&row);
            }

            // Select first result
            if let Some(first) = results_list.row_at_index(0) {
                results_list.select_row(Some(&first));

                // Update selected command
                if let Some(store) = history_store() {
                    let store = store.read();
                    let results = store.fuzzy_search(query, 1);
                    if let Some((entry, _)) = results.first() {
                        *selected_cmd.borrow_mut() = Some(entry.command.clone());
                    }
                }
            }
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set callback when a command is selected
    pub fn set_on_select<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        *self.on_select.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Focus the search entry
    pub fn focus(&self) {
        self.entry.grab_focus();
    }

    /// Get the entry for keyboard handling
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    /// Get selected command
    pub fn selected_command(&self) -> Option<String> {
        self.selected_command.borrow().clone()
    }

    /// Navigate up in the list
    pub fn navigate_up(&self) {
        if let Some(row) = self.results_list.selected_row() {
            let idx = row.index();
            if idx > 0 {
                if let Some(prev) = self.results_list.row_at_index(idx - 1) {
                    self.results_list.select_row(Some(&prev));
                    self.update_selected_from_row(&prev);
                }
            }
        }
    }

    /// Navigate down in the list
    pub fn navigate_down(&self) {
        if let Some(row) = self.results_list.selected_row() {
            let idx = row.index();
            if let Some(next) = self.results_list.row_at_index(idx + 1) {
                self.results_list.select_row(Some(&next));
                self.update_selected_from_row(&next);
            }
        } else if let Some(first) = self.results_list.row_at_index(0) {
            self.results_list.select_row(Some(&first));
            self.update_selected_from_row(&first);
        }
    }

    fn update_selected_from_row(&self, row: &ListBoxRow) {
        if let Some(label) = row
            .child()
            .and_then(|c| c.downcast::<Box>().ok())
            .and_then(|b| b.first_child())
            .and_then(|c| c.downcast::<Label>().ok())
        {
            *self.selected_command.borrow_mut() = Some(label.text().to_string());
        }
    }
}

impl Default for HistorySearch {
    fn default() -> Self {
        Self::new()
    }
}

/// Create and show history search popup
pub fn show_history_search_dialog<F>(parent: &impl IsA<gtk4::Widget>, on_select: F)
where
    F: Fn(&str) + 'static + Clone,
{
    let search = HistorySearch::new();
    search.set_on_select(on_select.clone());

    // Create popup window
    let window = gtk4::Window::builder()
        .title("Command History")
        .modal(true)
        .default_width(600)
        .default_height(450)
        .child(search.widget())
        .build();

    // Set transient for parent window
    if let Some(parent_window) = parent
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
    {
        window.set_transient_for(Some(&parent_window));
    }

    // Handle keyboard navigation
    let search_widget = search.widget().clone();
    let entry = search.entry().clone();
    let on_select_for_key = on_select.clone();

    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed({
        let window = window.clone();
        let _entry = entry.clone();
        move |_, key, _keycode, _state| {
            match key {
                gtk4::gdk::Key::Escape => {
                    window.close();
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Up => {
                    // Navigate up in list
                    if let Some(list) = search_widget
                        .last_child()
                        .and_then(|c| c.first_child())
                        .and_then(|c| c.downcast::<ListBox>().ok())
                    {
                        if let Some(row) = list.selected_row() {
                            let idx = row.index();
                            if idx > 0 {
                                if let Some(prev) = list.row_at_index(idx - 1) {
                                    list.select_row(Some(&prev));
                                }
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Down => {
                    // Navigate down in list
                    if let Some(list) = search_widget
                        .last_child()
                        .and_then(|c| c.first_child())
                        .and_then(|c| c.downcast::<ListBox>().ok())
                    {
                        if let Some(row) = list.selected_row() {
                            if let Some(next) = list.row_at_index(row.index() + 1) {
                                list.select_row(Some(&next));
                            }
                        } else if let Some(first) = list.row_at_index(0) {
                            list.select_row(Some(&first));
                        }
                    }
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Return => {
                    // Select current and close
                    if let Some(list) = search_widget
                        .last_child()
                        .and_then(|c| c.first_child())
                        .and_then(|c| c.downcast::<ListBox>().ok())
                    {
                        if let Some(row) = list.selected_row() {
                            if let Some(label) = row
                                .child()
                                .and_then(|c| c.downcast::<Box>().ok())
                                .and_then(|b| b.first_child())
                                .and_then(|c| c.downcast::<Label>().ok())
                            {
                                let cmd = label.text().to_string();
                                on_select_for_key(&cmd);
                                window.close();
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        }
    });
    window.add_controller(key_controller);

    window.present();
    search.focus();
}
