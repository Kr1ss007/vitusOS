//! AccessibilityProvider — exposes the AE surface tree to AT-SPI2 (Part 22.4).
//!
//! Direction: AnimusEngine -> D-Bus (outbound only from compositor's perspective).
//! Consumes: AENative surface tree (Panel, Dock, all open windows).
//! Exposes: org.a11y.atspi2 — Orca and other screen readers connect here.
//!
//! Focus chain: keyboard Tab order follows window z-order, then
//!   within-window order: toolbar -> sidebar -> content -> overlays.
//!
//! ReducedMotion: reads from StateManager user prefs.
//!   On change: publishes AEEvent::ReducedMotionChanged.
//!   SpringSolver reduced motion snaps to target instantly.
//!   MaterialRenderer reduces blur transition speed.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// AT-SPI role constants (subset of org.a11y.atspi2.Role).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum A11yRole {
    Window = 64,
    Panel = 48,
    ToolBar = 57,
    PushButton = 28,
    TextField = 60,
    Label = 34,
    CheckBox = 8,
    Slider = 52,
    ProgressBar = 29,
    List = 35,
    ListItem = 36,
    ScrollBar = 47,
    Separator = 49,
    Filler = 19,
}

/// An accessibility tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yNode {
    pub id: u32,
    pub role: A11yRole,
    pub name: String,
    pub description: String,
    pub focused: bool,
    pub enabled: bool,
    pub children: Vec<u32>,
    pub parent: Option<u32>,
}

/// Accessibility state flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct A11yState {
    pub focused: bool,
    pub enabled: bool,
    pub visible: bool,
    pub editable: bool,
    pub checked: bool,
}

impl Default for A11yState {
    fn default() -> Self {
        Self {
            focused: false,
            enabled: true,
            visible: true,
            editable: false,
            checked: false,
        }
    }
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// The accessibility provider.
pub struct AccessibilityProvider {
    /// The accessibility tree, indexed by node ID.
    tree: std::collections::HashMap<u32, A11yNode>,
    /// Focus chain — ordered list of node IDs for Tab navigation.
    focus_chain: Vec<u32>,
    /// Current focus position in the focus chain.
    focus_index: Option<usize>,
    /// Reduced motion state.
    reduced_motion: bool,
    /// Root node ID.
    root_id: Option<u32>,
}

impl AccessibilityProvider {
    pub fn new() -> Self {
        Self {
            tree: std::collections::HashMap::new(),
            focus_chain: Vec::new(),
            focus_index: None,
            reduced_motion: false,
            root_id: None,
        }
    }

    /// Initialize the accessibility provider.
    pub fn initialize(&mut self) -> bool {
        self.load_reduced_motion_setting();
        true
    }

    /// Rebuild the accessibility tree from current surface layout.
    /// Called on: window open/close, focus change, layout change.
    pub fn rebuild_tree(&mut self) {
        // Clear existing tree
        self.tree.clear();
        self.focus_chain.clear();

        // Create root window node
        let root_id = next_id();
        self.root_id = Some(root_id);
        let root = A11yNode {
            id: root_id,
            role: A11yRole::Window,
            name: "vitusOS Desktop".to_string(),
            description: "Main desktop window".to_string(),
            focused: true,
            enabled: true,
            children: Vec::new(),
            parent: None,
        };
        self.tree.insert(root_id, root);
        self.focus_chain.push(root_id);
        self.focus_index = Some(0);
    }

    /// Add a node to the accessibility tree.
    pub fn add_node(
        &mut self,
        role: A11yRole,
        name: impl Into<String>,
        description: impl Into<String>,
        parent_id: Option<u32>,
        enabled: bool,
        focusable: bool,
    ) -> u32 {
        let id = next_id();
        let node = A11yNode {
            id,
            role,
            name: name.into(),
            description: description.into(),
            focused: false,
            enabled,
            children: Vec::new(),
            parent: parent_id,
        };
        self.tree.insert(id, node);

        // Link to parent
        if let Some(pid) = parent_id {
            if let Some(parent) = self.tree.get_mut(&pid) {
                parent.children.push(id);
            }
        }

        // Add to focus chain if focusable
        if focusable {
            self.focus_chain.push(id);
        }

        id
    }

    /// Remove a node and all its descendants from the tree.
    pub fn remove_node(&mut self, id: u32) {
        // Collect all descendants
        let mut to_remove = Vec::new();
        self.collect_descendants(id, &mut to_remove);

        // Remove from parent's children
        if let Some(node) = self.tree.get(&id).cloned() {
            if let Some(pid) = node.parent {
                if let Some(parent) = self.tree.get_mut(&pid) {
                    parent.children.retain(|&c| c != id);
                }
            }
        }

        // Remove all nodes
        for rid in to_remove {
            self.tree.remove(&rid);
            self.focus_chain.retain(|&fid| fid != rid);
        }
        self.tree.remove(&id);
        self.focus_chain.retain(|&fid| fid != id);

        // Fix focus index
        if self.focus_chain.is_empty() {
            self.focus_index = None;
        } else if let Some(idx) = self.focus_index {
            if idx >= self.focus_chain.len() {
                self.focus_index = Some(self.focus_chain.len() - 1);
            }
        }
    }

    fn collect_descendants(&self, id: u32, out: &mut Vec<u32>) {
        if let Some(node) = self.tree.get(&id) {
            for &child_id in &node.children {
                out.push(child_id);
                self.collect_descendants(child_id, out);
            }
        }
    }

    /// Focus the next node in the focus chain (Tab key).
    pub fn next_focus(&mut self) -> Option<u32> {
        if self.focus_chain.is_empty() {
            return None;
        }
        let current = self.focus_index.unwrap_or(0);
        let next = (current + 1) % self.focus_chain.len();

        // Unfocus current
        if let Some(&cur_id) = self.focus_chain.get(current) {
            if let Some(node) = self.tree.get_mut(&cur_id) {
                node.focused = false;
            }
        }

        // Focus next
        let &next_id = self.focus_chain.get(next)?;
        if let Some(node) = self.tree.get_mut(&next_id) {
            node.focused = true;
        }
        self.focus_index = Some(next);
        Some(next_id)
    }

    /// Focus the previous node in the focus chain (Shift-Tab).
    pub fn prev_focus(&mut self) -> Option<u32> {
        if self.focus_chain.is_empty() {
            return None;
        }
        let current = self.focus_index.unwrap_or(0);
        let prev = if current == 0 {
            self.focus_chain.len() - 1
        } else {
            current - 1
        };

        // Unfocus current
        if let Some(&cur_id) = self.focus_chain.get(current) {
            if let Some(node) = self.tree.get_mut(&cur_id) {
                node.focused = false;
            }
        }

        // Focus prev
        let &prev_id = self.focus_chain.get(prev)?;
        if let Some(node) = self.tree.get_mut(&prev_id) {
            node.focused = true;
        }
        self.focus_index = Some(prev);
        Some(prev_id)
    }

    /// Focus a specific node by ID.
    pub fn set_focus(&mut self, id: u32) -> bool {
        // Unfocus current
        if let Some(idx) = self.focus_index {
            if let Some(&cur_id) = self.focus_chain.get(idx) {
                if let Some(node) = self.tree.get_mut(&cur_id) {
                    node.focused = false;
                }
            }
        }

        // Find the node in the focus chain
        if let Some(idx) = self.focus_chain.iter().position(|&fid| fid == id) {
            if let Some(node) = self.tree.get_mut(&id) {
                node.focused = true;
            }
            self.focus_index = Some(idx);
            true
        } else {
            false
        }
    }

    /// Get the currently focused node ID.
    pub fn current_focus(&self) -> Option<u32> {
        self.focus_index.and_then(|idx| self.focus_chain.get(idx).copied())
    }

    /// Announce focus change to screen reader.
    /// In a real implementation, this sends a D-Bus signal to org.a11y.atspi2.
    pub fn announce_node_focused(&self, id: u32) {
        if let Some(node) = self.tree.get(&id) {
            tracing::info!(
                "AccessibilityProvider: Focus announced — {} ({:?})",
                node.name,
                node.role
            );
        }
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: u32) -> Option<&A11yNode> {
        self.tree.get(&id)
    }

    /// Get all nodes in the tree.
    pub fn node_count(&self) -> usize {
        self.tree.len()
    }

    /// Get the focus chain length.
    pub fn focus_chain_len(&self) -> usize {
        self.focus_chain.len()
    }

    /// Check if reduced motion is enabled.
    pub fn reduced_motion_enabled(&self) -> bool {
        self.reduced_motion
    }

    /// Set reduced motion state and propagate to the global spring system.
    pub fn set_reduced_motion(&mut self, enabled: bool) {
        if self.reduced_motion != enabled {
            self.reduced_motion = enabled;
            animus_physics::set_reduced_motion(enabled);
            tracing::info!(
                "AccessibilityProvider: Reduced motion {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Load reduced motion setting from StateManager.
    fn load_reduced_motion_setting(&mut self) {
        // In a real implementation, this reads from StateManager:
        //   state_manager.get_bool("reduced_motion")
        // For now, default to false.
        self.reduced_motion = false;
        animus_physics::set_reduced_motion(false);
    }

    /// Get the root node ID.
    pub fn root_id(&self) -> Option<u32> {
        self.root_id
    }

    /// Get all focusable node IDs in order.
    pub fn focus_chain(&self) -> &[u32] {
        &self.focus_chain
    }
}

impl Default for AccessibilityProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a11y_rebuild_tree() {
        let mut a11y = AccessibilityProvider::new();
        assert_eq!(a11y.node_count(), 0);

        a11y.rebuild_tree();
        assert!(a11y.node_count() > 0);
        assert!(a11y.root_id().is_some());
    }

    #[test]
    fn a11y_add_remove_nodes() {
        let mut a11y = AccessibilityProvider::new();
        a11y.rebuild_tree();
        let root = a11y.root_id().unwrap();
        let initial_count = a11y.node_count();

        let btn_id = a11y.add_node(
            A11yRole::PushButton,
            "Close",
            "Close button",
            Some(root),
            true,
            true,
        );
        assert_eq!(a11y.node_count(), initial_count + 1);

        let label_id = a11y.add_node(
            A11yRole::Label,
            "Title",
            "Window title",
            Some(root),
            true,
            false,
        );
        assert_eq!(a11y.node_count(), initial_count + 2);
        assert!(a11y.focus_chain_len() >= 2); // root + button (label not focusable)

        a11y.remove_node(btn_id);
        assert_eq!(a11y.node_count(), initial_count + 1);

        a11y.remove_node(label_id);
        assert_eq!(a11y.node_count(), initial_count);
    }

    #[test]
    fn a11y_focus_navigation() {
        let mut a11y = AccessibilityProvider::new();
        a11y.rebuild_tree();
        let root = a11y.root_id().unwrap();

        let btn1 = a11y.add_node(A11yRole::PushButton, "Button 1", "", Some(root), true, true);
        let btn2 = a11y.add_node(A11yRole::PushButton, "Button 2", "", Some(root), true, true);
        let btn3 = a11y.add_node(A11yRole::PushButton, "Button 3", "", Some(root), true, true);

        // Initial focus should be root (index 0)
        assert_eq!(a11y.current_focus(), Some(root));

        // Tab → next focus
        let next = a11y.next_focus();
        assert_eq!(next, Some(btn1));

        // Tab → next focus
        let next = a11y.next_focus();
        assert_eq!(next, Some(btn2));

        // Tab → next focus
        let next = a11y.next_focus();
        assert_eq!(next, Some(btn3));

        // Tab → wraps to root
        let next = a11y.next_focus();
        assert_eq!(next, Some(root));

        // Shift-Tab → wraps to btn3
        let prev = a11y.prev_focus();
        assert_eq!(prev, Some(btn3));

        // Shift-Tab → btn2
        let prev = a11y.prev_focus();
        assert_eq!(prev, Some(btn2));
    }

    #[test]
    fn a11y_set_focus() {
        let mut a11y = AccessibilityProvider::new();
        a11y.rebuild_tree();
        let root = a11y.root_id().unwrap();

        let btn = a11y.add_node(A11yRole::PushButton, "Test", "", Some(root), true, true);

        assert!(a11y.set_focus(btn));
        assert_eq!(a11y.current_focus(), Some(btn));

        // Verify node is marked as focused
        let node = a11y.get_node(btn).unwrap();
        assert!(node.focused);

        // Verify previous focus is unfocused
        let root_node = a11y.get_node(root).unwrap();
        assert!(!root_node.focused);
    }

    #[test]
    fn a11y_reduced_motion_toggle() {
        let mut a11y = AccessibilityProvider::new();

        assert!(!a11y.reduced_motion_enabled());
        assert!(!animus_physics::is_reduced_motion());

        a11y.set_reduced_motion(true);
        assert!(a11y.reduced_motion_enabled());
        assert!(animus_physics::is_reduced_motion());

        a11y.set_reduced_motion(false);
        assert!(!a11y.reduced_motion_enabled());
        assert!(!animus_physics::is_reduced_motion());
    }

    #[test]
    fn a11y_remove_with_descendants() {
        let mut a11y = AccessibilityProvider::new();
        a11y.rebuild_tree();
        let root = a11y.root_id().unwrap();

        let panel = a11y.add_node(A11yRole::Panel, "Panel", "", Some(root), true, false);
        let btn1 = a11y.add_node(A11yRole::PushButton, "Btn 1", "", Some(panel), true, true);
        let btn2 = a11y.add_node(A11yRole::PushButton, "Btn 2", "", Some(panel), true, true);

        let count_before = a11y.node_count();

        // Remove panel — should also remove btn1 and btn2
        a11y.remove_node(panel);
        assert_eq!(a11y.node_count(), count_before - 3);
        assert!(a11y.get_node(btn1).is_none());
        assert!(a11y.get_node(btn2).is_none());
    }

    #[test]
    fn a11y_focus_chain_excludes_non_focusable() {
        let mut a11y = AccessibilityProvider::new();
        a11y.rebuild_tree();
        let root = a11y.root_id().unwrap();

        a11y.add_node(A11yRole::Label, "Label", "", Some(root), true, false);
        a11y.add_node(A11yRole::Separator, "Sep", "", Some(root), true, false);
        let btn = a11y.add_node(A11yRole::PushButton, "Button", "", Some(root), true, true);

        // Focus chain should be: root, button (not label or separator)
        assert_eq!(a11y.focus_chain_len(), 2);

        // Verify label and separator are not in the chain
        let chain = a11y.focus_chain();
        assert!(chain.contains(&root));
        assert!(chain.contains(&btn));
    }
}
