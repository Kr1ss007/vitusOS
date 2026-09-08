//! ClipboardBridge -- Wayland Clipboard via wlr_seat (Part 43 + Addendum J).
//!
//! MIME types supported: text/plain;charset=utf-8, text/plain, text/html, image/png, image/jpeg.
//! NOT supported: application/octet-stream, non-text non-image types.
//! History: max 20 entries, memory-only, never persisted to disk.
//! HEV never touches clipboard data -- clipboard is transient, not a secret.

use std::collections::VecDeque;

use parking_lot::RwLock;
use tracing::info;

use crate::event_bus::EventBus;
use crate::events::AEEvent;

/// Maximum clipboard history entries (Part 43.3).
const MAX_HISTORY: usize = 20;

/// Supported MIME types (Part 43.2).
pub const MIME_TEXT_UTF8: &str = "text/plain;charset=utf-8";
pub const MIME_TEXT_PLAIN: &str = "text/plain";
pub const MIME_TEXT_HTML: &str = "text/html";
pub const MIME_IMAGE_PNG: &str = "image/png";
pub const MIME_IMAGE_JPEG: &str = "image/jpeg";

pub struct ClipboardBridge {
    current: RwLock<String>,
    history: RwLock<VecDeque<String>>,
    bus: EventBus,
}

impl ClipboardBridge {
    pub fn new(bus: EventBus) -> Self {
        Self {
            current: RwLock::new(String::new()),
            history: RwLock::new(VecDeque::new()),
            bus,
        }
    }

    /// Copies text to clipboard, adds to history, and publishes ClipboardChanged (Part 43.1).
    pub fn set_text(&self, text: &str) {
        // 1. Add to history (front)
        let mut history = self.history.write();
        history.push_front(text.to_string());
        if history.len() > MAX_HISTORY {
            history.pop_back();
        }
        drop(history);

        // 2. Set current
        *self.current.write() = text.to_string();

        // 3. Publish event
        self.bus.publish(AEEvent::ClipboardChanged);
        info!("ClipboardBridge: Text copied ({} chars), history={}",
              text.len(), self.history.read().len());
    }

    /// Returns current clipboard content.
    pub fn get_text(&self) -> String {
        self.current.read().clone()
    }

    /// Returns clipboard history (most recent first).
    pub fn history(&self) -> Vec<String> {
        self.history.read().iter().cloned().collect()
    }

    /// Clears clipboard history (user action only).
    pub fn clear_history(&self) {
        self.history.write().clear();
        info!("ClipboardBridge: History cleared");
    }

    /// Checks if a MIME type is supported (Part 43.2).
    pub fn is_mime_supported(mime: &str) -> bool {
        matches!(mime, MIME_TEXT_UTF8 | MIME_TEXT_PLAIN | MIME_TEXT_HTML | MIME_IMAGE_PNG | MIME_IMAGE_JPEG)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_set_and_get() {
        let bus = EventBus::new();
        let cb = ClipboardBridge::new(bus);

        assert_eq!(cb.get_text(), "");
        cb.set_text("Hello vitusOS");
        assert_eq!(cb.get_text(), "Hello vitusOS");
    }

    #[test]
    fn test_clipboard_history_max() {
        let bus = EventBus::new();
        let cb = ClipboardBridge::new(bus);

        for i in 0..25 {
            cb.set_text(&format!("entry_{}", i));
        }

        let history = cb.history();
        assert_eq!(history.len(), MAX_HISTORY);
        assert_eq!(history[0], "entry_24"); // Most recent first
    }

    #[test]
    fn test_clipboard_clear_history() {
        let bus = EventBus::new();
        let cb = ClipboardBridge::new(bus);

        cb.set_text("test1");
        cb.set_text("test2");
        assert_eq!(cb.history().len(), 2);

        cb.clear_history();
        assert_eq!(cb.history().len(), 0);
    }

    #[test]
    fn test_mime_support() {
        assert!(ClipboardBridge::is_mime_supported("text/plain;charset=utf-8"));
        assert!(ClipboardBridge::is_mime_supported("image/png"));
        assert!(!ClipboardBridge::is_mime_supported("application/octet-stream"));
        assert!(!ClipboardBridge::is_mime_supported("video/mp4"));
    }
}
