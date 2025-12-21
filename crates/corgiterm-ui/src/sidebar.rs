//! Sidebar with project folders and file shortcuts
//!
//! Project sidebar that shows saved project folders and file shortcuts.
//! Projects persist across restarts using the SessionManager.
//! Clicking a project folder opens a terminal in that directory.
//! Clicking a file shortcut opens it in a document editor tab.

use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{Button, FileDialog, Label, ListBox, Orientation, ScrolledWindow};
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::app::session_manager;

/// Callback when a project folder is clicked - receives (name, path)
pub type ProjectCallback = Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str, &str)>>>>;

/// Callback when a file shortcut is clicked - receives (name, path)
pub type FileCallback = Rc<RefCell<Option<std::boxed::Box<dyn Fn(&str, &str)>>>>;

/// Project sidebar widget with Projects and Files sections
pub struct Sidebar {
    container: gtk4::Box,
    project_list: ListBox,
    file_list: ListBox,
    /// Stored project paths (name -> full path) - synced with SessionManager
    projects: Rc<RefCell<Vec<(String, PathBuf)>>>,
    /// Stored file shortcuts (name -> full path)
    files: Rc<RefCell<Vec<(String, PathBuf)>>>,
    on_project_click: ProjectCallback,
    on_file_click: FileCallback,
}

impl Sidebar {
    pub fn new() -> Self {
        let container = gtk4::Box::new(Orientation::Vertical, 0);
        container.add_css_class("sidebar");
        container.set_width_request(220);
        // Add padding to prevent clipping at edges
        container.set_margin_start(4);
        container.set_margin_end(4);

        // Callbacks
        let on_project_click: ProjectCallback = Rc::new(RefCell::new(None));
        let on_file_click: FileCallback = Rc::new(RefCell::new(None));
        let projects: Rc<RefCell<Vec<(String, PathBuf)>>> = Rc::new(RefCell::new(Vec::new()));
        let files: Rc<RefCell<Vec<(String, PathBuf)>>> = Rc::new(RefCell::new(Vec::new()));

        // ========== PROJECTS SECTION ==========
        let project_header_box = gtk4::Box::new(Orientation::Horizontal, 0);
        project_header_box.set_margin_start(12);
        project_header_box.set_margin_end(8);
        project_header_box.set_margin_top(12);
        project_header_box.set_margin_bottom(8);

        let project_header = Label::new(Some("PROJECTS"));
        project_header.add_css_class("heading");
        project_header.set_xalign(0.0);
        project_header.set_hexpand(true);
        project_header_box.append(&project_header);

        let add_project_btn = Button::from_icon_name("folder-new-symbolic");
        add_project_btn.set_tooltip_text(Some("Add Project Folder"));
        add_project_btn.add_css_class("flat");
        project_header_box.append(&add_project_btn);
        container.append(&project_header_box);

        // Project list
        let project_list = ListBox::new();
        project_list.add_css_class("navigation-sidebar");
        project_list.set_selection_mode(gtk4::SelectionMode::Single);

        let project_scrolled = ScrolledWindow::new();
        project_scrolled.set_vexpand(true);
        project_scrolled.set_min_content_height(150);
        project_scrolled.set_child(Some(&project_list));
        container.append(&project_scrolled);

        // ========== FILES SECTION ==========
        let file_header_box = gtk4::Box::new(Orientation::Horizontal, 0);
        file_header_box.set_margin_start(12);
        file_header_box.set_margin_end(8);
        file_header_box.set_margin_top(12);
        file_header_box.set_margin_bottom(8);

        let file_header = Label::new(Some("FILES"));
        file_header.add_css_class("heading");
        file_header.set_xalign(0.0);
        file_header.set_hexpand(true);
        file_header_box.append(&file_header);

        let add_file_btn = Button::from_icon_name("document-new-symbolic");
        add_file_btn.set_tooltip_text(Some("Add File Shortcut"));
        add_file_btn.add_css_class("flat");
        file_header_box.append(&add_file_btn);
        container.append(&file_header_box);

        // File list
        let file_list = ListBox::new();
        file_list.add_css_class("navigation-sidebar");
        file_list.set_selection_mode(gtk4::SelectionMode::Single);

        let file_scrolled = ScrolledWindow::new();
        file_scrolled.set_vexpand(true);
        file_scrolled.set_min_content_height(150);
        file_scrolled.set_child(Some(&file_list));
        container.append(&file_scrolled);

        let sidebar = Self {
            container: container.clone(),
            project_list: project_list.clone(),
            file_list: file_list.clone(),
            projects: projects.clone(),
            files: files.clone(),
            on_project_click: on_project_click.clone(),
            on_file_click: on_file_click.clone(),
        };

        // Load projects from SessionManager (persisted)
        if let Some(sm) = session_manager() {
            let session_mgr = sm.read();
            for project in session_mgr.projects() {
                sidebar.add_project_folder(&project.path);
            }
            drop(session_mgr);
            tracing::info!(
                "Loaded {} projects from session manager",
                sidebar.projects.borrow().len()
            );
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

        // Connect add project button to folder chooser
        let projects_for_add = projects.clone();
        let list_for_add = project_list.clone();
        let callback_for_add = on_project_click.clone();
        add_project_btn.connect_clicked(move |btn| {
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
                                let name = path
                                    .file_name()
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

        // Connect add file button to file chooser
        let files_for_add = files.clone();
        let file_list_for_add = file_list.clone();
        let file_callback_for_add = on_file_click.clone();
        add_file_btn.connect_clicked(move |btn| {
            let dialog = FileDialog::builder()
                .title("Add File Shortcut")
                .modal(true)
                .build();

            let files = files_for_add.clone();
            let list = file_list_for_add.clone();
            let callback = file_callback_for_add.clone();

            // Get the window from the button
            if let Some(root) = btn.root() {
                if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                    dialog.open(Some(window), None::<&gio::Cancellable>, move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                // Get file name for display
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("File")
                                    .to_string();

                                let path_str = path.to_string_lossy().to_string();

                                // Get file icon based on extension
                                let icon = get_file_icon(&path);

                                // Create row
                                let row = libadwaita::ActionRow::builder()
                                    .title(&name)
                                    .subtitle(&path_str)
                                    .activatable(true)
                                    .build();
                                row.add_prefix(&Label::new(Some(icon)));

                                // Add remove button
                                let remove_btn = Button::from_icon_name("user-trash-symbolic");
                                remove_btn.set_tooltip_text(Some("Remove shortcut"));
                                remove_btn.add_css_class("flat");
                                remove_btn.set_valign(gtk4::Align::Center);

                                let files_for_remove = files.clone();
                                let list_for_remove = list.clone();
                                let path_for_remove = path.clone();
                                let row_weak = row.downgrade();
                                remove_btn.connect_clicked(move |_| {
                                    // Remove from files list
                                    files_for_remove
                                        .borrow_mut()
                                        .retain(|(_, p)| p != &path_for_remove);

                                    // Remove row from list
                                    if let Some(row) = row_weak.upgrade() {
                                        list_for_remove.remove(&row);
                                    }

                                    // Persist removal to SessionManager
                                    if let Some(sm) = session_manager() {
                                        let mut session_mgr = sm.write();
                                        session_mgr.remove_file_shortcut(&path_for_remove);
                                        if let Err(e) = session_mgr.save() {
                                            tracing::error!("Failed to save file shortcuts: {}", e);
                                        }
                                    }

                                    tracing::info!("Removed file shortcut: {:?}", path_for_remove);
                                });
                                row.add_suffix(&remove_btn);

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
                                files.borrow_mut().push((name, path.clone()));

                                // Persist to SessionManager
                                if let Some(sm) = session_manager() {
                                    let mut session_mgr = sm.write();
                                    session_mgr.add_file_shortcut(path);
                                    if let Err(e) = session_mgr.save() {
                                        tracing::error!("Failed to save file shortcuts: {}", e);
                                    }
                                }

                                tracing::info!("Added file shortcut: {}", path_str);
                            }
                        }
                    });
                }
            }
        });

        // Load file shortcuts from SessionManager
        if let Some(sm) = session_manager() {
            let session_mgr = sm.read();
            for file_path in session_mgr.file_shortcuts() {
                sidebar.add_file_shortcut(file_path);
            }
            drop(session_mgr);
            tracing::info!(
                "Loaded {} file shortcuts from session manager",
                sidebar.files.borrow().len()
            );
        }

        sidebar
    }

    /// Add a project folder to the sidebar
    fn add_project_folder(&self, path: &PathBuf) {
        let name = path
            .file_name()
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

    /// Set callback for file shortcut clicks - receives (name, path)
    pub fn set_on_file_click<F>(&self, callback: F)
    where
        F: Fn(&str, &str) + 'static,
    {
        *self.on_file_click.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Add a file shortcut to the sidebar (used for loading from persistence)
    fn add_file_shortcut(&self, path: &PathBuf) {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("File")
            .to_string();

        let path_str = path.to_string_lossy().to_string();
        let icon = get_file_icon(path);

        let row = libadwaita::ActionRow::builder()
            .title(&name)
            .subtitle(&path_str)
            .activatable(true)
            .build();
        row.add_prefix(&Label::new(Some(icon)));

        // Add remove button
        let remove_btn = Button::from_icon_name("user-trash-symbolic");
        remove_btn.set_tooltip_text(Some("Remove shortcut"));
        remove_btn.add_css_class("flat");
        remove_btn.set_valign(gtk4::Align::Center);

        let files_for_remove = self.files.clone();
        let list_for_remove = self.file_list.clone();
        let path_for_remove = path.clone();
        let row_weak = row.downgrade();
        remove_btn.connect_clicked(move |_| {
            // Remove from files list
            files_for_remove
                .borrow_mut()
                .retain(|(_, p)| p != &path_for_remove);

            // Remove row from list
            if let Some(row) = row_weak.upgrade() {
                list_for_remove.remove(&row);
            }

            // Persist removal to SessionManager
            if let Some(sm) = session_manager() {
                let mut session_mgr = sm.write();
                session_mgr.remove_file_shortcut(&path_for_remove);
                if let Err(e) = session_mgr.save() {
                    tracing::error!("Failed to save file shortcuts: {}", e);
                }
            }

            tracing::info!("Removed file shortcut: {:?}", path_for_remove);
        });
        row.add_suffix(&remove_btn);

        // Connect click handler
        let n = name.clone();
        let p = path_str.clone();
        let callback = self.on_file_click.clone();
        row.connect_activated(move |_| {
            if let Some(ref cb) = *callback.borrow() {
                cb(&n, &p);
            }
        });

        self.file_list.append(&row);
        self.files.borrow_mut().push((name, path.clone()));
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

/// Get icon for file based on extension
fn get_file_icon(path: &PathBuf) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // Markdown and documentation
        "md" | "markdown" | "mdx" => "\u{1F4DD}", // memo
        "txt" | "text" => "\u{1F4C4}",            // page facing up
        "rst" | "adoc" => "\u{1F4D6}",            // book

        // Code files
        "rs" => "\u{1F980}",                           // crab (Rust)
        "py" => "\u{1F40D}",                           // snake (Python)
        "js" | "mjs" | "cjs" => "\u{1F7E1}",           // yellow circle (JavaScript)
        "ts" | "tsx" => "\u{1F535}",                   // blue circle (TypeScript)
        "go" => "\u{1F439}",                           // hamster (Go gopher)
        "rb" => "\u{1F48E}",                           // gem (Ruby)
        "c" | "h" => "\u{1F1E8}",                      // regional C
        "cpp" | "cc" | "cxx" | "hpp" => "\u{2795}",    // plus (C++)
        "java" => "\u{2615}",                          // coffee (Java)
        "sh" | "bash" | "zsh" | "fish" => "\u{1F41A}", // shell

        // Config files
        "toml" | "yaml" | "yml" | "json" | "xml" => "\u{2699}\u{FE0F}", // gear
        "ini" | "cfg" | "conf" => "\u{1F527}",                          // wrench

        // Web files
        "html" | "htm" => "\u{1F310}",                   // globe
        "css" | "scss" | "sass" | "less" => "\u{1F3A8}", // palette
        "vue" | "svelte" => "\u{1F49A}",                 // green heart

        // Data files
        "csv" | "tsv" => "\u{1F4CA}",        // bar chart
        "sql" | "db" => "\u{1F5C3}\u{FE0F}", // file cabinet

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" => "\u{1F5BC}\u{FE0F}", // framed picture

        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "\u{1F4E6}", // package

        // Executables and binaries
        "exe" | "bin" | "so" | "dylib" | "dll" => "\u{2699}\u{FE0F}", // gear

        // Default for unknown
        _ => "\u{1F4C4}", // page facing up
    }
}
