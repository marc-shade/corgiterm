//! Split pane container for terminal views
//!
//! Allows horizontal and vertical splitting of terminal panes.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Paned, Widget};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;

use crate::terminal_view::TerminalView;

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Content of a split pane node
enum PaneContent {
    /// Leaf node: single terminal
    Terminal(TerminalView),
    /// Split node: two child panes
    Split {
        paned: Paned,
        child1: Rc<RefCell<PaneNode>>,
        child2: Rc<RefCell<PaneNode>>,
    },
}

/// A node in the pane tree
struct PaneNode {
    content: PaneContent,
}

impl PaneNode {
    fn new_terminal(working_dir: Option<&Path>) -> Self {
        let terminal = if let Some(dir) = working_dir {
            TerminalView::with_working_dir(Some(dir))
        } else {
            TerminalView::new()
        };
        Self {
            content: PaneContent::Terminal(terminal),
        }
    }

    fn widget(&self) -> Widget {
        match &self.content {
            PaneContent::Terminal(tv) => tv.widget().clone().upcast(),
            PaneContent::Split { paned, .. } => paned.clone().upcast(),
        }
    }

    fn as_terminal(&self) -> Option<&TerminalView> {
        match &self.content {
            PaneContent::Terminal(tv) => Some(tv),
            _ => None,
        }
    }
}

/// Split pane container widget
pub struct SplitPane {
    /// Container box
    container: GtkBox,
    /// Root node of the pane tree
    root: Rc<RefCell<PaneNode>>,
    /// Currently focused pane (for keyboard navigation)
    focused_pane: Rc<RefCell<Option<Rc<RefCell<PaneNode>>>>>,
    /// Working directory for new panes
    working_dir: Rc<RefCell<Option<std::path::PathBuf>>>,
}

impl SplitPane {
    pub fn new() -> Self {
        Self::with_working_dir(None)
    }

    pub fn with_working_dir(working_dir: Option<&Path>) -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);
        container.add_css_class("split-pane");

        let root = Rc::new(RefCell::new(PaneNode::new_terminal(working_dir)));

        // Add root widget to container
        container.append(&root.borrow().widget());

        let focused_pane: Rc<RefCell<Option<Rc<RefCell<PaneNode>>>>> = Rc::new(RefCell::new(Some(root.clone())));

        let working_dir = Rc::new(RefCell::new(working_dir.map(|p| p.to_path_buf())));

        Self {
            container,
            root,
            focused_pane,
            working_dir,
        }
    }

    /// Get the main widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Split the currently focused pane
    pub fn split(&self, direction: SplitDirection) {
        let focused = self.focused_pane.borrow().clone();
        if let Some(focused_node) = focused {
            self.split_node(focused_node, direction);
        }
    }

    /// Split a specific pane node
    fn split_node(&self, node: Rc<RefCell<PaneNode>>, direction: SplitDirection) {
        let working_dir = self.working_dir.borrow();
        let working_path = working_dir.as_ref().map(|p| p.as_path());

        // Get the old widget before we modify the node
        let old_widget = node.borrow().widget();

        // Create the paned widget
        let orientation = match direction {
            SplitDirection::Horizontal => Orientation::Horizontal,
            SplitDirection::Vertical => Orientation::Vertical,
        };
        let paned = Paned::new(orientation);
        paned.set_vexpand(true);
        paned.set_hexpand(true);

        // Create new terminal for second pane
        let new_terminal = PaneNode::new_terminal(working_path);
        let child2 = Rc::new(RefCell::new(new_terminal));

        // Take ownership of the old terminal
        let old_content = {
            let mut node_mut = node.borrow_mut();
            std::mem::replace(&mut node_mut.content, PaneContent::Terminal(TerminalView::new()))
        };

        // Create child1 from old content
        let child1 = Rc::new(RefCell::new(PaneNode { content: old_content }));

        // Set up the paned widget
        paned.set_start_child(Some(&child1.borrow().widget()));
        paned.set_end_child(Some(&child2.borrow().widget()));
        paned.set_position(300); // Default split position

        // Update the node with the split content
        {
            let mut node_mut = node.borrow_mut();
            node_mut.content = PaneContent::Split {
                paned: paned.clone(),
                child1: child1.clone(),
                child2: child2.clone(),
            };
        }

        // Replace widget in container if this is the root
        if Rc::ptr_eq(&node, &self.root) {
            self.container.remove(&old_widget);
            self.container.append(&paned);
        } else {
            // For nested splits, we need to update the parent paned
            // This is handled automatically since we modified the node in place
            // and the parent holds a reference to this node
            if let Some(parent) = old_widget.parent() {
                if let Some(parent_paned) = parent.downcast_ref::<Paned>() {
                    if parent_paned.start_child().as_ref() == Some(&old_widget) {
                        parent_paned.set_start_child(Some(&paned));
                    } else {
                        parent_paned.set_end_child(Some(&paned));
                    }
                }
            }
        }

        // Focus the new pane
        *self.focused_pane.borrow_mut() = Some(child2);

        tracing::info!("Split pane {:?}", direction);
    }

    /// Close the currently focused pane
    pub fn close_focused(&self) -> bool {
        // Don't close if there's only one pane
        if matches!(self.root.borrow().content, PaneContent::Terminal(_)) {
            return false;
        }

        // TODO: Implement proper pane closing with tree restructuring
        // For now, this is a placeholder
        tracing::debug!("Close pane not fully implemented yet");
        false
    }

    /// Get the focused terminal view
    pub fn focused_terminal(&self) -> Option<Rc<RefCell<TerminalView>>> {
        let focused = self.focused_pane.borrow();
        focused.as_ref().and_then(|node| {
            match &node.borrow().content {
                PaneContent::Terminal(_) => {
                    // We can't easily return a reference here, so return None
                    // The caller should use send_command instead
                    None
                }
                _ => None,
            }
        })
    }

    /// Send command to the focused terminal
    pub fn send_command(&self, command: &str) -> bool {
        let focused = self.focused_pane.borrow();
        if let Some(node) = focused.as_ref() {
            if let Some(tv) = node.borrow().as_terminal() {
                tv.send_command(command);
                return true;
            }
        }

        // Fallback: send to root terminal
        if let Some(tv) = self.root.borrow().as_terminal() {
            tv.send_command(command);
            return true;
        }

        false
    }

    /// Get visible lines from focused terminal (for thumbnails)
    pub fn get_visible_lines(&self, max_lines: usize) -> Vec<String> {
        // Try focused first
        if let Some(node) = self.focused_pane.borrow().as_ref() {
            if let Some(tv) = node.borrow().as_terminal() {
                return tv.get_visible_lines(max_lines);
            }
        }

        // Fallback to root
        if let Some(tv) = self.root.borrow().as_terminal() {
            return tv.get_visible_lines(max_lines);
        }

        Vec::new()
    }

    /// Move focus between panes
    pub fn focus_next(&self) {
        // TODO: Implement focus cycling between panes
        tracing::debug!("Focus next pane");
    }

    /// Move focus to previous pane
    pub fn focus_prev(&self) {
        // TODO: Implement focus cycling between panes
        tracing::debug!("Focus previous pane");
    }

    /// Check if this pane is split
    pub fn is_split(&self) -> bool {
        !matches!(self.root.borrow().content, PaneContent::Terminal(_))
    }

    /// Get pane count
    pub fn pane_count(&self) -> usize {
        self.count_terminals(&self.root)
    }

    fn count_terminals(&self, node: &Rc<RefCell<PaneNode>>) -> usize {
        match &node.borrow().content {
            PaneContent::Terminal(_) => 1,
            PaneContent::Split { child1, child2, .. } => {
                self.count_terminals(child1) + self.count_terminals(child2)
            }
        }
    }
}

impl Default for SplitPane {
    fn default() -> Self {
        Self::new()
    }
}
