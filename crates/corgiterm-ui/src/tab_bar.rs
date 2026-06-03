//! Tab management using libadwaita TabView

use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita::{TabBar, TabPage, TabView};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::document_view::DocumentView;
use crate::split_pane::{SplitDirection, SplitPane};

/// Type of tab content
pub enum TabContent {
    Terminal(SplitPane),
    Document(DocumentView),
}

impl TabContent {
    /// Get the widget for this tab content
    pub fn widget(&self) -> gtk4::Widget {
        match self {
            TabContent::Terminal(sp) => sp.widget().clone().upcast(),
            TabContent::Document(dv) => dv.widget().clone().upcast(),
        }
    }

    /// Get as split pane if this is a terminal tab
    pub fn as_split_pane(&self) -> Option<&SplitPane> {
        match self {
            TabContent::Terminal(sp) => Some(sp),
            _ => None,
        }
    }

    /// Send command to terminal if this is a terminal tab
    pub fn send_command(&self, command: &str) -> bool {
        if let TabContent::Terminal(sp) = self {
            sp.send_command(command)
        } else {
            false
        }
    }

    /// Send text (bytes) to terminal without newline
    pub fn send_text(&self, text: &str) -> bool {
        if let TabContent::Terminal(sp) = self {
            sp.send_bytes(text.as_bytes());
            true
        } else {
            false
        }
    }

    /// Split the current pane horizontally
    pub fn split_horizontal(&self) {
        if let TabContent::Terminal(sp) = self {
            sp.split(SplitDirection::Horizontal);
        }
    }

    /// Split the current pane vertically
    pub fn split_vertical(&self) {
        if let TabContent::Terminal(sp) = self {
            sp.split(SplitDirection::Vertical);
        }
    }
}

struct TabEntry {
    title: String,
    scope: String,
    content: TabContent,
    page: TabPage,
    visible: bool,
}

/// Tab manager with libadwaita TabView
pub struct TerminalTabs {
    tab_view: TabView,
    parking_tab_view: TabView,
    tab_bar: TabBar,
    entries: Rc<RefCell<Vec<TabEntry>>>,
    visible_indices: Rc<RefCell<Vec<usize>>>,
    active_scope: Rc<RefCell<String>>,
}

impl Clone for TerminalTabs {
    fn clone(&self) -> Self {
        Self {
            tab_view: self.tab_view.clone(),
            parking_tab_view: self.parking_tab_view.clone(),
            tab_bar: self.tab_bar.clone(),
            entries: self.entries.clone(),
            visible_indices: self.visible_indices.clone(),
            active_scope: self.active_scope.clone(),
        }
    }
}

impl TerminalTabs {
    pub fn new() -> Self {
        // Create TabView for content
        let tab_view = TabView::new();
        let parking_tab_view = TabView::new();

        // Create TabBar header
        let tab_bar = TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        tab_bar.set_autohide(false);
        tab_bar.set_expand_tabs(false);

        let entries = Rc::new(RefCell::new(Vec::new()));
        let visible_indices = Rc::new(RefCell::new(Vec::new()));

        let tabs = Self {
            tab_view,
            parking_tab_view,
            tab_bar,
            entries,
            visible_indices,
            active_scope: Rc::new(RefCell::new(default_scope_key())),
        };

        tabs.connect_tab_close_handler();

        // Add initial terminal tab
        tabs.add_terminal_tab("Terminal", None);

        // Broadcast mode action (toggle via TabView action)
        let tabs_clone = tabs.clone();
        let action = gio::SimpleAction::new("broadcast-toggle", None);
        action.connect_activate(move |_, _| {
            tabs_clone.toggle_broadcast_on_active();
        });
        if let Some(app) = gtk4::gio::Application::default() {
            app.add_action(&action);
        }

        tabs
    }

    fn connect_tab_close_handler(&self) {
        let entries_for_close = self.entries.clone();
        let visible_indices_for_close = self.visible_indices.clone();

        self.tab_view.connect_close_page(move |view, page| {
            rebuild_visible_indices_for(view, &entries_for_close, &visible_indices_for_close);

            if visible_indices_for_close.borrow().len() <= 1 {
                return glib::Propagation::Stop;
            }

            {
                let mut entries = entries_for_close.borrow_mut();
                if let Some(idx) = entries
                    .iter()
                    .position(|entry| entry.visible && same_page(&entry.page, page))
                {
                    entries.remove(idx);
                }
            }

            let view_for_idle = view.clone();
            let entries_for_idle = entries_for_close.clone();
            let visible_indices_for_idle = visible_indices_for_close.clone();
            glib::idle_add_local_once(move || {
                rebuild_visible_indices_for(
                    &view_for_idle,
                    &entries_for_idle,
                    &visible_indices_for_idle,
                );
            });

            glib::Propagation::Proceed
        });

        let entries_for_reorder = self.entries.clone();
        let visible_indices_for_reorder = self.visible_indices.clone();
        self.tab_view.connect_page_reordered(move |view, _, _| {
            rebuild_visible_indices_for(view, &entries_for_reorder, &visible_indices_for_reorder);
        });
    }

    pub fn toggle_broadcast_on_active(&self) {
        if let Some(page) = self.tab_view.selected_page() {
            if let Some(idx) = self.current_content() {
                if let Some(entry) = self.entries.borrow_mut().get_mut(idx) {
                    if let TabContent::Terminal(sp) = &entry.content {
                        let enabled = sp.toggle_broadcast();
                        if enabled {
                            page.set_title(&format!("{} (broadcast)", page.title()));
                        } else {
                            // Strip suffix if present
                            let title = page.title();
                            let cleaned = title.replace(" (broadcast)", "");
                            page.set_title(&cleaned);
                        }
                        entry.title = page.title().to_string();
                    }
                }
            }
        }
    }

    /// Get the active location scope for new tabs.
    pub fn active_scope(&self) -> String {
        self.active_scope.borrow().clone()
    }

    /// Change the visible tab scope to a project/location.
    pub fn set_active_scope(&self, scope: &str) {
        let next_scope = normalize_scope(scope);
        let changed = {
            let mut active_scope = self.active_scope.borrow_mut();
            if *active_scope == next_scope {
                false
            } else {
                *active_scope = next_scope;
                true
            }
        };

        if changed {
            self.sync_visible_pages();
        } else {
            self.rebuild_visible_indices();
        }
    }

    /// Select an existing terminal for a location, or create one if none exists.
    pub fn select_or_create_terminal_for_scope(&self, title: &str, working_dir: &str) -> TabPage {
        let scope = normalize_scope(working_dir);
        self.set_active_scope(&scope);

        if let Some(page) = self.terminal_page_for_scope(&scope) {
            self.tab_view.set_selected_page(&page);
            return page;
        }

        self.add_terminal_tab(title, Some(working_dir))
    }

    fn terminal_page_for_scope(&self, scope: &str) -> Option<TabPage> {
        self.entries.borrow().iter().find_map(|entry| {
            if entry.scope == scope && matches!(entry.content, TabContent::Terminal(_)) {
                Some(entry.page.clone())
            } else {
                None
            }
        })
    }

    fn sync_visible_pages(&self) {
        let active_scope = self.active_scope();

        {
            let mut entries = self.entries.borrow_mut();
            for entry in entries.iter_mut() {
                let should_be_visible = entry.scope == active_scope;

                match (entry.visible, should_be_visible) {
                    (true, false) => {
                        let target_position = self.parking_tab_view.n_pages();
                        self.tab_view.transfer_page(
                            &entry.page,
                            &self.parking_tab_view,
                            target_position,
                        );
                        entry.visible = false;
                    }
                    (false, true) => {
                        let target_position = self.tab_view.n_pages();
                        self.parking_tab_view.transfer_page(
                            &entry.page,
                            &self.tab_view,
                            target_position,
                        );
                        entry.visible = true;
                    }
                    _ => {}
                }
            }
        }

        self.rebuild_visible_indices();

        if self.tab_view.n_pages() > 0 && self.tab_view.selected_page().is_none() {
            let first_page = self.tab_view.nth_page(0);
            self.tab_view.set_selected_page(&first_page);
        }
    }

    fn rebuild_visible_indices(&self) {
        rebuild_visible_indices_for(&self.tab_view, &self.entries, &self.visible_indices);
    }

    /// Add a new terminal tab
    pub fn add_terminal_tab(&self, title: &str, working_dir: Option<&str>) -> TabPage {
        let split_pane = if let Some(dir) = working_dir {
            SplitPane::with_working_dir(Some(std::path::Path::new(dir)))
        } else {
            SplitPane::new()
        };
        let widget = split_pane.widget().clone();
        let page = self.tab_view.append(&widget);
        page.set_title(title);
        page.set_icon(Some(&gtk4::gio::ThemedIcon::new(
            "utilities-terminal-symbolic",
        )));

        self.entries.borrow_mut().push(TabEntry {
            title: title.to_string(),
            scope: self.active_scope(),
            content: TabContent::Terminal(split_pane),
            page: page.clone(),
            visible: true,
        });

        self.rebuild_visible_indices();

        // Select the new tab
        self.tab_view.set_selected_page(&page);

        page
    }

    /// Add a new document tab
    pub fn add_document_tab(&self, title: &str, file_path: Option<&PathBuf>) -> TabPage {
        let document = DocumentView::new();

        // Load file if provided
        if let Some(path) = file_path {
            if let Err(e) = document.load_file(path) {
                tracing::error!("Failed to load document: {}", e);
            }
        }

        let widget = document.widget().clone();
        let page = self.tab_view.append(&widget);
        page.set_title(title);
        page.set_icon(Some(&gtk4::gio::ThemedIcon::new("text-x-generic-symbolic")));

        self.entries.borrow_mut().push(TabEntry {
            title: title.to_string(),
            scope: self.active_scope(),
            content: TabContent::Document(document),
            page: page.clone(),
            visible: true,
        });

        self.rebuild_visible_indices();

        // Select the new tab
        self.tab_view.set_selected_page(&page);

        page
    }

    /// Get the tab bar widget (for the header)
    pub fn tab_bar_widget(&self) -> &TabBar {
        &self.tab_bar
    }

    /// Get the tab view widget (for content)
    pub fn tab_view_widget(&self) -> &TabView {
        &self.tab_view
    }

    /// Get number of visible tabs in the active location
    pub fn tab_count(&self) -> i32 {
        self.tab_view.n_pages()
    }

    /// Close the currently selected tab
    pub fn close_current_tab(&self) {
        self.rebuild_visible_indices();

        if self.visible_indices.borrow().len() <= 1 {
            return;
        }

        if let Some(page) = self.tab_view.selected_page() {
            self.tab_view.close_page(&page);
        }
    }

    /// Get the logical content index for the current visible tab position
    pub fn current_content(&self) -> Option<usize> {
        self.rebuild_visible_indices();

        let page = self.tab_view.selected_page()?;
        let position = self.tab_view.page_position(&page);
        if position < 0 {
            return None;
        }

        self.visible_indices
            .borrow()
            .get(position as usize)
            .copied()
    }

    /// Switch to the next tab (wraps around to first tab)
    pub fn select_next_tab(&self) {
        let n_pages = self.tab_view.n_pages();
        if n_pages <= 1 {
            return;
        }

        if let Some(current_page) = self.tab_view.selected_page() {
            let current_pos = self.tab_view.page_position(&current_page);
            let next_pos = (current_pos + 1) % n_pages;
            let next_page = self.tab_view.nth_page(next_pos);
            self.tab_view.set_selected_page(&next_page);
        }
    }

    /// Switch to the previous tab (wraps around to last tab)
    pub fn select_previous_tab(&self) {
        let n_pages = self.tab_view.n_pages();
        if n_pages <= 1 {
            return;
        }

        if let Some(current_page) = self.tab_view.selected_page() {
            let current_pos = self.tab_view.page_position(&current_page);
            let prev_pos = if current_pos == 0 {
                n_pages - 1
            } else {
                current_pos - 1
            };
            let prev_page = self.tab_view.nth_page(prev_pos);
            self.tab_view.set_selected_page(&prev_page);
        }
    }

    /// Switch to tab at specific index (0-based)
    /// If index is out of bounds, does nothing
    pub fn select_tab_by_index(&self, index: usize) {
        let n_pages = self.tab_view.n_pages() as usize;
        if index >= n_pages {
            return;
        }

        let page = self.tab_view.nth_page(index as i32);
        self.tab_view.set_selected_page(&page);
    }

    /// Send a command to the currently selected terminal tab
    /// Returns true if command was sent, false if not a terminal tab
    pub fn send_command_to_current(&self, command: &str) -> bool {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                return entry.content.send_command(command);
            }
        }
        false
    }

    /// Send text to the currently selected terminal tab without newline
    pub fn send_text_to_current(&self, text: &str) -> bool {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                return entry.content.send_text(text);
            }
        }
        false
    }

    /// Access current split pane for direct operations
    pub fn with_current_split_pane<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&SplitPane) -> R,
    {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                if let Some(sp) = entry.content.as_split_pane() {
                    return Some(f(sp));
                }
            }
        }
        None
    }

    /// Split the current pane horizontally
    pub fn split_current_horizontal(&self) {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                entry.content.split_horizontal();
            }
        }
    }

    /// Split the current pane vertically
    pub fn split_current_vertical(&self) {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                entry.content.split_vertical();
            }
        }
    }

    /// Close the currently focused pane
    pub fn close_focused_pane(&self) {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                if let Some(sp) = entry.content.as_split_pane() {
                    sp.close_focused();
                }
            }
        }
    }

    /// Focus the next pane in the current tab
    pub fn focus_next_pane(&self) {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                if let Some(sp) = entry.content.as_split_pane() {
                    sp.focus_next();
                }
            }
        }
    }

    /// Focus the previous pane in the current tab
    pub fn focus_prev_pane(&self) {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                if let Some(sp) = entry.content.as_split_pane() {
                    sp.focus_prev();
                }
            }
        }
    }

    /// Get visible lines from current terminal (for thumbnails)
    pub fn get_current_visible_lines(&self, max_lines: usize) -> Vec<String> {
        if let Some(idx) = self.current_content() {
            let entries = self.entries.borrow();
            if let Some(entry) = entries.get(idx) {
                if let Some(sp) = entry.content.as_split_pane() {
                    return sp.get_visible_lines(max_lines);
                }
            }
        }
        Vec::new()
    }

    /// Update tab titles based on current working directory
    /// This should be called periodically to keep titles in sync
    pub fn update_tab_titles(&self) {
        let mut entries = self.entries.borrow_mut();
        for entry in entries.iter_mut() {
            if let TabContent::Terminal(sp) = &entry.content {
                let dir_name = sp.current_directory_name();

                // Only update if the title has actually changed
                if entry.title != dir_name {
                    entry.title = dir_name.clone();
                    entry.page.set_title(&dir_name);
                }
            }
        }
    }

    /// Queue redraw on all terminal panes (for theme changes)
    pub fn queue_redraw_all_terminals(&self) {
        let entries = self.entries.borrow();
        for entry in entries.iter() {
            if let TabContent::Terminal(sp) = &entry.content {
                sp.queue_redraw_all();
            }
        }
    }
}

impl Default for TerminalTabs {
    fn default() -> Self {
        Self::new()
    }
}

fn default_scope_key() -> String {
    dirs::home_dir()
        .map(|path| path.to_string_lossy().to_string())
        .or_else(|| std::env::var("HOME").ok())
        .unwrap_or_else(|| "default".to_string())
}

fn normalize_scope(scope: &str) -> String {
    let trimmed = scope.trim();
    if trimmed.is_empty() {
        default_scope_key()
    } else {
        trimmed.to_string()
    }
}

fn same_page(left: &TabPage, right: &TabPage) -> bool {
    left == right
}

fn rebuild_visible_indices_for(
    tab_view: &TabView,
    entries: &Rc<RefCell<Vec<TabEntry>>>,
    visible_indices: &Rc<RefCell<Vec<usize>>>,
) {
    let entries = entries.borrow();
    let mut next_indices = Vec::new();

    for position in 0..tab_view.n_pages() {
        let page = tab_view.nth_page(position);
        if let Some((idx, _)) = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.visible && same_page(&entry.page, &page))
        {
            next_indices.push(idx);
        }
    }

    *visible_indices.borrow_mut() = next_indices;
}
