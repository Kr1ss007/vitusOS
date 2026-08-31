//! Thread-safe EventBus implementation matching §4.3 & §4.4 of the specification.
//!
//! Provides synchronous dispatch on the main thread and lock-protected `publish_async()`
//! for background worker threads draining into the main frame loop.

use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::events::AEEvent;

type EventHandler = Arc<dyn Fn(&AEEvent) + Send + Sync>;

#[derive(Clone)]
struct Subscription {
    id: u64,
    handler: EventHandler,
}

/// Thread-safe event bus supporting synchronous callbacks, asynchronous queue drain, and tokio broadcast.
#[derive(Clone)]
pub struct EventBus {
    handlers: Arc<RwLock<Vec<Subscription>>>,
    async_queue: Arc<parking_lot::Mutex<Vec<AEEvent>>>,
    async_tx: broadcast::Sender<AEEvent>,
    next_id: Arc<RwLock<u64>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (async_tx, _) = broadcast::channel(512);
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            async_queue: Arc::new(parking_lot::Mutex::new(Vec::new())),
            async_tx,
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Registers a handler callback. Returns a unique subscription ID for unregistering.
    pub fn subscribe<F>(&self, handler: F) -> u64
    where
        F: Fn(&AEEvent) + Send + Sync + 'static,
    {
        let mut id_guard = self.next_id.write();
        let id = *id_guard;
        *id_guard += 1;

        let sub = Subscription {
            id,
            handler: Arc::new(handler),
        };

        self.handlers.write().push(sub);
        id
    }

    /// Unregisters a handler by subscription ID.
    pub fn unsubscribe(&self, id: u64) {
        self.handlers.write().retain(|sub| sub.id != id);
    }

    /// Publishes an event synchronously to all registered handlers on the current thread.
    pub fn publish(&self, event: AEEvent) {
        let handlers = self.handlers.read().clone();
        for sub in handlers {
            (sub.handler)(&event);
        }
        let _ = self.async_tx.send(event);
    }

    /// Publishes an event from a background worker thread. Drained on main thread via `drain_async_queue`.
    pub fn publish_async(&self, event: AEEvent) {
        {
            let mut q = self.async_queue.lock();
            q.push(event);
        }
    }

    /// Drains all queued background events into synchronous publish handlers on the main loop.
    pub fn drain_async_queue(&self) {
        let events = {
            let mut q = self.async_queue.lock();
            if q.is_empty() {
                return;
            }
            std::mem::take(&mut *q)
        };

        for ev in events {
            self.publish(ev);
        }
    }

    /// Creates an asynchronous receiver stream for AEEvent broadcasts.
    pub fn subscribe_async(&self) -> broadcast::Receiver<AEEvent> {
        self.async_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_bus_sync_and_async_drain() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let c_clone = counter.clone();
        let sub_id = bus.subscribe(move |ev| {
            if let AEEvent::ShutdownRequested = ev {
                c_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Test synchronous publish
        bus.publish(AEEvent::ShutdownRequested);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Test background publish_async and drain
        bus.publish_async(AEEvent::ShutdownRequested);
        bus.publish_async(AEEvent::ShutdownRequested);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Not drained yet

        bus.drain_async_queue();
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Drained

        bus.unsubscribe(sub_id);
        bus.publish(AEEvent::ShutdownRequested);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Unsubscribed
    }
}
