//! SSH Connection Manager
//!
//! Visual management of SSH hosts with support for:
//! - Saved host configurations
//! - Import from ~/.ssh/config
//! - Quick connect with one click
//! - Add/Edit/Delete hosts

use gtk4::prelude::*;
use gtk4::{Box, Button, FileDialog, Label, ListBox, Orientation, ScrolledWindow, SearchEntry, SelectionMode};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, Dialog, EntryRow, PreferencesGroup};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use corgiterm_config::SshHost;
use crate::dialogs::get_config;

/// SSH Manager widget
pub struct SshManager {
    dialog: Dialog,
    hosts_list: ListBox,
    search_entry: SearchEntry,
    all_hosts: Rc<RefCell<Vec<SshHost>>>,
}

impl SshManager {
    /// Create a new SSH Manager
    pub fn new(_parent: &impl IsA<gtk4::Widget>) -> Self {
        let dialog = Dialog::builder()
            .title("SSH Connection Manager")
            .content_width(700)
            .content_height(500)
            .build();

        // Main container
        let main_box = Box::new(Orientation::Vertical, 16);
        main_box.set_margin_start(24);
        main_box.set_margin_end(24);
        main_box.set_margin_top(24);
        main_box.set_margin_bottom(24);

        // Header with search and buttons
        let header_box = Box::new(Orientation::Horizontal, 8);

        let search_entry = SearchEntry::builder()
            .placeholder_text("Search SSH hosts...")
            .hexpand(true)
            .build();
        header_box.append(&search_entry);

        let import_btn = Button::with_label("Import from ~/.ssh/config");
        import_btn.add_css_class("pill");
        header_box.append(&import_btn);

        let add_btn = Button::with_label("Add Host");
        add_btn.add_css_class("pill");
        add_btn.add_css_class("suggested-action");
        header_box.append(&add_btn);

        main_box.append(&header_box);

        // Scrolled hosts list
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(350)
            .build();

        let hosts_list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(vec!["boxed-list"])
            .build();

        scrolled.set_child(Some(&hosts_list));
        main_box.append(&scrolled);

        dialog.set_child(Some(&main_box));

        // Load hosts from config
        let all_hosts = Rc::new(RefCell::new(Vec::new()));
        if let Some(config_manager) = get_config() {
            let config = config_manager.read().config();
            *all_hosts.borrow_mut() = config.ssh.hosts.clone();
        }

        let manager = Self {
            dialog,
            hosts_list: hosts_list.clone(),
            search_entry: search_entry.clone(),
            all_hosts: all_hosts.clone(),
        };

        // Connect search
        let hosts_list_for_search = hosts_list.clone();
        let all_hosts_for_search = all_hosts.clone();
        search_entry.connect_search_changed(move |entry| {
            Self::filter_hosts(&hosts_list_for_search, &all_hosts_for_search, &entry.text());
        });

        // Connect add button
        let dialog_for_add = manager.dialog.clone();
        let hosts_list_for_add = hosts_list.clone();
        let all_hosts_for_add = all_hosts.clone();
        add_btn.connect_clicked(move |_| {
            Self::show_host_editor(&dialog_for_add, None, &hosts_list_for_add, &all_hosts_for_add);
        });

        // Connect import button
        let dialog_for_import = manager.dialog.clone();
        let hosts_list_for_import = hosts_list.clone();
        let all_hosts_for_import = all_hosts.clone();
        import_btn.connect_clicked(move |_| {
            Self::import_from_ssh_config(&dialog_for_import, &hosts_list_for_import, &all_hosts_for_import);
        });

        // Initial populate
        manager.populate_hosts();

        manager
    }

    /// Show the SSH Manager dialog
    pub fn show(&self, parent: &impl IsA<gtk4::Widget>) {
        self.dialog.present(Some(parent));
    }

    /// Populate the hosts list
    fn populate_hosts(&self) {
        Self::filter_hosts(&self.hosts_list, &self.all_hosts, &self.search_entry.text());
    }

    /// Filter and display hosts based on search query
    fn filter_hosts(hosts_list: &ListBox, all_hosts: &Rc<RefCell<Vec<SshHost>>>, query: &str) {
        // Clear existing rows
        while let Some(child) = hosts_list.first_child() {
            hosts_list.remove(&child);
        }

        let query_lower = query.to_lowercase();
        let filtered_hosts: Vec<SshHost> = all_hosts
            .borrow()
            .iter()
            .filter(|host| {
                query.is_empty()
                    || host.name.to_lowercase().contains(&query_lower)
                    || host.hostname.to_lowercase().contains(&query_lower)
                    || host.username.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false)
                    || host.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        if filtered_hosts.is_empty() {
            let empty_row = ActionRow::builder()
                .title("No SSH hosts found")
                .subtitle("Add a new host or import from ~/.ssh/config")
                .build();
            hosts_list.append(&empty_row);
        } else {
            for host in filtered_hosts {
                let row = Self::create_host_row(&host, hosts_list, all_hosts);
                hosts_list.append(&row);
            }
        }
    }

    /// Create a row for an SSH host
    fn create_host_row(host: &SshHost, hosts_list: &ListBox, all_hosts: &Rc<RefCell<Vec<SshHost>>>) -> ActionRow {
        let row = ActionRow::builder()
            .title(&host.name)
            .subtitle(&host.display_string())
            .activatable(true)
            .build();

        // Add tags as subtitle suffix
        if !host.tags.is_empty() {
            let tags_label = Label::new(Some(&format!("🏷 {}", host.tags.join(", "))));
            tags_label.add_css_class("dim-label");
            tags_label.add_css_class("caption");
            row.add_suffix(&tags_label);
        }

        // Connect button
        let connect_btn = Button::from_icon_name("media-playback-start-symbolic");
        connect_btn.set_tooltip_text(Some("Connect"));
        connect_btn.add_css_class("flat");
        connect_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&connect_btn);

        let host_for_connect = host.clone();
        connect_btn.connect_clicked(move |_| {
            Self::connect_to_host(&host_for_connect);
        });

        // Edit button
        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_tooltip_text(Some("Edit"));
        edit_btn.add_css_class("flat");
        edit_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&edit_btn);

        let host_for_edit = host.clone();
        let hosts_list_for_edit = hosts_list.clone();
        let all_hosts_for_edit = all_hosts.clone();
        edit_btn.connect_clicked(move |btn| {
            // Get the main SSH manager dialog - we need to traverse up to find it
            // For now, we'll create a new dialog context
            if btn.root().is_some() {
                // Try to get an adw::Dialog from the widget hierarchy
                // Since we can't downcast Root to Dialog directly, we'll use a workaround
                // by finding the parent dialog widget
                let mut current = btn.clone().upcast::<gtk4::Widget>();
                loop {
                    if let Ok(dlg) = current.clone().downcast::<Dialog>() {
                        Self::show_host_editor(&dlg, Some(&host_for_edit), &hosts_list_for_edit, &all_hosts_for_edit);
                        break;
                    }
                    if let Some(parent) = current.parent() {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        });

        // Delete button
        let delete_btn = Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_tooltip_text(Some("Delete"));
        delete_btn.add_css_class("flat");
        delete_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&delete_btn);

        let host_for_delete = host.clone();
        let hosts_list_for_delete = hosts_list.clone();
        let all_hosts_for_delete = all_hosts.clone();
        delete_btn.connect_clicked(move |_| {
            Self::delete_host(&host_for_delete, &hosts_list_for_delete, &all_hosts_for_delete);
        });

        row
    }

    /// Connect to an SSH host
    fn connect_to_host(host: &SshHost) {
        tracing::info!("Connecting to SSH host: {} ({})", host.name, host.display_string());

        // Get the terminal tabs instance and create new tab
        if let Some(app) = gtk4::gio::Application::default() {
            if let Ok(gtk_app) = app.downcast::<gtk4::Application>() {
            if let Some(window) = gtk_app.active_window() {
                // Build SSH command
                let ssh_cmd = host.build_command();
                let cmd_string = ssh_cmd.join(" ");

                // Create new terminal tab with SSH command
                // This will be handled by the tab bar integration
                tracing::info!("SSH command: {}", cmd_string);

                // For now, we'll trigger this via a custom action
                // The actual terminal spawning will be handled by the window/tab_bar
                window.activate_action("win.ssh-connect", Some(&cmd_string.to_variant())).ok();
            }
            }
        }
    }

    /// Show host editor dialog
    fn show_host_editor(
        parent: &Dialog,
        host: Option<&SshHost>,
        hosts_list: &ListBox,
        all_hosts: &Rc<RefCell<Vec<SshHost>>>,
    ) {
        let editor_dialog = Dialog::builder()
            .title(if host.is_some() { "Edit SSH Host" } else { "Add SSH Host" })
            .content_width(500)
            .build();

        let content_box = Box::new(Orientation::Vertical, 12);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);

        let prefs_group = PreferencesGroup::new();

        // Name field
        let name_entry = EntryRow::builder()
            .title("Name")
            .text(host.map(|h| h.name.as_str()).unwrap_or(""))
            .build();
        prefs_group.add(&name_entry);

        // Hostname field
        let hostname_entry = EntryRow::builder()
            .title("Hostname")
            .text(host.map(|h| h.hostname.as_str()).unwrap_or(""))
            .build();
        prefs_group.add(&hostname_entry);

        // Port field
        let port_adj = gtk4::Adjustment::new(
            host.map(|h| h.port as f64).unwrap_or(22.0),
            1.0,
            65535.0,
            1.0,
            10.0,
            0.0,
        );
        let port_row = libadwaita::SpinRow::builder()
            .title("Port")
            .adjustment(&port_adj)
            .build();
        prefs_group.add(&port_row);

        // Username field
        let username_entry = EntryRow::builder()
            .title("Username")
            .text(host.and_then(|h| h.username.as_deref()).unwrap_or(""))
            .build();
        prefs_group.add(&username_entry);

        // Identity file field
        let identity_text = host.and_then(|h| h.identity_file.as_ref().map(|p| p.display().to_string())).unwrap_or_default();
        let identity_entry = EntryRow::builder()
            .title("Identity File (Private Key)")
            .text(&identity_text)
            .build();

        let browse_btn = Button::from_icon_name("document-open-symbolic");
        browse_btn.set_valign(gtk4::Align::Center);
        browse_btn.set_tooltip_text(Some("Browse..."));
        identity_entry.add_suffix(&browse_btn);

        let identity_entry_for_browse = identity_entry.clone();
        let _editor_dialog_for_browse = editor_dialog.clone();
        browse_btn.connect_clicked(move |btn| {
            let file_dialog = FileDialog::builder()
                .title("Select Identity File")
                .modal(true)
                .build();

            // Get the window parent - FileDialog needs a gtk4::Window, not an adw::Dialog
            let window_parent = if let Some(root) = btn.root() {
                root.downcast::<gtk4::Window>().ok()
            } else {
                None
            };

            file_dialog.open(window_parent.as_ref(), None::<&gtk4::gio::Cancellable>, {
                let identity_entry = identity_entry_for_browse.clone();
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            identity_entry.set_text(&path.display().to_string());
                        }
                    }
                }
            });
        });

        prefs_group.add(&identity_entry);

        // Tags field
        let tags_text = host.map(|h| h.tags.join(", ")).unwrap_or_default();
        let tags_entry = EntryRow::builder()
            .title("Tags")
            .text(&tags_text)
            .build();
        tags_entry.add_suffix(&Label::new(Some("Comma-separated")));
        prefs_group.add(&tags_entry);

        content_box.append(&prefs_group);

        // Buttons
        let button_box = Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::End);
        button_box.set_margin_top(12);

        let cancel_btn = Button::with_label("Cancel");
        cancel_btn.add_css_class("pill");
        let editor_dialog_for_cancel = editor_dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            editor_dialog_for_cancel.close();
        });
        button_box.append(&cancel_btn);

        let save_btn = Button::with_label("Save");
        save_btn.add_css_class("pill");
        save_btn.add_css_class("suggested-action");

        let original_host = host.map(|h| h.clone());
        let editor_dialog_for_save = editor_dialog.clone();
        let hosts_list_for_save = hosts_list.clone();
        let all_hosts_for_save = all_hosts.clone();
        save_btn.connect_clicked(move |_| {
            let name = name_entry.text().to_string();
            let hostname = hostname_entry.text().to_string();

            if name.is_empty() || hostname.is_empty() {
                tracing::warn!("Name and hostname are required");
                return;
            }

            let username = username_entry.text();
            let identity_text = identity_entry.text();
            let tags_text = tags_entry.text();

            let new_host = SshHost {
                name,
                hostname,
                port: port_row.value() as u16,
                username: if username.is_empty() { None } else { Some(username.to_string()) },
                identity_file: if identity_text.is_empty() { None } else { Some(PathBuf::from(identity_text.as_str())) },
                options: Vec::new(),
                tags: tags_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            };

            // Update or add host
            let mut hosts = all_hosts_for_save.borrow_mut();
            if let Some(ref original) = original_host {
                if let Some(pos) = hosts.iter().position(|h| h == original) {
                    hosts[pos] = new_host;
                }
            } else {
                hosts.push(new_host);
            }
            drop(hosts);

            // Save to config
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ssh.hosts = all_hosts_for_save.borrow().clone();
                });
                let _ = config_manager.read().save();
            }

            // Refresh list
            Self::filter_hosts(&hosts_list_for_save, &all_hosts_for_save, "");

            editor_dialog_for_save.close();
        });
        button_box.append(&save_btn);

        content_box.append(&button_box);
        editor_dialog.set_child(Some(&content_box));

        // adw::Dialog needs to be presented with a widget, not necessarily a Window
        // Use the parent dialog's widget as the presentation parent
        editor_dialog.present(Some(parent));
    }

    /// Delete an SSH host
    fn delete_host(host: &SshHost, hosts_list: &ListBox, all_hosts: &Rc<RefCell<Vec<SshHost>>>) {
        let mut hosts = all_hosts.borrow_mut();
        if let Some(pos) = hosts.iter().position(|h| h == host) {
            hosts.remove(pos);
            drop(hosts);

            // Save to config
            if let Some(config_manager) = get_config() {
                config_manager.read().update(|config| {
                    config.ssh.hosts = all_hosts.borrow().clone();
                });
                let _ = config_manager.read().save();
            }

            // Refresh list
            Self::filter_hosts(hosts_list, all_hosts, "");

            tracing::info!("Deleted SSH host: {}", host.name);
        }
    }

    /// Import hosts from ~/.ssh/config
    fn import_from_ssh_config(_parent: &Dialog, hosts_list: &ListBox, all_hosts: &Rc<RefCell<Vec<SshHost>>>) {
        let ssh_config_path = dirs::home_dir()
            .map(|h| h.join(".ssh/config"))
            .unwrap_or_else(|| PathBuf::from("~/.ssh/config"));

        if !ssh_config_path.exists() {
            tracing::warn!("No SSH config found at {:?}", ssh_config_path);
            return;
        }

        match Self::parse_ssh_config(&ssh_config_path) {
            Ok(imported_hosts) => {
                let mut hosts = all_hosts.borrow_mut();
                let mut added_count = 0;

                for new_host in imported_hosts {
                    // Skip if host with same name already exists
                    if !hosts.iter().any(|h| h.name == new_host.name) {
                        hosts.push(new_host);
                        added_count += 1;
                    }
                }
                drop(hosts);

                // Save to config
                if let Some(config_manager) = get_config() {
                    config_manager.read().update(|config| {
                        config.ssh.hosts = all_hosts.borrow().clone();
                    });
                    let _ = config_manager.read().save();
                }

                // Refresh list
                Self::filter_hosts(hosts_list, all_hosts, "");

                tracing::info!("Imported {} SSH hosts from ~/.ssh/config", added_count);
            }
            Err(e) => {
                tracing::error!("Failed to import SSH config: {}", e);
            }
        }
    }

    /// Parse SSH config file
    fn parse_ssh_config(path: &PathBuf) -> anyhow::Result<Vec<SshHost>> {
        let content = std::fs::read_to_string(path)?;
        let mut hosts = Vec::new();
        let mut current_host: Option<SshHost> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let key = parts[0].to_lowercase();
            let value = parts[1..].join(" ");

            match key.as_str() {
                "host" => {
                    // Save previous host
                    if let Some(host) = current_host.take() {
                        if !host.hostname.is_empty() {
                            hosts.push(host);
                        }
                    }

                    // Start new host
                    current_host = Some(SshHost {
                        name: value.clone(),
                        hostname: value,
                        port: 22,
                        username: None,
                        identity_file: None,
                        options: Vec::new(),
                        tags: vec!["imported".to_string()],
                    });
                }
                "hostname" => {
                    if let Some(ref mut host) = current_host {
                        host.hostname = value;
                    }
                }
                "port" => {
                    if let Some(ref mut host) = current_host {
                        if let Ok(port) = value.parse::<u16>() {
                            host.port = port;
                        }
                    }
                }
                "user" => {
                    if let Some(ref mut host) = current_host {
                        host.username = Some(value);
                    }
                }
                "identityfile" => {
                    if let Some(ref mut host) = current_host {
                        let expanded_path = if value.starts_with("~/") {
                            if let Some(home) = dirs::home_dir() {
                                home.join(&value[2..])
                            } else {
                                PathBuf::from(&value)
                            }
                        } else {
                            PathBuf::from(&value)
                        };
                        host.identity_file = Some(expanded_path);
                    }
                }
                _ => {
                    // Store other options
                    if let Some(ref mut host) = current_host {
                        host.options.push(format!("-o {}={}", key, value));
                    }
                }
            }
        }

        // Save last host
        if let Some(host) = current_host {
            if !host.hostname.is_empty() {
                hosts.push(host);
            }
        }

        Ok(hosts)
    }
}
