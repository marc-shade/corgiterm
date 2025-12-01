//! Broadcast Mode - Type in multiple terminals simultaneously
//!
//! Allows users to send the same input to multiple terminal panes/tabs at once.
//! Useful for managing multiple servers or running the same commands across environments.

use std::cell::RefCell;
use std::collections::HashSet;

/// Global broadcast state manager
pub struct BroadcastManager {
    /// Whether broadcast mode is enabled globally
    enabled: bool,
    /// Set of terminal IDs that are participating in broadcast
    /// If empty when enabled, broadcasts to ALL terminals
    broadcast_targets: HashSet<usize>,
    /// ID counter for assigning unique IDs to terminals
    next_id: usize,
}

impl BroadcastManager {
    pub fn new() -> Self {
        Self {
            enabled: false,
            broadcast_targets: HashSet::new(),
            next_id: 0,
        }
    }

    /// Check if broadcast mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Toggle broadcast mode on/off
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        if !self.enabled {
            // Clear targets when disabling
            self.broadcast_targets.clear();
        }
        tracing::info!(
            "Broadcast mode: {}",
            if self.enabled { "ON" } else { "OFF" }
        );
        self.enabled
    }

    /// Enable broadcast mode
    pub fn enable(&mut self) {
        self.enabled = true;
        tracing::info!("Broadcast mode enabled");
    }

    /// Disable broadcast mode
    pub fn disable(&mut self) {
        self.enabled = false;
        self.broadcast_targets.clear();
        tracing::info!("Broadcast mode disabled");
    }

    /// Get the next unique terminal ID
    pub fn next_terminal_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a terminal to broadcast targets
    pub fn add_target(&mut self, terminal_id: usize) {
        self.broadcast_targets.insert(terminal_id);
        tracing::debug!("Added terminal {} to broadcast targets", terminal_id);
    }

    /// Remove a terminal from broadcast targets
    pub fn remove_target(&mut self, terminal_id: usize) {
        self.broadcast_targets.remove(&terminal_id);
        tracing::debug!("Removed terminal {} from broadcast targets", terminal_id);
    }

    /// Toggle a terminal's broadcast participation
    pub fn toggle_target(&mut self, terminal_id: usize) -> bool {
        if self.broadcast_targets.contains(&terminal_id) {
            self.broadcast_targets.remove(&terminal_id);
            false
        } else {
            self.broadcast_targets.insert(terminal_id);
            true
        }
    }

    /// Check if a specific terminal is a broadcast target
    pub fn is_target(&self, terminal_id: usize) -> bool {
        // If no specific targets, all terminals are targets
        self.broadcast_targets.is_empty() || self.broadcast_targets.contains(&terminal_id)
    }

    /// Get number of active broadcast targets
    pub fn target_count(&self) -> usize {
        self.broadcast_targets.len()
    }

    /// Check if broadcasting to all (no specific targets)
    pub fn is_broadcasting_to_all(&self) -> bool {
        self.enabled && self.broadcast_targets.is_empty()
    }

    /// Get all target IDs
    pub fn targets(&self) -> &HashSet<usize> {
        &self.broadcast_targets
    }

    /// Clear all specific targets (broadcast to all when enabled)
    pub fn clear_targets(&mut self) {
        self.broadcast_targets.clear();
    }
}

impl Default for BroadcastManager {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local broadcast manager for single-threaded GTK access
thread_local! {
    static BROADCAST_MANAGER: RefCell<BroadcastManager> = RefCell::new(BroadcastManager::new());
}

/// Get access to the global broadcast manager
pub fn with_broadcast_manager<F, R>(f: F) -> R
where
    F: FnOnce(&BroadcastManager) -> R,
{
    BROADCAST_MANAGER.with(|bm| f(&bm.borrow()))
}

/// Get mutable access to the global broadcast manager
pub fn with_broadcast_manager_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut BroadcastManager) -> R,
{
    BROADCAST_MANAGER.with(|bm| f(&mut bm.borrow_mut()))
}

/// Check if broadcast mode is enabled (convenience function)
pub fn is_broadcast_enabled() -> bool {
    with_broadcast_manager(|bm| bm.is_enabled())
}

/// Toggle broadcast mode (convenience function)
pub fn toggle_broadcast() -> bool {
    with_broadcast_manager_mut(|bm| bm.toggle())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_toggle() {
        let mut bm = BroadcastManager::new();
        assert!(!bm.is_enabled());

        bm.toggle();
        assert!(bm.is_enabled());

        bm.toggle();
        assert!(!bm.is_enabled());
    }

    #[test]
    fn test_broadcast_targets() {
        let mut bm = BroadcastManager::new();
        bm.enable();

        // With no targets, is_target returns true for all
        assert!(bm.is_target(0));
        assert!(bm.is_target(1));

        // Add specific target
        bm.add_target(1);
        assert!(!bm.is_target(0));
        assert!(bm.is_target(1));

        // Toggle target
        bm.toggle_target(0);
        assert!(bm.is_target(0));
        assert!(bm.is_target(1));
    }
}
