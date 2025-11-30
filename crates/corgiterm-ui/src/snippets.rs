//! Snippets library UI for saving and reusing common commands

use gtk4::prelude::*;
use gtk4::{Orientation, SelectionMode};
use libadwaita::prelude::*;
use corgiterm_config::{Snippet, SnippetsManager};
use std::sync::Arc;
use parking_lot::RwLock;

/// Global snippets manager (initialized by app)
static SNIPPETS: std::sync::OnceLock<Arc<RwLock<SnippetsManager>>> = std::sync::OnceLock::new();

/// Initialize the global snippets manager
pub fn init_snippets(snippets: Arc<RwLock<SnippetsManager>>) {
    let _ = SNIPPETS.set(snippets);
}

/// Get the global snippets manager
pub fn get_snippets() -> Option<Arc<RwLock<SnippetsManager>>> {
    SNIPPETS.get().cloned()
}

/// Show the snippets library dialog
pub fn show_snippets_dialog<W, F>(
    parent: &W,
    on_insert: F,
) where
    W: IsA<gtk4::Window> + IsA<gtk4::Widget>,
    F: Fn(String) + 'static + Clone,
{
    let dialog = libadwaita::Dialog::builder()
        .title("Command Snippets")
        .content_width(650)
        .content_height(500)
        .build();

    let main_box = gtk4::Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Toolbar with search and add button
    let toolbar_box = gtk4::Box::new(Orientation::Horizontal, 8);

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Search snippets...")
        .hexpand(true)
        .build();
    toolbar_box.append(&search_entry);

    let add_button = gtk4::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add new snippet")
        .css_classes(vec!["circular"])
        .build();
    toolbar_box.append(&add_button);

    main_box.append(&toolbar_box);

    // Sort options
    let sort_box = gtk4::Box::new(Orientation::Horizontal, 8);
    sort_box.set_margin_bottom(8);

    let sort_label = gtk4::Label::new(Some("Sort by:"));
    sort_label.add_css_class("dim-label");
    sort_box.append(&sort_label);

    // Use modern DropDown instead of deprecated ComboBoxText
    let sort_options = ["Name", "Most Used", "Recently Used"];
    let sort_model = gtk4::StringList::new(&sort_options);
    let sort_dropdown = gtk4::DropDown::builder()
        .model(&sort_model)
        .selected(0) // "Name" by default
        .build();
    sort_box.append(&sort_dropdown);

    main_box.append(&sort_box);

    // Scrolled window with snippets list
    let scrolled = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    let list_box = gtk4::ListBox::builder()
        .selection_mode(SelectionMode::None)
        .css_classes(vec!["boxed-list"])
        .build();

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    // Function to populate the list
    let populate_list = {
        let list_box = list_box.clone();
        let on_insert = on_insert.clone();
        let dialog_ref = dialog.clone();

        move |query: &str, sort_by: &str| {
            // Clear existing rows
            while let Some(child) = list_box.first_child() {
                list_box.remove(&child);
            }

            if let Some(snippets_mgr) = get_snippets() {
                let snippets = snippets_mgr.read();

                // Get snippets based on search and sort
                let snippet_list: Vec<Snippet> = if query.is_empty() {
                    match sort_by {
                        "usage" => snippets.by_usage(),
                        "recent" => snippets.by_recency(),
                        _ => {
                            let mut all = snippets.snippets().snippets;
                            all.sort_by(|a, b| a.name.cmp(&b.name));
                            all
                        }
                    }
                } else {
                    let mut results = snippets.search(query);
                    results.sort_by(|a, b| a.name.cmp(&b.name));
                    results
                };

                // Show empty state if no snippets
                if snippet_list.is_empty() {
                    let empty_row = libadwaita::ActionRow::builder()
                        .title(if query.is_empty() {
                            "No snippets yet"
                        } else {
                            "No matching snippets"
                        })
                        .subtitle(if query.is_empty() {
                            "Click the + button to add your first snippet"
                        } else {
                            "Try a different search term"
                        })
                        .build();
                    empty_row.set_activatable(false);
                    list_box.append(&empty_row);
                    return;
                }

                // Create rows for each snippet
                for snippet in snippet_list {
                    let row = create_snippet_row(&snippet, on_insert.clone(), dialog_ref.clone());
                    list_box.append(&row);
                }
            }
        }
    };

    // Initial population
    populate_list("", "name");

    // Connect search
    {
        let populate = populate_list.clone();
        let sort_dropdown = sort_dropdown.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text();
            let sort_by = match sort_dropdown.selected() {
                1 => "usage",
                2 => "recent",
                _ => "name",
            };
            populate(&query, sort_by);
        });
    }

    // Connect sort change
    {
        let populate = populate_list.clone();
        let search_entry = search_entry.clone();
        sort_dropdown.connect_selected_notify(move |dropdown| {
            let query = search_entry.text();
            let sort_by = match dropdown.selected() {
                1 => "usage",
                2 => "recent",
                _ => "name",
            };
            populate(&query, sort_by);
        });
    }

    // Connect add button
    {
        let _dialog_ref = dialog.clone();
        let populate = populate_list.clone();
        let search_entry = search_entry.clone();
        let sort_dropdown = sort_dropdown.clone();

        add_button.connect_clicked(move |btn| {
            let populate_for_callback = populate.clone();
            let search_entry_for_callback = search_entry.clone();
            let sort_dropdown_for_callback = sort_dropdown.clone();

            // Get the root window for the editor dialog
            if let Some(root) = btn.root() {
                if let Ok(window) = root.downcast::<gtk4::Window>() {
                    show_snippet_editor_dialog(&window, None, move || {
                        let query = search_entry_for_callback.text();
                        let sort_by = match sort_dropdown_for_callback.selected() {
                            1 => "usage",
                            2 => "recent",
                            _ => "name",
                        };
                        populate_for_callback(&query, sort_by);
                    });
                }
            }
        });
    }

    dialog.set_child(Some(&main_box));
    dialog.present(Some(parent));

    // Focus search entry
    search_entry.grab_focus();
}

/// Create a snippet row for the list
fn create_snippet_row<F>(
    snippet: &Snippet,
    on_insert: F,
    parent_dialog: libadwaita::Dialog,
) -> libadwaita::ActionRow
where
    F: Fn(String) + 'static + Clone,
{
    let row = libadwaita::ActionRow::builder()
        .title(&snippet.name)
        .subtitle(&snippet.description)
        .build();

    // Command preview
    let command_label = gtk4::Label::new(Some(&snippet.command));
    command_label.add_css_class("monospace");
    command_label.add_css_class("dim-label");
    command_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    command_label.set_max_width_chars(50);
    row.add_suffix(&command_label);

    // Stats label (usage count)
    if snippet.use_count > 0 {
        let stats_label = gtk4::Label::new(Some(&format!("{}×", snippet.use_count)));
        stats_label.add_css_class("dim-label");
        stats_label.set_margin_start(8);
        row.add_suffix(&stats_label);
    }

    // Insert button
    let insert_btn = gtk4::Button::builder()
        .icon_name("insert-text-symbolic")
        .tooltip_text("Insert into terminal")
        .valign(gtk4::Align::Center)
        .css_classes(vec!["flat", "circular"])
        .build();

    {
        let snippet_id = snippet.id.clone();
        let command = snippet.command.clone();
        let on_insert = on_insert.clone();
        let parent_dialog = parent_dialog.clone();

        insert_btn.connect_clicked(move |_| {
            // Record usage
            if let Some(snippets_mgr) = get_snippets() {
                let _ = snippets_mgr.read().record_use(&snippet_id);
            }

            // Insert command
            on_insert(command.clone());

            // Close dialog
            parent_dialog.close();
        });
    }
    row.add_suffix(&insert_btn);

    // Edit button
    let edit_btn = gtk4::Button::builder()
        .icon_name("document-edit-symbolic")
        .tooltip_text("Edit snippet")
        .valign(gtk4::Align::Center)
        .css_classes(vec!["flat", "circular"])
        .build();

    {
        let snippet = snippet.clone();
        let _parent_dialog_for_edit = parent_dialog.clone();

        edit_btn.connect_clicked(move |btn| {
            if let Some(root) = btn.root() {
                if let Ok(window) = root.downcast::<gtk4::Window>() {
                    show_snippet_editor_dialog(&window, Some(snippet.clone()), || {
                        // Refresh not needed here, will be handled by parent
                    });
                }
            }
        });
    }
    row.add_suffix(&edit_btn);

    // Delete button
    let delete_btn = gtk4::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Delete snippet")
        .valign(gtk4::Align::Center)
        .css_classes(vec!["flat", "circular", "error"])
        .build();

    {
        let snippet_id = snippet.id.clone();
        let snippet_name = snippet.name.clone();

        delete_btn.connect_clicked(move |btn| {
            if let Some(root) = btn.root() {
                if let Ok(window) = root.downcast::<gtk4::Window>() {
                    show_delete_confirmation(&window, &snippet_name, &snippet_id);
                }
            }
        });
    }
    row.add_suffix(&delete_btn);

    row
}

/// Show snippet editor dialog (for creating or editing)
fn show_snippet_editor_dialog<W, F>(
    parent: &W,
    existing_snippet: Option<Snippet>,
    on_save: F,
) where
    W: IsA<gtk4::Window> + IsA<gtk4::Widget>,
    F: Fn() + 'static,
{
    let is_edit = existing_snippet.is_some();
    let title = if is_edit { "Edit Snippet" } else { "New Snippet" };

    let dialog = libadwaita::Dialog::builder()
        .title(title)
        .content_width(500)
        .build();

    let main_box = gtk4::Box::new(Orientation::Vertical, 16);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);

    // Form fields
    let name_row = libadwaita::EntryRow::builder()
        .title("Name")
        .build();

    let command_row = libadwaita::EntryRow::builder()
        .title("Command")
        .build();

    let desc_row = libadwaita::EntryRow::builder()
        .title("Description")
        .build();

    let tags_row = libadwaita::EntryRow::builder()
        .title("Tags")
        .build();
    // Note: set_subtitle is not available for EntryRow, use a label instead
    // tags_row.set_subtitle("Comma-separated tags for organization");

    // Populate if editing
    if let Some(ref snippet) = existing_snippet {
        name_row.set_text(&snippet.name);
        command_row.set_text(&snippet.command);
        desc_row.set_text(&snippet.description);
        tags_row.set_text(&snippet.tags.join(", "));
    }

    let form_group = libadwaita::PreferencesGroup::new();
    form_group.add(&name_row);
    form_group.add(&command_row);
    form_group.add(&desc_row);
    form_group.add(&tags_row);

    main_box.append(&form_group);

    // Buttons
    let button_box = gtk4::Box::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(12);

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("pill");
    let dialog_for_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });
    button_box.append(&cancel_btn);

    let save_btn = gtk4::Button::with_label(if is_edit { "Update" } else { "Add" });
    save_btn.add_css_class("pill");
    save_btn.add_css_class("suggested-action");

    {
        let dialog_for_save = dialog.clone();
        let name_row = name_row.clone();
        let command_row = command_row.clone();
        let desc_row = desc_row.clone();
        let tags_row = tags_row.clone();

        save_btn.connect_clicked(move |_| {
            let name = name_row.text().to_string();
            let command = command_row.text().to_string();
            let description = desc_row.text().to_string();
            let tags_text = tags_row.text().to_string();

            // Validation
            if name.is_empty() || command.is_empty() {
                tracing::warn!("Name and command are required");
                return;
            }

            // Parse tags
            let tags: Vec<String> = tags_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Save snippet
            if let Some(snippets_mgr) = get_snippets() {
                let result = if let Some(ref existing) = existing_snippet {
                    // Update existing
                    let mut updated = existing.clone();
                    updated.name = name;
                    updated.command = command;
                    updated.description = description;
                    updated.tags = tags;
                    snippets_mgr.read().update(updated)
                } else {
                    // Create new
                    let snippet = Snippet::new(name, command, description, tags);
                    snippets_mgr.read().add(snippet).map(|_| true)
                };

                match result {
                    Ok(true) => {
                        tracing::info!("Snippet saved successfully");
                        on_save();
                        dialog_for_save.close();
                    }
                    Ok(false) => {
                        tracing::warn!("Failed to save snippet");
                    }
                    Err(e) => {
                        tracing::error!("Error saving snippet: {}", e);
                    }
                }
            }
        });
    }
    button_box.append(&save_btn);

    main_box.append(&button_box);

    dialog.set_child(Some(&main_box));
    dialog.present(Some(parent));

    // Focus name field
    name_row.grab_focus();
}

/// Show delete confirmation dialog
fn show_delete_confirmation<W>(
    parent: &W,
    snippet_name: &str,
    snippet_id: &str,
) where
    W: IsA<gtk4::Window> + IsA<gtk4::Widget>,
{
    let dialog = libadwaita::AlertDialog::builder()
        .heading(&format!("Delete \"{}\"?", snippet_name))
        .body("This action cannot be undone.")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", libadwaita::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let snippet_id = snippet_id.to_string();
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            if let Some(snippets_mgr) = get_snippets() {
                match snippets_mgr.read().remove(&snippet_id) {
                    Ok(true) => {
                        tracing::info!("Snippet deleted successfully");
                        // Parent list will need manual refresh
                    }
                    Ok(false) => {
                        tracing::warn!("Snippet not found");
                    }
                    Err(e) => {
                        tracing::error!("Error deleting snippet: {}", e);
                    }
                }
            }
        }
    });

    dialog.present(Some(parent));
}

/// Show quick insert dialog (Ctrl+Shift+P style)
pub fn show_quick_insert_dialog<W, F>(
    parent: &W,
    on_insert: F,
) where
    W: IsA<gtk4::Window> + IsA<gtk4::Widget>,
    F: Fn(String) + 'static + Clone,
{
    let dialog = libadwaita::Dialog::builder()
        .title("Insert Snippet")
        .content_width(500)
        .content_height(400)
        .build();

    let main_box = gtk4::Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Search entry
    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Type to search snippets...")
        .hexpand(true)
        .build();
    main_box.append(&search_entry);

    // List
    let scrolled = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .min_content_height(300)
        .build();

    let list_box = gtk4::ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .css_classes(vec!["boxed-list"])
        .build();

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    // Populate function
    let populate = {
        let list_box = list_box.clone();
        let _on_insert = on_insert.clone();
        let _dialog_ref = dialog.clone();

        move |query: &str| {
            // Clear existing
            while let Some(child) = list_box.first_child() {
                list_box.remove(&child);
            }

            if let Some(snippets_mgr) = get_snippets() {
                let snippets = snippets_mgr.read();
                let snippet_list: Vec<Snippet> = if query.is_empty() {
                    snippets.by_usage()
                } else {
                    snippets.search(query)
                };

                for (idx, snippet) in snippet_list.iter().take(10).enumerate() {
                    let row_box = gtk4::Box::new(Orientation::Horizontal, 12);
                    row_box.set_margin_top(8);
                    row_box.set_margin_bottom(8);
                    row_box.set_margin_start(12);
                    row_box.set_margin_end(12);

                    // Icon
                    let icon = gtk4::Image::from_icon_name("utilities-terminal-symbolic");
                    icon.set_pixel_size(24);
                    row_box.append(&icon);

                    // Name and command
                    let text_box = gtk4::Box::new(Orientation::Vertical, 4);
                    text_box.set_hexpand(true);

                    let name_label = gtk4::Label::new(Some(&snippet.name));
                    name_label.set_halign(gtk4::Align::Start);
                    name_label.add_css_class("heading");
                    text_box.append(&name_label);

                    let cmd_label = gtk4::Label::new(Some(&snippet.command));
                    cmd_label.set_halign(gtk4::Align::Start);
                    cmd_label.add_css_class("monospace");
                    cmd_label.add_css_class("dim-label");
                    cmd_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                    text_box.append(&cmd_label);

                    row_box.append(&text_box);

                    // Hint
                    let hint_label = gtk4::Label::new(Some(&format!("#{}", idx + 1)));
                    hint_label.add_css_class("dim-label");
                    row_box.append(&hint_label);

                    let row = gtk4::ListBoxRow::new();
                    row.set_child(Some(&row_box));
                    row.set_widget_name(&snippet.id);
                    list_box.append(&row);
                }

                // Select first row
                if let Some(first_row) = list_box.row_at_index(0) {
                    list_box.select_row(Some(&first_row));
                }
            }
        }
    };

    // Initial populate
    populate("");

    // Connect search
    {
        let populate = populate.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text();
            populate(&query);
        });
    }

    // Handle activation
    {
        let on_insert = on_insert.clone();
        let dialog_ref = dialog.clone();

        list_box.connect_row_activated(move |_, row| {
            let snippet_id = row.widget_name();
            if let Some(snippets_mgr) = get_snippets() {
                let snippets = snippets_mgr.read();
                let all_snippets = snippets.snippets().snippets;
                if let Some(snippet) = all_snippets.iter().find(|s| s.id == snippet_id) {
                    // Record usage
                    let _ = snippets.record_use(&snippet.id);

                    // Insert
                    on_insert(snippet.command.clone());
                    dialog_ref.close();
                }
            }
        });
    }

    // Handle Enter on search
    {
        let list_box = list_box.clone();
        let on_insert = on_insert.clone();
        let dialog_ref = dialog.clone();

        search_entry.connect_activate(move |_| {
            if let Some(selected) = list_box.selected_row() {
                let snippet_id = selected.widget_name();
                if let Some(snippets_mgr) = get_snippets() {
                    let snippets = snippets_mgr.read();
                    let all_snippets = snippets.snippets().snippets;
                    if let Some(snippet) = all_snippets.iter().find(|s| s.id == snippet_id) {
                        let _ = snippets.record_use(&snippet.id);
                        on_insert(snippet.command.clone());
                        dialog_ref.close();
                    }
                }
            }
        });
    }

    // Handle Escape
    let key_controller = gtk4::EventControllerKey::new();
    let dialog_for_escape = dialog.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        use gtk4::gdk::Key;
        if key == Key::Escape {
            dialog_for_escape.close();
            gtk4::glib::Propagation::Stop
        } else {
            gtk4::glib::Propagation::Proceed
        }
    });
    search_entry.add_controller(key_controller);

    dialog.set_child(Some(&main_box));
    dialog.present(Some(parent));

    // Focus search
    search_entry.grab_focus();
}
