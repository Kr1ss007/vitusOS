//! AnimationEngine and Settle Dispatcher (FIX4-02).
//!
//! Evaluates active physics springs and invokes dedicated settle callbacks directly
//! without polluting `AEEvent::Tick`.

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

pub type SettlePredicate = Arc<dyn Fn() -> bool + Send + Sync>;
pub type SettleCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct Settler {
    pub id: u64,
    pub is_settled: SettlePredicate,
    pub callback: SettleCallback,
}

pub struct AnimationEngine {
    is_running: AtomicBool,
    settlers: Mutex<Vec<Settler>>,
    next_id: AtomicU64,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            is_running: AtomicBool::new(true),
            settlers: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Registers a one-shot callback when a spring satisfies the `is_settled` predicate.
    pub fn on_settle<P, C>(&self, is_settled: P, callback: C) -> u64
    where
        P: Fn() -> bool + Send + Sync + 'static,
        C: Fn() + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let settler = Settler {
            id,
            is_settled: Arc::new(is_settled),
            callback: Arc::new(callback),
        };

        self.settlers.lock().push(settler);
        id
    }

    /// Cancels a registered settle callback by ID.
    pub fn cancel_settle(&self, id: u64) {
        self.settlers.lock().retain(|s| s.id != id);
    }

    /// Advances frame time and evaluates all active settlers on the main loop.
    pub fn tick(&self, _dt: f32) {
        if !self.is_running.load(Ordering::Relaxed) {
            return;
        }

        let mut fired = Vec::new();
        {
            let mut list = self.settlers.lock();
            list.retain(|s| {
                if (s.is_settled)() {
                    fired.push(s.callback.clone());
                    false // Remove from list
                } else {
                    true // Keep waiting
                }
            });
        }

        // Fire callbacks directly on the main thread (FIX4-02)
        for cb in fired {
            cb();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_animation_engine_on_settle() {
        let engine = AnimationEngine::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let c_clone = counter.clone();

        engine.on_settle(
            move || flag_clone.load(Ordering::SeqCst),
            move || {
                c_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        engine.tick(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        flag.store(true, Ordering::SeqCst);
        engine.tick(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        engine.tick(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // One-shot only
    }
}
