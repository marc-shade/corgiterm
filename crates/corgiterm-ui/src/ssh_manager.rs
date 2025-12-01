//! SSH Connection Manager
//!
//! Visual management of SSH hosts with support for:
//! - Saved host configurations
//! - Import from ~/.ssh/config
//! - Quick connect with one click
//! - Add/Edit/Delete hosts
//! - Connection history tracking
//! - SSH key management
//! - Port forwarding presets
//! - Jump host / ProxyJump support

use crossbeam_channel;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, DropDown, FileDialog, Label, ListBox, Orientation, ScrolledWindow, SearchEntry,
    SelectionMode, StringList, ToggleButton, Stack, StackSidebar,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, Dialog, EntryRow, PreferencesGroup};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dialogs::get_config;
use corgiterm_config::{parse_ssh_config, SshHost};

/// Port forwarding configuration
#[derive(Debug, Clone, Default)]
pub struct PortForward {
    pub name: String,
    pub forward_type: PortForwardType,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PortForwardType {
    #[default]
    Local,      // -L
    Remote,     // -R
    Dynamic,    // -D (SOCKS proxy)
}

impl PortForwardType {
    fn as_str(&self) -> &'static str {
        match self {
            PortForwardType::Local => "Local (-L)",
            PortForwardType::Remote => "Remote (-R)",
            PortForwardType::Dynamic => "Dynamic (-D)",
        }
    }

    fn from_index(idx: u32) -> Self {
        match idx {
            0 => PortForwardType::Local,
            1 => PortForwardType::Remote,
            2 => PortForwardType::Dynamic,
            _ => PortForwardType::Local,
        }
    }
}

/// Connection history entry
#[derive(Debug, Clone)]
pub struct ConnectionHistory {
    pub host_name: String,
    pub timestamp: u64,
    pub duration_secs: Option<u64>,
    pub success: bool,
}

/// SSH Key info
#[derive(Debug, Clone)]
pub struct SshKeyInfo {
    pub name: String,
    pub path: PathBuf,
    pub key_type: String,
    pub bits: Option<u32>,
    pub fingerprint: String,
    pub has_passphrase: bool,
    pub public_key: String,
}

/// SSH Manager state
struct SshManagerState {
    hosts: Vec<SshHost>,
    history: Vec<ConnectionHistory>,
    port_forwards: HashMap<String, Vec<PortForward>>, // host_name -> forwards
    keys: Vec<SshKeyInfo>,
}

/// SSH Manager widget
pub struct SshManager {
    dialog: Dialog,
    state: Rc<RefCell<SshManagerState>>,
    // Hosts tab
    hosts_list: ListBox,
    search_entry: SearchEntry,
    tag_filter: DropDown,
    favorites_toggle: ToggleButton,
    // History tab
    history_list: ListBox,
    // Keys tab
    keys_list: ListBox,
}

impl SshManager {
    /// Create a new SSH Manager
    pub fn new(_parent: &impl IsA<gtk4::Widget>) -> Self {
        let dialog = Dialog::builder()
            .title("SSH Connection Manager")
            .content_width(850)
            .content_height(600)
            .build();

        // Main container with stack sidebar
        let main_box = Box::new(Orientation::Horizontal, 0);

        let stack = Stack::new();
        stack.set_hexpand(true);

        let sidebar = StackSidebar::new();
        sidebar.set_stack(&stack);
        sidebar.set_width_request(180);
        sidebar.add_css_class("navigation-sidebar");

        main_box.append(&sidebar);
        main_box.append(&stack);

        // Initialize state
        let state = Rc::new(RefCell::new(SshManagerState {
            hosts: Vec::new(),
            history: Vec::new(),
            port_forwards: HashMap::new(),
            keys: Vec::new(),
        }));

        // Load hosts from config
        if let Some(config_manager) = get_config() {
            let config = config_manager.read().config();
            state.borrow_mut().hosts = config.ssh.hosts.clone();
        }

        // Load SSH keys
        Self::scan_ssh_keys(&state);

        // Create tabs
        let (hosts_page, hosts_list, search_entry, tag_filter, favorites_toggle) =
            Self::create_hosts_page(&state, &dialog);
        stack.add_titled(&hosts_page, Some("hosts"), "🖥 Hosts");

        let (history_page, history_list) = Self::create_history_page(&state);
        stack.add_titled(&history_page, Some("history"), "📜 History");

        let (keys_page, keys_list) = Self::create_keys_page(&state, &dialog);
        stack.add_titled(&keys_page, Some("keys"), "🔑 Keys");

        let tunnels_page = Self::create_tunnels_page(&state);
        stack.add_titled(&tunnels_page, Some("tunnels"), "🚇 Tunnels");

        let quick_connect_page = Self::create_quick_connect_page();
        stack.add_titled(&quick_connect_page, Some("quick"), "⚡ Quick Connect");

        dialog.set_child(Some(&main_box));

        let manager = Self {
            dialog,
            state,
            hosts_list,
            search_entry,
            tag_filter,
            favorites_toggle,
            history_list,
            keys_list,
        };

        // Initial populate
        manager.populate_hosts();
        manager.populate_history();

        manager
    }

    /// Show the SSH Manager dialog
    pub fn show(&self, parent: &impl IsA<gtk4::Widget>) {
        self.dialog.present(Some(parent));
    }

    // ==================== HOSTS TAB ====================

    fn create_hosts_page(
        state: &Rc<RefCell<SshManagerState>>,
        dialog: &Dialog,
    ) -> (Box, ListBox, SearchEntry, DropDown, ToggleButton) {
        let page_box = Box::new(Orientation::Vertical, 12);
        page_box.set_margin_start(16);
        page_box.set_margin_end(16);
        page_box.set_margin_top(16);
        page_box.set_margin_bottom(16);

        // Header with search and buttons
        let header_box = Box::new(Orientation::Horizontal, 8);

        let search_entry = SearchEntry::builder()
            .placeholder_text("Search hosts by name, hostname, user, or tag...")
            .hexpand(true)
            .build();
        header_box.append(&search_entry);

        let favorites_toggle = ToggleButton::builder()
            .icon_name("starred-symbolic")
            .tooltip_text("Show favorites only")
            .css_classes(vec!["flat", "circular"])
            .build();
        header_box.append(&favorites_toggle);

        let tag_model = StringList::new(&["All"]);
        let tag_filter = DropDown::builder()
            .model(&tag_model)
            .selected(0)
            .tooltip_text("Filter by tag")
            .build();
        header_box.append(&tag_filter);

        page_box.append(&header_box);

        // Action buttons row
        let action_box = Box::new(Orientation::Horizontal, 8);
        action_box.set_halign(gtk4::Align::End);

        let import_btn = Button::builder()
            .label("Import ~/.ssh/config")
            .css_classes(vec!["pill"])
            .build();
        action_box.append(&import_btn);

        let export_btn = Button::builder()
            .label("Export")
            .tooltip_text("Export hosts to JSON")
            .css_classes(vec!["pill"])
            .build();
        action_box.append(&export_btn);

        let add_btn = Button::builder()
            .label("Add Host")
            .css_classes(vec!["pill", "suggested-action"])
            .build();
        action_box.append(&add_btn);

        page_box.append(&action_box);

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
        page_box.append(&scrolled);

        // Stats row
        let stats_box = Box::new(Orientation::Horizontal, 12);
        stats_box.add_css_class("dim-label");
        let stats_label = Label::new(None);
        stats_box.append(&stats_label);
        page_box.append(&stats_box);

        // Update stats based on hosts
        {
            let state_ref = state.borrow();
            let total = state_ref.hosts.len();
            let favs = state_ref.hosts.iter().filter(|h| h.favorite).count();
            stats_label.set_label(&format!("{} hosts • {} favorites", total, favs));
        }

        // Connect search
        {
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            let favorites_toggle_c = favorites_toggle.clone();
            let tag_filter_c = tag_filter.clone();
            let stats_label_c = stats_label.clone();
            search_entry.connect_search_changed(move |entry| {
                Self::filter_hosts(
                    &hosts_list_c,
                    &state_c,
                    &entry.text(),
                    favorites_toggle_c.is_active(),
                    Self::current_tag(&tag_filter_c),
                    &stats_label_c,
                );
            });
        }

        // Favorites filter
        {
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            let search_entry_c = search_entry.clone();
            let tag_filter_c = tag_filter.clone();
            let stats_label_c = stats_label.clone();
            favorites_toggle.connect_toggled(move |btn| {
                Self::filter_hosts(
                    &hosts_list_c,
                    &state_c,
                    &search_entry_c.text(),
                    btn.is_active(),
                    Self::current_tag(&tag_filter_c),
                    &stats_label_c,
                );
            });
        }

        // Tag filter
        {
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            let search_entry_c = search_entry.clone();
            let favorites_toggle_c = favorites_toggle.clone();
            let stats_label_c = stats_label.clone();
            tag_filter.connect_selected_notify(move |dropdown| {
                Self::filter_hosts(
                    &hosts_list_c,
                    &state_c,
                    &search_entry_c.text(),
                    favorites_toggle_c.is_active(),
                    Self::current_tag(dropdown),
                    &stats_label_c,
                );
            });
        }

        // Connect add button
        {
            let dialog_c = dialog.clone();
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            let tag_filter_c = tag_filter.clone();
            add_btn.connect_clicked(move |_| {
                Self::show_host_editor(
                    &dialog_c,
                    None,
                    &hosts_list_c,
                    &state_c,
                    &tag_filter_c,
                );
            });
        }

        // Connect import button
        {
            let dialog_c = dialog.clone();
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            let favorites_toggle_c = favorites_toggle.clone();
            let tag_filter_c = tag_filter.clone();
            let search_c = search_entry.clone();
            let stats_label_c = stats_label.clone();
            import_btn.connect_clicked(move |_| {
                Self::import_from_ssh_config(
                    &dialog_c,
                    &hosts_list_c,
                    &state_c,
                    &search_c.text(),
                    favorites_toggle_c.is_active(),
                    Self::current_tag(&tag_filter_c),
                    &tag_filter_c,
                    &stats_label_c,
                );
            });
        }

        // Connect export button
        {
            let state_c = state.clone();
            let dialog_c = dialog.clone();
            export_btn.connect_clicked(move |_| {
                Self::export_hosts(&dialog_c, &state_c);
            });
        }

        (page_box, hosts_list, search_entry, tag_filter, favorites_toggle)
    }

    /// Populate the hosts list
    fn populate_hosts(&self) {
        self.refresh_tag_filter();
        Self::filter_hosts(
            &self.hosts_list,
            &self.state,
            &self.search_entry.text(),
            self.favorites_toggle.is_active(),
            Self::current_tag(&self.tag_filter),
            &Label::new(None), // Dummy label for initial load
        );
    }

    fn refresh_tag_filter(&self) {
        if let Some(model) = self.tag_filter.model().and_downcast::<StringList>() {
            while model.n_items() > 0 {
                model.remove(0);
            }
            model.append("All");
            let mut tags: Vec<String> = self
                .state
                .borrow()
                .hosts
                .iter()
                .flat_map(|h| h.tags.clone())
                .collect();
            tags.sort();
            tags.dedup();
            for tag in tags {
                model.append(&tag);
            }
        }
    }

    /// Filter and display hosts based on search query
    fn filter_hosts(
        hosts_list: &ListBox,
        state: &Rc<RefCell<SshManagerState>>,
        query: &str,
        favorites_only: bool,
        tag: Option<String>,
        stats_label: &Label,
    ) {
        // Clear existing rows
        while let Some(child) = hosts_list.first_child() {
            hosts_list.remove(&child);
        }

        let query_lower = query.to_lowercase();
        let state_ref = state.borrow();
        let filtered_hosts: Vec<SshHost> = state_ref
            .hosts
            .iter()
            .filter(|host| {
                (query.is_empty()
                    || host.name.to_lowercase().contains(&query_lower)
                    || host.hostname.to_lowercase().contains(&query_lower)
                    || host
                        .username
                        .as_ref()
                        .map(|u| u.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || host
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower)))
                    && (!favorites_only || host.favorite)
                    && match tag.as_deref() {
                        None => true,
                        Some("All") => true,
                        Some(t) => host.tags.iter().any(|ht| ht.eq_ignore_ascii_case(t)),
                    }
            })
            .cloned()
            .collect();

        // Update stats
        let total = state_ref.hosts.len();
        let showing = filtered_hosts.len();
        let favs = state_ref.hosts.iter().filter(|h| h.favorite).count();
        stats_label.set_label(&format!("Showing {} of {} hosts • {} favorites", showing, total, favs));

        drop(state_ref);

        if filtered_hosts.is_empty() {
            let empty_row = ActionRow::builder()
                .title("No SSH hosts found")
                .subtitle("Add a new host or import from ~/.ssh/config")
                .build();
            hosts_list.append(&empty_row);
        } else {
            for host in filtered_hosts {
                let row = Self::create_host_row(&host, hosts_list, state);
                hosts_list.append(&row);
            }
        }
    }

    fn current_tag(dropdown: &DropDown) -> Option<String> {
        dropdown
            .selected_item()
            .and_then(|obj| obj.downcast::<gtk4::StringObject>().ok())
            .map(|o| o.string().to_string())
    }

    /// Create a row for an SSH host
    fn create_host_row(
        host: &SshHost,
        hosts_list: &ListBox,
        state: &Rc<RefCell<SshManagerState>>,
    ) -> ActionRow {
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
        connect_btn.add_css_class("success");
        connect_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&connect_btn);

        {
            let host_c = host.clone();
            let state_c = state.clone();
            connect_btn.connect_clicked(move |_| {
                Self::connect_to_host(&host_c, &state_c);
            });
        }

        // Copy SSH command button
        let copy_btn = Button::from_icon_name("edit-copy-symbolic");
        copy_btn.set_tooltip_text(Some("Copy SSH command"));
        copy_btn.add_css_class("flat");
        copy_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&copy_btn);

        {
            let host_c = host.clone();
            copy_btn.connect_clicked(move |btn| {
                let cmd = host_c.build_command().join(" ");
                let display = btn.display();
                let clipboard = display.clipboard();
                clipboard.set_text(&cmd);
            });
        }

        // Test button (dry run)
        let test_btn = Button::from_icon_name("network-server-symbolic");
        test_btn.set_tooltip_text(Some("Test connection (config check)"));
        test_btn.add_css_class("flat");
        test_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&test_btn);

        {
            let host_c = host.clone();
            let row_c = row.clone();
            test_btn.connect_clicked(move |_| {
                Self::test_host(&host_c, &row_c);
            });
        }

        // Favorite toggle
        let fav_btn = ToggleButton::builder()
            .icon_name("starred-symbolic")
            .tooltip_text(if host.favorite { "Unfavorite" } else { "Favorite" })
            .css_classes(vec!["flat", "circular"])
            .valign(gtk4::Align::Center)
            .active(host.favorite)
            .build();

        {
            let host_name = host.name.clone();
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            fav_btn.connect_toggled(move |btn| {
                let is_fav = btn.is_active();
                {
                    let mut state_mut = state_c.borrow_mut();
                    for h in state_mut.hosts.iter_mut() {
                        if h.name == host_name {
                            h.favorite = is_fav;
                        }
                    }
                }
                if let Some(cfg) = get_config() {
                    cfg.write().update(|c| {
                        for h in c.ssh.hosts.iter_mut() {
                            if h.name == host_name {
                                h.favorite = is_fav;
                            }
                        }
                    });
                    let _ = cfg.read().save();
                }
                Self::filter_hosts(&hosts_list_c, &state_c, "", false, None, &Label::new(None));
                btn.set_tooltip_text(Some(if is_fav { "Unfavorite" } else { "Favorite" }));
            });
        }
        row.add_suffix(&fav_btn);

        // Edit button
        let edit_btn = Button::from_icon_name("document-edit-symbolic");
        edit_btn.set_tooltip_text(Some("Edit"));
        edit_btn.add_css_class("flat");
        edit_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&edit_btn);

        {
            let host_c = host.clone();
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            edit_btn.connect_clicked(move |btn| {
                if btn.root().is_some() {
                    let mut current = btn.clone().upcast::<gtk4::Widget>();
                    loop {
                        if let Ok(dlg) = current.clone().downcast::<Dialog>() {
                            Self::show_host_editor(
                                &dlg,
                                Some(&host_c),
                                &hosts_list_c,
                                &state_c,
                                &DropDown::new(None::<gtk4::gio::ListModel>, None::<gtk4::Expression>),
                            );
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
        }

        // Delete button
        let delete_btn = Button::from_icon_name("user-trash-symbolic");
        delete_btn.set_tooltip_text(Some("Delete"));
        delete_btn.add_css_class("flat");
        delete_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&delete_btn);

        {
            let host_c = host.clone();
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            delete_btn.connect_clicked(move |_| {
                Self::delete_host(&host_c, &hosts_list_c, &state_c);
            });
        }

        row
    }

    /// Connect to an SSH host
    fn connect_to_host(host: &SshHost, state: &Rc<RefCell<SshManagerState>>) {
        tracing::info!(
            "Connecting to SSH host: {} ({})",
            host.name,
            host.display_string()
        );

        // Record in history
        {
            let mut state_mut = state.borrow_mut();
            state_mut.history.push(ConnectionHistory {
                host_name: host.name.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                duration_secs: None,
                success: true, // Optimistic; could update later
            });
        }

        // Get the terminal tabs instance and create new tab
        if let Some(app) = gtk4::gio::Application::default() {
            if let Ok(gtk_app) = app.downcast::<gtk4::Application>() {
                if let Some(window) = gtk_app.active_window() {
                    // Build SSH command
                    let ssh_cmd = host.build_command();
                    let cmd_string = ssh_cmd.join(" ");

                    window
                        .activate_action("win.ssh-connect", Some(&cmd_string.to_variant()))
                        .ok();
                }
            }
        }
    }

    /// Run a quick, non-interactive test using `ssh -G`
    fn test_host(host: &SshHost, row: &ActionRow) {
        use std::process::Command;
        let target = host.display_string();
        let row_clone = row.clone();
        let (sender, receiver) = crossbeam_channel::unbounded::<Result<bool, anyhow::Error>>();

        std::thread::spawn({
            let host = host.clone();
            let sender = sender.clone();
            move || {
                let mut cmd = host.build_command();
                cmd.push("-G".to_string());
                let status = Command::new(&cmd[0]).args(&cmd[1..]).status();
                sender
                    .send(status.map(|s| s.success()).map_err(|e| anyhow::anyhow!(e)))
                    .ok();
            }
        });

        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            match receiver.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(true) => row_clone.set_subtitle(&format!("{} ✓ config ok", target)),
                        Ok(false) => row_clone.set_subtitle(&format!("{} ✗ config error", target)),
                        Err(e) => row_clone.set_subtitle(&format!("{} ⚠ {}", target, e)),
                    }
                    glib::ControlFlow::Break
                }
                Err(crossbeam_channel::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    row_clone.set_subtitle(&format!("{} ⚠ disconnected", target));
                    glib::ControlFlow::Break
                }
            }
        });
    }

    /// Show host editor dialog
    fn show_host_editor(
        parent: &Dialog,
        host: Option<&SshHost>,
        hosts_list: &ListBox,
        state: &Rc<RefCell<SshManagerState>>,
        tag_filter: &DropDown,
    ) {
        let editor_dialog = Dialog::builder()
            .title(if host.is_some() { "Edit SSH Host" } else { "Add SSH Host" })
            .content_width(550)
            .build();

        let content_box = Box::new(Orientation::Vertical, 12);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);

        // Basic settings group
        let basic_group = PreferencesGroup::builder()
            .title("Basic Settings")
            .build();

        let name_entry = EntryRow::builder()
            .title("Name")
            .text(host.map(|h| h.name.as_str()).unwrap_or(""))
            .build();
        basic_group.add(&name_entry);

        let hostname_entry = EntryRow::builder()
            .title("Hostname")
            .text(host.map(|h| h.hostname.as_str()).unwrap_or(""))
            .build();
        basic_group.add(&hostname_entry);

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
        basic_group.add(&port_row);

        let username_entry = EntryRow::builder()
            .title("Username")
            .text(host.and_then(|h| h.username.as_deref()).unwrap_or(""))
            .build();
        basic_group.add(&username_entry);

        content_box.append(&basic_group);

        // Authentication group
        let auth_group = PreferencesGroup::builder()
            .title("Authentication")
            .build();

        let identity_text = host
            .and_then(|h| h.identity_file.as_ref().map(|p| p.display().to_string()))
            .unwrap_or_default();
        let identity_entry = EntryRow::builder()
            .title("Identity File (Private Key)")
            .text(&identity_text)
            .build();

        let browse_btn = Button::from_icon_name("document-open-symbolic");
        browse_btn.set_valign(gtk4::Align::Center);
        browse_btn.set_tooltip_text(Some("Browse..."));
        identity_entry.add_suffix(&browse_btn);

        {
            let identity_entry_c = identity_entry.clone();
            browse_btn.connect_clicked(move |btn| {
                let file_dialog = FileDialog::builder()
                    .title("Select Identity File")
                    .modal(true)
                    .build();

                let window_parent = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok());

                file_dialog.open(window_parent.as_ref(), None::<&gtk4::gio::Cancellable>, {
                    let identity_entry = identity_entry_c.clone();
                    move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                identity_entry.set_text(&path.display().to_string());
                            }
                        }
                    }
                });
            });
        }

        auth_group.add(&identity_entry);
        content_box.append(&auth_group);

        // Advanced settings (expandable)
        let advanced_group = PreferencesGroup::builder()
            .title("Advanced")
            .build();

        // Jump host / ProxyJump
        let jump_host_entry = EntryRow::builder()
            .title("Jump Host (ProxyJump)")
            .text("")
            .build();
        jump_host_entry.set_tooltip_text(Some("SSH host to jump through (e.g., bastion.example.com)"));
        advanced_group.add(&jump_host_entry);

        // Extra SSH options
        let options_entry = EntryRow::builder()
            .title("Extra SSH Options")
            .text("")
            .build();
        options_entry.set_tooltip_text(Some("Additional ssh options (e.g., -o StrictHostKeyChecking=no)"));
        advanced_group.add(&options_entry);

        content_box.append(&advanced_group);

        // Organization group
        let org_group = PreferencesGroup::builder()
            .title("Organization")
            .build();

        let tags_text = host.map(|h| h.tags.join(", ")).unwrap_or_default();
        let tags_entry = EntryRow::builder()
            .title("Tags")
            .text(&tags_text)
            .build();
        let tags_hint = Label::new(Some("Comma-separated"));
        tags_hint.add_css_class("dim-label");
        tags_entry.add_suffix(&tags_hint);
        org_group.add(&tags_entry);

        content_box.append(&org_group);

        // Buttons
        let button_box = Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::End);
        button_box.set_margin_top(12);

        let cancel_btn = Button::with_label("Cancel");
        cancel_btn.add_css_class("pill");
        {
            let editor_dialog_c = editor_dialog.clone();
            cancel_btn.connect_clicked(move |_| {
                editor_dialog_c.close();
            });
        }
        button_box.append(&cancel_btn);

        let save_btn = Button::with_label("Save");
        save_btn.add_css_class("pill");
        save_btn.add_css_class("suggested-action");

        let original_host = host.cloned();
        {
            let editor_dialog_c = editor_dialog.clone();
            let hosts_list_c = hosts_list.clone();
            let state_c = state.clone();
            let tag_filter_c = tag_filter.clone();
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
                    username: if username.is_empty() {
                        None
                    } else {
                        Some(username.to_string())
                    },
                    identity_file: if identity_text.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(identity_text.as_str()))
                    },
                    options: Vec::new(),
                    tags: tags_text
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    favorite: original_host.as_ref().map(|h| h.favorite).unwrap_or(false),
                };

                // Update state
                {
                    let mut state_mut = state_c.borrow_mut();
                    if let Some(ref original) = original_host {
                        if let Some(pos) = state_mut.hosts.iter().position(|h| h == original) {
                            state_mut.hosts[pos] = new_host.clone();
                        }
                    } else {
                        state_mut.hosts.push(new_host.clone());
                    }
                }

                // Save to config
                if let Some(config_manager) = get_config() {
                    config_manager.write().update(|config| {
                        config.ssh.hosts = state_c.borrow().hosts.clone();
                    });
                    let _ = config_manager.read().save();
                }

                // Refresh tag filter
                if let Some(model) = tag_filter_c.model().and_downcast::<StringList>() {
                    while model.n_items() > 0 {
                        model.remove(0);
                    }
                    model.append("All");
                    let mut tags: Vec<String> = state_c
                        .borrow()
                        .hosts
                        .iter()
                        .flat_map(|h| h.tags.clone())
                        .collect();
                    tags.sort();
                    tags.dedup();
                    for tag in tags {
                        model.append(&tag);
                    }
                }

                Self::filter_hosts(&hosts_list_c, &state_c, "", false, None, &Label::new(None));
                editor_dialog_c.close();
            });
        }
        button_box.append(&save_btn);

        content_box.append(&button_box);
        editor_dialog.set_child(Some(&content_box));
        editor_dialog.present(Some(parent));
    }

    /// Delete an SSH host
    fn delete_host(
        host: &SshHost,
        hosts_list: &ListBox,
        state: &Rc<RefCell<SshManagerState>>,
    ) {
        {
            let mut state_mut = state.borrow_mut();
            if let Some(pos) = state_mut.hosts.iter().position(|h| h == host) {
                state_mut.hosts.remove(pos);
            }
        }

        // Save to config
        if let Some(config_manager) = get_config() {
            config_manager.write().update(|config| {
                config.ssh.hosts = state.borrow().hosts.clone();
            });
            let _ = config_manager.read().save();
        }

        Self::filter_hosts(hosts_list, state, "", false, None, &Label::new(None));
        tracing::info!("Deleted SSH host: {}", host.name);
    }

    /// Import hosts from ~/.ssh/config
    fn import_from_ssh_config(
        _parent: &Dialog,
        hosts_list: &ListBox,
        state: &Rc<RefCell<SshManagerState>>,
        query: &str,
        favorites_only: bool,
        tag: Option<String>,
        tag_filter: &DropDown,
        stats_label: &Label,
    ) {
        let ssh_config_path = dirs::home_dir()
            .map(|h| h.join(".ssh/config"))
            .unwrap_or_else(|| PathBuf::from("~/.ssh/config"));

        if !ssh_config_path.exists() {
            tracing::warn!("No SSH config found at {:?}", ssh_config_path);
            return;
        }

        let imported_hosts =
            std::fs::read_to_string(&ssh_config_path).map(|c| parse_ssh_config(&c, 22));

        match imported_hosts {
            Ok(imported_hosts) => {
                let mut added_or_merged = 0;

                {
                    let mut state_mut = state.borrow_mut();
                    for new_host in imported_hosts {
                        if let Some(existing) = state_mut.hosts.iter_mut().find(|h| {
                            h.hostname == new_host.hostname
                                && h.username == new_host.username
                                && h.port == new_host.port
                        }) {
                            existing.merge_from(&new_host);
                            added_or_merged += 1;
                        } else {
                            state_mut.hosts.push(new_host);
                            added_or_merged += 1;
                        }
                    }
                }

                if let Some(config_manager) = get_config() {
                    config_manager.write().update(|config| {
                        config.ssh.hosts = state.borrow().hosts.clone();
                    });
                    let _ = config_manager.read().save();
                }

                // Refresh tag filter options
                if let Some(model) = tag_filter.model().and_downcast::<StringList>() {
                    while model.n_items() > 0 {
                        model.remove(0);
                    }
                    model.append("All");
                    let mut tags: Vec<String> = state
                        .borrow()
                        .hosts
                        .iter()
                        .flat_map(|h| h.tags.clone())
                        .collect();
                    tags.sort();
                    tags.dedup();
                    for tag_val in tags {
                        model.append(&tag_val);
                    }
                    tag_filter.set_selected(0);
                }

                Self::filter_hosts(hosts_list, state, query, favorites_only, tag, stats_label);

                tracing::info!(
                    "Imported/merged {} SSH hosts from ~/.ssh/config",
                    added_or_merged
                );
            }
            Err(e) => {
                tracing::error!("Failed to import SSH config: {}", e);
            }
        }
    }

    /// Export hosts to JSON
    fn export_hosts(parent: &Dialog, state: &Rc<RefCell<SshManagerState>>) {
        let file_dialog = FileDialog::builder()
            .title("Export SSH Hosts")
            .initial_name("ssh_hosts.json")
            .modal(true)
            .build();

        let state_c = state.clone();
        let parent_widget = parent.clone().upcast::<gtk4::Widget>();
        let window_parent = parent_widget.root().and_then(|r| r.downcast::<gtk4::Window>().ok());

        file_dialog.save(window_parent.as_ref(), None::<&gtk4::gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let hosts = &state_c.borrow().hosts;
                    if let Ok(json) = serde_json::to_string_pretty(hosts) {
                        if let Err(e) = std::fs::write(&path, json) {
                            tracing::error!("Failed to export hosts: {}", e);
                        } else {
                            tracing::info!("Exported {} hosts to {:?}", hosts.len(), path);
                        }
                    }
                }
            }
        });
    }

    // ==================== HISTORY TAB ====================

    fn create_history_page(state: &Rc<RefCell<SshManagerState>>) -> (Box, ListBox) {
        let page_box = Box::new(Orientation::Vertical, 12);
        page_box.set_margin_start(16);
        page_box.set_margin_end(16);
        page_box.set_margin_top(16);
        page_box.set_margin_bottom(16);

        let header = Label::builder()
            .label("Recent SSH Connections")
            .css_classes(vec!["title-3"])
            .halign(gtk4::Align::Start)
            .build();
        page_box.append(&header);

        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(400)
            .build();

        let history_list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(vec!["boxed-list"])
            .build();

        scrolled.set_child(Some(&history_list));
        page_box.append(&scrolled);

        // Clear history button
        let clear_btn = Button::builder()
            .label("Clear History")
            .css_classes(vec!["pill", "destructive-action"])
            .halign(gtk4::Align::End)
            .build();

        {
            let state_c = state.clone();
            let history_list_c = history_list.clone();
            clear_btn.connect_clicked(move |_| {
                state_c.borrow_mut().history.clear();
                while let Some(child) = history_list_c.first_child() {
                    history_list_c.remove(&child);
                }
                let empty_row = ActionRow::builder()
                    .title("No connection history")
                    .subtitle("Your recent SSH connections will appear here")
                    .build();
                history_list_c.append(&empty_row);
            });
        }
        page_box.append(&clear_btn);

        (page_box, history_list)
    }

    fn populate_history(&self) {
        while let Some(child) = self.history_list.first_child() {
            self.history_list.remove(&child);
        }

        let state_ref = self.state.borrow();
        if state_ref.history.is_empty() {
            let empty_row = ActionRow::builder()
                .title("No connection history")
                .subtitle("Your recent SSH connections will appear here")
                .build();
            self.history_list.append(&empty_row);
        } else {
            // Show most recent first
            for entry in state_ref.history.iter().rev().take(50) {
                let time_str = Self::format_timestamp(entry.timestamp);
                let row = ActionRow::builder()
                    .title(&entry.host_name)
                    .subtitle(&time_str)
                    .activatable(true)
                    .build();

                // Status indicator
                let status = if entry.success { "✓" } else { "✗" };
                let status_label = Label::new(Some(status));
                status_label.add_css_class(if entry.success { "success" } else { "error" });
                row.add_prefix(&status_label);

                // Reconnect button
                let reconnect_btn = Button::from_icon_name("media-playback-start-symbolic");
                reconnect_btn.set_tooltip_text(Some("Reconnect"));
                reconnect_btn.add_css_class("flat");
                reconnect_btn.set_valign(gtk4::Align::Center);

                {
                    let host_name = entry.host_name.clone();
                    let state_c = self.state.clone();
                    reconnect_btn.connect_clicked(move |_| {
                        let host_opt = {
                            let state_ref = state_c.borrow();
                            state_ref.hosts.iter().find(|h| h.name == host_name).cloned()
                        };
                        if let Some(host) = host_opt {
                            Self::connect_to_host(&host, &state_c);
                        }
                    });
                }
                row.add_suffix(&reconnect_btn);

                self.history_list.append(&row);
            }
        }
    }

    fn format_timestamp(ts: u64) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let diff = now.saturating_sub(ts);

        if diff < 60 {
            "Just now".to_string()
        } else if diff < 3600 {
            format!("{} minutes ago", diff / 60)
        } else if diff < 86400 {
            format!("{} hours ago", diff / 3600)
        } else {
            format!("{} days ago", diff / 86400)
        }
    }

    // ==================== KEYS TAB ====================

    fn create_keys_page(state: &Rc<RefCell<SshManagerState>>, dialog: &Dialog) -> (Box, ListBox) {
        let page_box = Box::new(Orientation::Vertical, 12);
        page_box.set_margin_start(16);
        page_box.set_margin_end(16);
        page_box.set_margin_top(16);
        page_box.set_margin_bottom(16);

        let header_box = Box::new(Orientation::Horizontal, 8);

        let header = Label::builder()
            .label("SSH Keys")
            .css_classes(vec!["title-3"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        header_box.append(&header);

        let generate_btn = Button::builder()
            .label("Generate New Key")
            .css_classes(vec!["pill", "suggested-action"])
            .build();

        {
            let dialog_c = dialog.clone();
            let state_c = state.clone();
            generate_btn.connect_clicked(move |_| {
                Self::show_generate_key_dialog(&dialog_c, &state_c);
            });
        }
        header_box.append(&generate_btn);

        let refresh_btn = Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Rescan SSH keys"));
        refresh_btn.add_css_class("flat");
        {
            let state_c = state.clone();
            refresh_btn.connect_clicked(move |_| {
                Self::scan_ssh_keys(&state_c);
            });
        }
        header_box.append(&refresh_btn);

        page_box.append(&header_box);

        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(400)
            .build();

        let keys_list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(vec!["boxed-list"])
            .build();

        // Populate keys
        {
            let state_ref = state.borrow();
            if state_ref.keys.is_empty() {
                let empty_row = ActionRow::builder()
                    .title("No SSH keys found")
                    .subtitle("Generate a new key or add existing keys to ~/.ssh/")
                    .build();
                keys_list.append(&empty_row);
            } else {
                for key in &state_ref.keys {
                    let row = Self::create_key_row(key);
                    keys_list.append(&row);
                }
            }
        }

        scrolled.set_child(Some(&keys_list));
        page_box.append(&scrolled);

        // SSH Agent status
        let agent_box = Box::new(Orientation::Horizontal, 8);
        agent_box.add_css_class("dim-label");
        let agent_status = Label::new(Some("SSH Agent: Checking..."));
        agent_box.append(&agent_status);
        page_box.append(&agent_box);

        // Check SSH agent asynchronously
        glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            let status = std::process::Command::new("ssh-add")
                .arg("-l")
                .output()
                .map(|o| {
                    if o.status.success() {
                        let count = String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .count();
                        format!("SSH Agent: {} keys loaded", count)
                    } else {
                        "SSH Agent: No keys loaded".to_string()
                    }
                })
                .unwrap_or_else(|_| "SSH Agent: Not running".to_string());
            agent_status.set_label(&status);
        });

        (page_box, keys_list)
    }

    fn scan_ssh_keys(state: &Rc<RefCell<SshManagerState>>) {
        let ssh_dir = dirs::home_dir()
            .map(|h| h.join(".ssh"))
            .unwrap_or_else(|| PathBuf::from("~/.ssh"));

        let mut keys = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    // Look for public key files
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".pub") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let parts: Vec<&str> = content.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let key_type = parts[0].to_string();
                                    let private_path = path.with_extension("");

                                    // Get fingerprint
                                    let fingerprint = std::process::Command::new("ssh-keygen")
                                        .args(["-lf", path.to_str().unwrap_or("")])
                                        .output()
                                        .map(|o| {
                                            String::from_utf8_lossy(&o.stdout)
                                                .split_whitespace()
                                                .nth(1)
                                                .unwrap_or("")
                                                .to_string()
                                        })
                                        .unwrap_or_default();

                                    keys.push(SshKeyInfo {
                                        name: name.trim_end_matches(".pub").to_string(),
                                        path: private_path,
                                        key_type,
                                        bits: None,
                                        fingerprint,
                                        has_passphrase: false, // Can't easily detect
                                        public_key: content.trim().to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        state.borrow_mut().keys = keys;
    }

    fn create_key_row(key: &SshKeyInfo) -> ActionRow {
        let row = ActionRow::builder()
            .title(&key.name)
            .subtitle(&format!("{} • {}", key.key_type, key.fingerprint))
            .build();

        // Key type icon
        let icon = Label::new(Some("🔑"));
        row.add_prefix(&icon);

        // Copy public key button
        let copy_btn = Button::from_icon_name("edit-copy-symbolic");
        copy_btn.set_tooltip_text(Some("Copy public key"));
        copy_btn.add_css_class("flat");
        copy_btn.set_valign(gtk4::Align::Center);

        {
            let public_key = key.public_key.clone();
            copy_btn.connect_clicked(move |btn| {
                let display = btn.display();
                let clipboard = display.clipboard();
                clipboard.set_text(&public_key);
            });
        }
        row.add_suffix(&copy_btn);

        // Add to agent button
        let add_agent_btn = Button::from_icon_name("list-add-symbolic");
        add_agent_btn.set_tooltip_text(Some("Add to SSH agent"));
        add_agent_btn.add_css_class("flat");
        add_agent_btn.set_valign(gtk4::Align::Center);

        {
            let path = key.path.clone();
            add_agent_btn.connect_clicked(move |_| {
                let _ = std::process::Command::new("ssh-add")
                    .arg(&path)
                    .spawn();
            });
        }
        row.add_suffix(&add_agent_btn);

        // View public key button
        let view_btn = Button::from_icon_name("document-open-symbolic");
        view_btn.set_tooltip_text(Some("View public key"));
        view_btn.add_css_class("flat");
        view_btn.set_valign(gtk4::Align::Center);
        row.add_suffix(&view_btn);

        row
    }

    fn show_generate_key_dialog(parent: &Dialog, _state: &Rc<RefCell<SshManagerState>>) {
        let gen_dialog = Dialog::builder()
            .title("Generate SSH Key")
            .content_width(450)
            .build();

        let content_box = Box::new(Orientation::Vertical, 12);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);

        let prefs_group = PreferencesGroup::new();

        // Key name
        let name_entry = EntryRow::builder()
            .title("Key Name")
            .text("id_ed25519")
            .build();
        prefs_group.add(&name_entry);

        // Key type dropdown
        let type_model = StringList::new(&["ed25519 (recommended)", "rsa", "ecdsa"]);
        let type_dropdown = DropDown::builder()
            .model(&type_model)
            .selected(0)
            .build();
        let type_row = ActionRow::builder()
            .title("Key Type")
            .build();
        type_row.add_suffix(&type_dropdown);
        prefs_group.add(&type_row);

        // Comment - use environment variables instead of whoami crate
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "user".to_string());
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| {
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
                    .map_err(|_| std::env::VarError::NotPresent)
            })
            .unwrap_or_else(|_| "localhost".to_string());
        let comment_entry = EntryRow::builder()
            .title("Comment")
            .text(&format!("{}@{}", username, hostname))
            .build();
        prefs_group.add(&comment_entry);

        // Passphrase
        let passphrase_entry = libadwaita::PasswordEntryRow::builder()
            .title("Passphrase (optional)")
            .build();
        prefs_group.add(&passphrase_entry);

        content_box.append(&prefs_group);

        // Buttons
        let button_box = Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::End);
        button_box.set_margin_top(12);

        let cancel_btn = Button::with_label("Cancel");
        cancel_btn.add_css_class("pill");
        {
            let gen_dialog_c = gen_dialog.clone();
            cancel_btn.connect_clicked(move |_| {
                gen_dialog_c.close();
            });
        }
        button_box.append(&cancel_btn);

        let generate_btn = Button::with_label("Generate");
        generate_btn.add_css_class("pill");
        generate_btn.add_css_class("suggested-action");
        {
            let gen_dialog_c = gen_dialog.clone();
            generate_btn.connect_clicked(move |_| {
                let name = name_entry.text().to_string();
                let key_type = match type_dropdown.selected() {
                    0 => "ed25519",
                    1 => "rsa",
                    2 => "ecdsa",
                    _ => "ed25519",
                };
                let comment = comment_entry.text().to_string();
                let passphrase = passphrase_entry.text().to_string();

                let ssh_dir = dirs::home_dir()
                    .map(|h| h.join(".ssh"))
                    .unwrap_or_else(|| PathBuf::from("~/.ssh"));
                let key_path = ssh_dir.join(&name);

                let mut cmd = std::process::Command::new("ssh-keygen");
                cmd.args(["-t", key_type, "-f"])
                    .arg(&key_path)
                    .args(["-C", &comment]);

                if passphrase.is_empty() {
                    cmd.args(["-N", ""]);
                } else {
                    cmd.args(["-N", &passphrase]);
                }

                match cmd.output() {
                    Ok(output) => {
                        if output.status.success() {
                            tracing::info!("Generated SSH key: {:?}", key_path);
                        } else {
                            tracing::error!("Failed to generate key: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to run ssh-keygen: {}", e);
                    }
                }

                gen_dialog_c.close();
            });
        }
        button_box.append(&generate_btn);

        content_box.append(&button_box);
        gen_dialog.set_child(Some(&content_box));
        gen_dialog.present(Some(parent));
    }

    // ==================== TUNNELS TAB ====================

    fn create_tunnels_page(_state: &Rc<RefCell<SshManagerState>>) -> Box {
        let page_box = Box::new(Orientation::Vertical, 12);
        page_box.set_margin_start(16);
        page_box.set_margin_end(16);
        page_box.set_margin_top(16);
        page_box.set_margin_bottom(16);

        let header_box = Box::new(Orientation::Horizontal, 8);

        let header = Label::builder()
            .label("SSH Tunnels & Port Forwarding")
            .css_classes(vec!["title-3"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        header_box.append(&header);

        let add_tunnel_btn = Button::builder()
            .label("Add Tunnel")
            .css_classes(vec!["pill", "suggested-action"])
            .build();
        header_box.append(&add_tunnel_btn);

        page_box.append(&header_box);

        // Info box
        let info_box = Box::new(Orientation::Vertical, 4);
        info_box.add_css_class("card");
        info_box.set_margin_top(8);
        info_box.set_margin_bottom(8);

        let info_label = Label::builder()
            .label("Configure SSH tunnels for port forwarding. Tunnels can be started when connecting to a host.")
            .wrap(true)
            .halign(gtk4::Align::Start)
            .build();
        info_label.add_css_class("dim-label");
        info_box.append(&info_label);
        page_box.append(&info_box);

        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(300)
            .build();

        let tunnels_list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(vec!["boxed-list"])
            .build();

        // Example tunnels
        let example_tunnels = vec![
            ("Local DB Forward", "Local (-L)", "5432 → localhost:5432"),
            ("Remote Web Access", "Remote (-R)", "8080 → localhost:80"),
            ("SOCKS Proxy", "Dynamic (-D)", "1080"),
        ];

        for (name, fwd_type, desc) in example_tunnels {
            let row = ActionRow::builder()
                .title(name)
                .subtitle(&format!("{} • {}", fwd_type, desc))
                .build();

            let enable_switch = gtk4::Switch::builder()
                .valign(gtk4::Align::Center)
                .build();
            row.add_suffix(&enable_switch);

            let edit_btn = Button::from_icon_name("document-edit-symbolic");
            edit_btn.add_css_class("flat");
            edit_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&edit_btn);

            let delete_btn = Button::from_icon_name("user-trash-symbolic");
            delete_btn.add_css_class("flat");
            delete_btn.set_valign(gtk4::Align::Center);
            row.add_suffix(&delete_btn);

            tunnels_list.append(&row);
        }

        scrolled.set_child(Some(&tunnels_list));
        page_box.append(&scrolled);

        // Tunnel type reference
        let ref_group = PreferencesGroup::builder()
            .title("Tunnel Types")
            .build();

        let local_row = ActionRow::builder()
            .title("Local (-L)")
            .subtitle("Forward local port to remote destination")
            .build();
        ref_group.add(&local_row);

        let remote_row = ActionRow::builder()
            .title("Remote (-R)")
            .subtitle("Forward remote port to local destination")
            .build();
        ref_group.add(&remote_row);

        let dynamic_row = ActionRow::builder()
            .title("Dynamic (-D)")
            .subtitle("SOCKS proxy for dynamic port forwarding")
            .build();
        ref_group.add(&dynamic_row);

        page_box.append(&ref_group);

        page_box
    }

    // ==================== QUICK CONNECT TAB ====================

    fn create_quick_connect_page() -> Box {
        let page_box = Box::new(Orientation::Vertical, 12);
        page_box.set_margin_start(16);
        page_box.set_margin_end(16);
        page_box.set_margin_top(16);
        page_box.set_margin_bottom(16);

        let header = Label::builder()
            .label("Quick Connect")
            .css_classes(vec!["title-3"])
            .halign(gtk4::Align::Start)
            .build();
        page_box.append(&header);

        let subtitle = Label::builder()
            .label("Connect to any SSH host without saving")
            .css_classes(vec!["dim-label"])
            .halign(gtk4::Align::Start)
            .build();
        page_box.append(&subtitle);

        let prefs_group = PreferencesGroup::new();

        // Connection string entry
        let conn_entry = EntryRow::builder()
            .title("SSH Connection")
            .build();
        conn_entry.set_tooltip_text(Some("user@hostname or user@hostname:port"));
        prefs_group.add(&conn_entry);

        // Port (optional)
        let port_adj = gtk4::Adjustment::new(22.0, 1.0, 65535.0, 1.0, 10.0, 0.0);
        let port_row = libadwaita::SpinRow::builder()
            .title("Port")
            .adjustment(&port_adj)
            .build();
        prefs_group.add(&port_row);

        // Identity file (optional)
        let identity_entry = EntryRow::builder()
            .title("Identity File (optional)")
            .build();
        let browse_btn = Button::from_icon_name("document-open-symbolic");
        browse_btn.set_valign(gtk4::Align::Center);
        identity_entry.add_suffix(&browse_btn);
        prefs_group.add(&identity_entry);

        page_box.append(&prefs_group);

        // Connect button
        let connect_btn = Button::builder()
            .label("Connect")
            .css_classes(vec!["pill", "suggested-action"])
            .halign(gtk4::Align::Center)
            .margin_top(24)
            .build();

        {
            let conn_entry_c = conn_entry.clone();
            let port_row_c = port_row.clone();
            let identity_entry_c = identity_entry.clone();
            connect_btn.connect_clicked(move |_| {
                let conn_str = conn_entry_c.text().to_string();
                if conn_str.is_empty() {
                    return;
                }

                // Parse connection string
                let (user_host, port) = if conn_str.contains(':') {
                    let parts: Vec<&str> = conn_str.rsplitn(2, ':').collect();
                    if parts.len() == 2 {
                        (parts[1].to_string(), parts[0].parse().unwrap_or(22))
                    } else {
                        (conn_str.clone(), port_row_c.value() as u16)
                    }
                } else {
                    (conn_str.clone(), port_row_c.value() as u16)
                };

                let (username, hostname) = if user_host.contains('@') {
                    let parts: Vec<&str> = user_host.splitn(2, '@').collect();
                    (Some(parts[0].to_string()), parts[1].to_string())
                } else {
                    (None, user_host)
                };

                let identity = identity_entry_c.text().to_string();
                let identity_file = if identity.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(identity))
                };

                let host = SshHost {
                    name: format!("Quick: {}", hostname),
                    hostname,
                    port,
                    username,
                    identity_file,
                    options: Vec::new(),
                    tags: Vec::new(),
                    favorite: false,
                };

                // Create dummy state for connect_to_host
                let dummy_state = Rc::new(RefCell::new(SshManagerState {
                    hosts: vec![host.clone()],
                    history: Vec::new(),
                    port_forwards: HashMap::new(),
                    keys: Vec::new(),
                }));

                Self::connect_to_host(&host, &dummy_state);
            });
        }
        page_box.append(&connect_btn);

        // Common examples
        let examples_group = PreferencesGroup::builder()
            .title("Examples")
            .margin_top(24)
            .build();

        let examples = vec![
            ("user@example.com", "Basic connection"),
            ("root@192.168.1.1:2222", "Custom port"),
            ("deploy@staging.server.com", "Named server"),
        ];

        for (cmd, desc) in examples {
            let row = ActionRow::builder()
                .title(cmd)
                .subtitle(desc)
                .activatable(true)
                .build();

            let use_btn = Button::from_icon_name("go-next-symbolic");
            use_btn.add_css_class("flat");
            use_btn.set_valign(gtk4::Align::Center);
            {
                let conn_entry_c = conn_entry.clone();
                let cmd = cmd.to_string();
                use_btn.connect_clicked(move |_| {
                    conn_entry_c.set_text(&cmd);
                });
            }
            row.add_suffix(&use_btn);

            examples_group.add(&row);
        }

        page_box.append(&examples_group);

        page_box
    }
}
