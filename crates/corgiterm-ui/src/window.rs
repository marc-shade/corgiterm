//! Main application window

use gtk4::prelude::*;
use gtk4::{Application, Box, Button, EventControllerKey, FileDialog, FileFilter, Orientation, Paned};
use gtk4::gdk::ModifierType;
use gtk4::gio;
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, HeaderBar, WindowTitle};
use std::rc::Rc;

use crate::sidebar::Sidebar;
use crate::tab_bar::TerminalTabs;

/// Main application window
pub struct MainWindow {
    window: ApplicationWindow,
    tabs: Rc<TerminalTabs>,
    #[allow(dead_code)]
    sidebar: Rc<Sidebar>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        // Create window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("CorgiTerm")
            .default_width(1200)
            .default_height(800)
            .build();

        // Create components
        let sidebar = Rc::new(Sidebar::new());
        let tabs = Rc::new(TerminalTabs::new());

        // Create header bar
        let header = HeaderBar::new();

        // Window title
        let title = WindowTitle::new("CorgiTerm", "~/");
        header.set_title_widget(Some(&title));

        // New tab button
        let new_tab_btn = Button::from_icon_name("tab-new-symbolic");
        new_tab_btn.set_tooltip_text(Some("New Tab (Ctrl+T)"));
        let tabs_for_btn = tabs.clone();
        new_tab_btn.connect_clicked(move |_| {
            tabs_for_btn.add_terminal_tab("Terminal", None);
        });
        header.pack_start(&new_tab_btn);

        // Menu button
        let menu_btn = Button::from_icon_name("open-menu-symbolic");
        menu_btn.set_tooltip_text(Some("Menu"));
        header.pack_end(&menu_btn);

        // Main layout with header + content
        let main_box = Box::new(Orientation::Vertical, 0);

        // Header bar with tab bar integrated
        let header_box = Box::new(Orientation::Vertical, 0);
        header_box.append(&header);
        header_box.append(tabs.tab_bar_widget());
        main_box.append(&header_box);

        // Content: sidebar + tab view
        let content_paned = Paned::new(Orientation::Horizontal);
        content_paned.set_start_child(Some(sidebar.widget()));
        content_paned.set_end_child(Some(tabs.tab_view_widget()));
        content_paned.set_position(220);
        content_paned.set_shrink_start_child(false);
        content_paned.set_shrink_end_child(false);
        content_paned.set_vexpand(true);

        main_box.append(&content_paned);

        window.set_content(Some(&main_box));

        // Connect sidebar project folder clicks to tab creation
        let tabs_for_session = tabs.clone();
        sidebar.set_on_session_click(move |name, path| {
            // Open terminal in the selected folder
            tabs_for_session.add_terminal_tab(name, Some(path));
            tracing::info!("Opened terminal in: {}", path);
        });

        // Connect sidebar AI actions
        let tabs_for_ai = tabs.clone();
        sidebar.set_on_ai_action(move |action| {
            match action {
                "chat" => {
                    tabs_for_ai.add_document_tab("AI Chat", None);
                    tracing::info!("Opening AI chat");
                }
                "command" => {
                    tracing::info!("Opening AI command mode");
                }
                _ => {}
            }
        });

        // Set up keyboard shortcuts
        let key_controller = EventControllerKey::new();
        let tabs_for_keys = tabs.clone();
        let window_for_keys = window.clone();
        key_controller.connect_key_pressed(move |_, key, _keycode, modifier| {
            use gtk4::gdk::Key;

            let ctrl = modifier.contains(ModifierType::CONTROL_MASK);
            let shift = modifier.contains(ModifierType::SHIFT_MASK);

            if ctrl && !shift {
                match key {
                    Key::t | Key::T => {
                        // Ctrl+T: New tab
                        tabs_for_keys.add_terminal_tab("Terminal", None);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::w | Key::W => {
                        // Ctrl+W: Close tab
                        tabs_for_keys.close_current_tab();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::q | Key::Q => {
                        // Ctrl+Q: Quit
                        window_for_keys.close();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::plus | Key::equal => {
                        // Ctrl++: Zoom in (TODO)
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::minus => {
                        // Ctrl+-: Zoom out (TODO)
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_0 => {
                        // Ctrl+0: Switch to last tab
                        let n_tabs = tabs_for_keys.tab_count() as usize;
                        if n_tabs > 0 {
                            tabs_for_keys.select_tab_by_index(n_tabs - 1);
                        }
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_1 => {
                        // Ctrl+1: Switch to tab 1
                        tabs_for_keys.select_tab_by_index(0);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_2 => {
                        // Ctrl+2: Switch to tab 2
                        tabs_for_keys.select_tab_by_index(1);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_3 => {
                        // Ctrl+3: Switch to tab 3
                        tabs_for_keys.select_tab_by_index(2);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_4 => {
                        // Ctrl+4: Switch to tab 4
                        tabs_for_keys.select_tab_by_index(3);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_5 => {
                        // Ctrl+5: Switch to tab 5
                        tabs_for_keys.select_tab_by_index(4);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_6 => {
                        // Ctrl+6: Switch to tab 6
                        tabs_for_keys.select_tab_by_index(5);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_7 => {
                        // Ctrl+7: Switch to tab 7
                        tabs_for_keys.select_tab_by_index(6);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_8 => {
                        // Ctrl+8: Switch to tab 8
                        tabs_for_keys.select_tab_by_index(7);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::_9 => {
                        // Ctrl+9: Switch to tab 9
                        tabs_for_keys.select_tab_by_index(8);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::o | Key::O => {
                        // Ctrl+O: New document tab
                        tabs_for_keys.add_document_tab("Document", None);
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::Tab => {
                        // Ctrl+Tab: Next tab
                        tabs_for_keys.select_next_tab();
                        return gtk4::glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            if ctrl && shift {
                match key {
                    Key::T => {
                        // Ctrl+Shift+T: Reopen closed tab (TODO)
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::Tab | Key::ISO_Left_Tab => {
                        // Ctrl+Shift+Tab: Previous tab
                        tabs_for_keys.select_previous_tab();
                        return gtk4::glib::Propagation::Stop;
                    }
                    Key::o | Key::O => {
                        // Ctrl+Shift+O: Open file dialog
                        let tabs = tabs_for_keys.clone();
                        let win = window_for_keys.clone();

                        // Create file filter for text files
                        let filter = FileFilter::new();
                        filter.set_name(Some("Text Files"));
                        filter.add_mime_type("text/*");
                        filter.add_suffix("txt");
                        filter.add_suffix("md");
                        filter.add_suffix("rs");
                        filter.add_suffix("py");
                        filter.add_suffix("js");
                        filter.add_suffix("ts");
                        filter.add_suffix("json");
                        filter.add_suffix("toml");
                        filter.add_suffix("yaml");
                        filter.add_suffix("yml");

                        let filters = gio::ListStore::new::<FileFilter>();
                        filters.append(&filter);

                        let dialog = FileDialog::builder()
                            .title("Open File")
                            .modal(true)
                            .filters(&filters)
                            .build();

                        dialog.open(Some(&win), None::<&gio::Cancellable>, move |result| {
                            if let Ok(file) = result {
                                if let Some(path) = file.path() {
                                    let filename = path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("Document");
                                    let page = tabs.add_document_tab(filename, Some(&path));
                                    tracing::info!("Opened file: {:?}", path);
                                    let _ = page; // Use the page if needed
                                }
                            }
                        });
                        return gtk4::glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        Self {
            window,
            tabs,
            sidebar,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn widget(&self) -> &ApplicationWindow {
        &self.window
    }
}
