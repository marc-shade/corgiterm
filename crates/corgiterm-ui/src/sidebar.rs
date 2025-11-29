//! Sidebar with project folders
//!
//! Project sidebar that shows saved project folders. Projects persist
//! across restarts using the SessionManager. Clicking a project folder
//! opens a terminal in that directory.

use gtk4::prelude::*;
use gtk4::{Button, FileDialog, Label, ListBox, Orientation, ScrolledWindow, Separator};
use gtk4::gio;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::app::session_manager;

/// Callback when a project folder is clicked - receives full path
pub type ProjectCallback = Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str, &str)>>>>; // (name, path)
pub type AiActionCallback = Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str)>>>>;

/// Project sidebar widget
pub struct Sidebar {
    container: gtk4::Box,
    project_list: ListBox,
    /// Stored project paths (name -> full path) - synced with SessionManager
    projects: Rc<RefCell<Vec<(String, PathBuf)>>>,
    on_project_click: ProjectCallback,
    on_ai_action: AiActionCallback,
}

impl Sidebar {
    pub fn new() -> Self {
        let container = gtk4::Box::new(Orientation::Vertical, 0);
        container.add_css_class("sidebar");
        container.set_width_request(220);

        // Callbacks
        let on_project_click: ProjectCallback = Rc::new(RefCell::new(None));
        let on_ai_action: AiActionCallback = Rc::new(RefCell::new(None));
        let projects: Rc<RefCell<Vec<(String, PathBuf)>>> = Rc::new(RefCell::new(Vec::new()));

        // Header with "Add Project" button
        let header_box = gtk4::Box::new(Orientation::Horizontal, 0);
        header_box.set_margin_start(12);
        header_box.set_margin_end(8);
        header_box.set_margin_top(12);
        header_box.set_margin_bottom(8);

        let header = Label::new(Some("PROJECTS"));
        header.add_css_class("heading");
        header.set_xalign(0.0);
        header.set_hexpand(true);
        header_box.append(&header);

        let add_btn = Button::from_icon_name("folder-new-symbolic");
        add_btn.set_tooltip_text(Some("Add Project Folder"));
        add_btn.add_css_class("flat");
        header_box.append(&add_btn);
        container.append(&header_box);

        // Project list
        let project_list = ListBox::new();
        project_list.add_css_class("navigation-sidebar");
        project_list.set_selection_mode(gtk4::SelectionMode::Single);

        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&project_list));
        container.append(&scrolled);

        let sidebar = Self {
            container: container.clone(),
            project_list: project_list.clone(),
            projects: projects.clone(),
            on_project_click: on_project_click.clone(),
            on_ai_action: on_ai_action.clone(),
        };

        // Load projects from SessionManager (persisted)
        if let Some(sm) = session_manager() {
            let session_mgr = sm.read();
            for project in session_mgr.projects() {
                sidebar.add_project_folder(&project.path);
            }
            drop(session_mgr);
            tracing::info!("Loaded {} projects from session manager", sidebar.projects.borrow().len());
        }

        // If no saved projects, add default folders
        if sidebar.projects.borrow().is_empty() {
            if let Some(home) = dirs::home_dir() {
                sidebar.add_project_folder(&home);

                // Add ~/projects if it exists
                let projects_dir = home.join("projects");
                if projects_dir.exists() {
                    sidebar.add_project_folder(&projects_dir);
                }
            }
        }

        // Connect add button to folder chooser
        let projects_for_add = projects.clone();
        let list_for_add = project_list.clone();
        let callback_for_add = on_project_click.clone();
        add_btn.connect_clicked(move |btn| {
            let dialog = FileDialog::builder()
                .title("Add Project Folder")
                .modal(true)
                .build();

            let projects = projects_for_add.clone();
            let list = list_for_add.clone();
            let callback = callback_for_add.clone();

            // Get the window from the button
            if let Some(root) = btn.root() {
                if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                    dialog.select_folder(Some(window), None::<&gio::Cancellable>, move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                // Add to projects list
                                let name = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Project")
                                    .to_string();

                                let path_str = path.to_string_lossy().to_string();

                                // Create row
                                let row = libadwaita::ActionRow::builder()
                                    .title(&name)
                                    .subtitle(&path_str)
                                    .activatable(true)
                                    .build();
                                row.add_prefix(&Label::new(Some("📁")));

                                // Connect click
                                let n = name.clone();
                                let p = path_str.clone();
                                let cb = callback.clone();
                                row.connect_activated(move |_| {
                                    if let Some(ref f) = *cb.borrow() {
                                        f(&n, &p);
                                    }
                                });

                                list.append(&row);
                                projects.borrow_mut().push((name, path.clone()));

                                // Persist to SessionManager
                                if let Some(sm) = session_manager() {
                                    let mut session_mgr = sm.write();
                                    session_mgr.open_project(path);
                                    if let Err(e) = session_mgr.save() {
                                        tracing::error!("Failed to save projects: {}", e);
                                    }
                                }

                                tracing::info!("Added project folder: {}", path_str);
                            }
                        }
                    });
                }
            }
        });

        // Separator
        container.append(&Separator::new(Orientation::Horizontal));

        // AI section
        let ai_header = Label::new(Some("AI Assistant"));
        ai_header.add_css_class("heading");
        ai_header.set_xalign(0.0);
        ai_header.set_margin_start(12);
        ai_header.set_margin_top(8);
        ai_header.set_margin_bottom(8);
        container.append(&ai_header);

        // AI quick actions
        let ai_list = ListBox::new();
        ai_list.add_css_class("navigation-sidebar");

        let chat_row = libadwaita::ActionRow::builder()
            .title("Chat")
            .subtitle("Ask anything")
            .activatable(true)
            .build();
        chat_row.add_prefix(&Label::new(Some("💬")));
        ai_list.append(&chat_row);

        let cmd_row = libadwaita::ActionRow::builder()
            .title("Command")
            .subtitle("Describe what to do")
            .activatable(true)
            .build();
        cmd_row.add_prefix(&Label::new(Some("⚡")));
        ai_list.append(&cmd_row);

        // Connect AI action clicks
        let on_ai_for_list = on_ai_action.clone();
        ai_list.connect_row_activated(move |_, row| {
            if let Some(ref callback) = *on_ai_for_list.borrow() {
                let index = row.index();
                let action = match index {
                    0 => "chat",
                    1 => "command",
                    _ => "unknown",
                };
                callback(action);
            }
        });

        container.append(&ai_list);

        sidebar
    }

    /// Add a project folder to the sidebar
    fn add_project_folder(&self, path: &PathBuf) {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Folder")
            .to_string();

        let path_str = path.to_string_lossy().to_string();

        let row = libadwaita::ActionRow::builder()
            .title(&name)
            .subtitle(&path_str)
            .activatable(true)
            .build();
        row.add_prefix(&Label::new(Some("📁")));

        // Connect click handler
        let n = name.clone();
        let p = path_str.clone();
        let callback = self.on_project_click.clone();
        row.connect_activated(move |_| {
            if let Some(ref cb) = *callback.borrow() {
                cb(&n, &p);
            }
        });

        self.project_list.append(&row);
        self.projects.borrow_mut().push((name, path.clone()));
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    /// Set callback for project folder clicks - receives (name, path)
    pub fn set_on_session_click<F>(&self, callback: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        *self.on_project_click.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Set callback for AI actions
    pub fn set_on_ai_action<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        *self.on_ai_action.borrow_mut() = Some(std::boxed::Box::new(callback));
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}
