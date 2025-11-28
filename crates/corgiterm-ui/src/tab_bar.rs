//! Tab management using libadwaita TabView

use gtk4::prelude::*;
use gtk4::glib;
use libadwaita::prelude::*;
use libadwaita::{TabBar, TabPage, TabView};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::terminal_view::TerminalView;
use crate::document_view::DocumentView;

/// Type of tab content
pub enum TabContent {
    Terminal(TerminalView),
    Document(DocumentView),
}

impl TabContent {
    /// Get the widget for this tab content
    pub fn widget(&self) -> gtk4::Widget {
        match self {
            TabContent::Terminal(tv) => tv.widget().clone().upcast(),
            TabContent::Document(dv) => dv.widget().clone().upcast(),
        }
    }
}

/// Tab manager with libadwaita TabView
pub struct TerminalTabs {
    tab_view: TabView,
    tab_bar: TabBar,
    contents: Rc<RefCell<Vec<TabContent>>>,
}

impl TerminalTabs {
    pub fn new() -> Self {
        // Create TabView for content
        let tab_view = TabView::new();

        // Create TabBar header
        let tab_bar = TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        tab_bar.set_autohide(false);
        tab_bar.set_expand_tabs(false);

        let contents = Rc::new(RefCell::new(Vec::new()));

        let tabs = Self {
            tab_view,
            tab_bar,
            contents,
        };

        // Add initial terminal tab
        tabs.add_terminal_tab("Terminal", None);

        // Connect drag-out handler (disabled for now)
        tabs.tab_view.connect_create_window(move |_| {
            None
        });

        tabs
    }

    /// Add a new terminal tab
    pub fn add_terminal_tab(&self, title: &str, working_dir: Option<&str>) -> TabPage {
        let terminal = if let Some(dir) = working_dir {
            TerminalView::with_working_dir(Some(std::path::Path::new(dir)))
        } else {
            TerminalView::new()
        };
        let widget = terminal.widget().clone();

        // Store content
        self.contents.borrow_mut().push(TabContent::Terminal(terminal));

        // Add to tab view
        let page = self.tab_view.append(&widget);
        page.set_title(title);
        page.set_icon(Some(&gtk4::gio::ThemedIcon::new("utilities-terminal-symbolic")));

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

        // Store content
        self.contents.borrow_mut().push(TabContent::Document(document));

        // Add to tab view
        let page = self.tab_view.append(&widget);
        page.set_title(title);
        page.set_icon(Some(&gtk4::gio::ThemedIcon::new("text-x-generic-symbolic")));

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

    /// Get number of tabs
    pub fn tab_count(&self) -> i32 {
        self.tab_view.n_pages()
    }

    /// Close the currently selected tab
    pub fn close_current_tab(&self) {
        if let Some(page) = self.tab_view.selected_page() {
            if self.tab_view.n_pages() > 1 {
                // Find and remove the content
                let position = self.tab_view.page_position(&page);
                if position >= 0 {
                    let idx = position as usize;
                    if idx < self.contents.borrow().len() {
                        self.contents.borrow_mut().remove(idx);
                    }
                }
                self.tab_view.close_page(&page);
            }
        }
    }

    /// Get the content at the current tab position
    pub fn current_content(&self) -> Option<usize> {
        self.tab_view.selected_page().map(|page| {
            self.tab_view.page_position(&page) as usize
        })
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
}

impl Default for TerminalTabs {
    fn default() -> Self {
        Self::new()
    }
}
