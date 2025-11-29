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
    /// Cached list of all terminal panes (for focus cycling)
    all_panes: Rc<RefCell<Vec<Rc<RefCell<PaneNode>>>>>,
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

        let all_panes = Rc::new(RefCell::new(vec![root.clone()]));

        Self {
            container,
            root,
            focused_pane,
            working_dir,
            all_panes,
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

        // Update all_panes cache
        self.refresh_pane_list();

        tracing::info!("Split pane {:?}", direction);
    }

    /// Close the currently focused pane
    pub fn close_focused(&self) -> bool {
        // Don't close if there's only one pane
        if matches!(self.root.borrow().content, PaneContent::Terminal(_)) {
            return false;
        }

        let focused = self.focused_pane.borrow().clone();
        if let Some(focused_node) = focused {
            // If focused is root, we can't close it
            if Rc::ptr_eq(&focused_node, &self.root) {
                return false;
            }

            // Find parent and sibling
            if let Some((parent, sibling, is_start_child)) = self.find_parent_and_sibling(&focused_node) {
                self.close_pane_with_parent(parent, sibling, is_start_child);

                // Update all_panes cache
                self.refresh_pane_list();

                tracing::info!("Closed pane, restructured tree");
                return true;
            }
        }

        false
    }

    /// Find parent of a node and its sibling
    fn find_parent_and_sibling(
        &self,
        target: &Rc<RefCell<PaneNode>>,
    ) -> Option<(Rc<RefCell<PaneNode>>, Rc<RefCell<PaneNode>>, bool)> {
        self.find_parent_recursive(&self.root, target)
    }

    fn find_parent_recursive(
        &self,
        current: &Rc<RefCell<PaneNode>>,
        target: &Rc<RefCell<PaneNode>>,
    ) -> Option<(Rc<RefCell<PaneNode>>, Rc<RefCell<PaneNode>>, bool)> {
        match &current.borrow().content {
            PaneContent::Terminal(_) => None,
            PaneContent::Split { child1, child2, .. } => {
                // Check if target is one of our children
                if Rc::ptr_eq(child1, target) {
                    return Some((current.clone(), child2.clone(), true));
                }
                if Rc::ptr_eq(child2, target) {
                    return Some((current.clone(), child1.clone(), false));
                }

                // Recursively search children
                if let Some(result) = self.find_parent_recursive(child1, target) {
                    return Some(result);
                }
                self.find_parent_recursive(child2, target)
            }
        }
    }

    /// Close a pane by promoting its sibling to replace the parent
    fn close_pane_with_parent(
        &self,
        parent: Rc<RefCell<PaneNode>>,
        sibling: Rc<RefCell<PaneNode>>,
        _is_start_child: bool,
    ) {
        let parent_widget = parent.borrow().widget();

        // Take the sibling's content and put it in the parent
        let sibling_content = {
            let mut sibling_mut = sibling.borrow_mut();
            std::mem::replace(&mut sibling_mut.content, PaneContent::Terminal(TerminalView::new()))
        };

        // Replace parent's content with sibling's content
        {
            let mut parent_mut = parent.borrow_mut();
            parent_mut.content = sibling_content;
        }

        // Update the widget tree
        if Rc::ptr_eq(&parent, &self.root) {
            // Parent is root - update container
            self.container.remove(&parent_widget);
            self.container.append(&parent.borrow().widget());
        } else {
            // Parent is a child of another split - update the grandparent
            if let Some(grandparent) = parent_widget.parent() {
                if let Some(grandparent_paned) = grandparent.downcast_ref::<Paned>() {
                    let new_widget = parent.borrow().widget();
                    if grandparent_paned.start_child().as_ref() == Some(&parent_widget) {
                        grandparent_paned.set_start_child(Some(&new_widget));
                    } else {
                        grandparent_paned.set_end_child(Some(&new_widget));
                    }
                }
            }
        }

        // Update focus to a valid terminal
        self.update_focus_after_close();
    }

    /// Update focus to a valid terminal after closing a pane
    fn update_focus_after_close(&self) {
        // Find the first terminal in the tree
        let first_terminal = self.find_first_terminal(&self.root);
        *self.focused_pane.borrow_mut() = first_terminal;
    }

    /// Find the first terminal node in the tree
    fn find_first_terminal(&self, node: &Rc<RefCell<PaneNode>>) -> Option<Rc<RefCell<PaneNode>>> {
        match &node.borrow().content {
            PaneContent::Terminal(_) => Some(node.clone()),
            PaneContent::Split { child1, .. } => self.find_first_terminal(child1),
        }
    }

    /// Refresh the cached list of all terminal panes
    fn refresh_pane_list(&self) {
        let mut panes = Vec::new();
        self.collect_terminals(&self.root, &mut panes);
        *self.all_panes.borrow_mut() = panes;
    }

    /// Recursively collect all terminal nodes
    fn collect_terminals(&self, node: &Rc<RefCell<PaneNode>>, panes: &mut Vec<Rc<RefCell<PaneNode>>>) {
        match &node.borrow().content {
            PaneContent::Terminal(_) => {
                panes.push(node.clone());
            }
            PaneContent::Split { child1, child2, .. } => {
                self.collect_terminals(child1, panes);
                self.collect_terminals(child2, panes);
            }
        }
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

    /// Get current directory name from focused terminal for tab title
    pub fn current_directory_name(&self) -> String {
        // Try focused first
        if let Some(node) = self.focused_pane.borrow().as_ref() {
            if let Some(tv) = node.borrow().as_terminal() {
                return tv.current_directory_name();
            }
        }

        // Fallback to root
        if let Some(tv) = self.root.borrow().as_terminal() {
            return tv.current_directory_name();
        }

        "Terminal".to_string()
    }

    /// Move focus between panes
    pub fn focus_next(&self) {
        let panes = self.all_panes.borrow();
        if panes.len() <= 1 {
            return;
        }

        let focused = self.focused_pane.borrow().clone();
        if let Some(current) = focused {
            // Find current index
            if let Some(current_idx) = panes.iter().position(|p| Rc::ptr_eq(p, &current)) {
                // Move to next, wrapping around
                let next_idx = (current_idx + 1) % panes.len();
                *self.focused_pane.borrow_mut() = Some(panes[next_idx].clone());
                tracing::debug!("Focus moved to pane {}/{}", next_idx + 1, panes.len());

                // Request focus on the terminal widget
                if let Some(tv) = panes[next_idx].borrow().as_terminal() {
                    tv.widget().grab_focus();
                }
            }
        }
    }

    /// Move focus to previous pane
    pub fn focus_prev(&self) {
        let panes = self.all_panes.borrow();
        if panes.len() <= 1 {
            return;
        }

        let focused = self.focused_pane.borrow().clone();
        if let Some(current) = focused {
            // Find current index
            if let Some(current_idx) = panes.iter().position(|p| Rc::ptr_eq(p, &current)) {
                // Move to previous, wrapping around
                let prev_idx = if current_idx == 0 {
                    panes.len() - 1
                } else {
                    current_idx - 1
                };
                *self.focused_pane.borrow_mut() = Some(panes[prev_idx].clone());
                tracing::debug!("Focus moved to pane {}/{}", prev_idx + 1, panes.len());

                // Request focus on the terminal widget
                if let Some(tv) = panes[prev_idx].borrow().as_terminal() {
                    tv.widget().grab_focus();
                }
            }
        }
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
