//! High-Precision Animation Clock with EMA refresh rate smoothing.

use std::time::Instant;

/// Monotonic animation clock with EMA refresh rate calculation.
#[derive(Debug)]
pub struct AnimationClock {
    last_present: Option<Instant>,
    dt: f32,
    refresh_hz: f32,
    total_time: f64,
}

impl Default for AnimationClock {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationClock {
    pub fn new() -> Self {
        Self {
            last_present: None,
            dt: 1.0 / 60.0,
            refresh_hz: 60.0,
            total_time: 0.0,
        }
    }

    /// Advances the clock from an explicit hardware present timestamp or instant.
    pub fn tick(&mut self, now: Instant) -> f32 {
        if let Some(last) = self.last_present {
            let elapsed = now.duration_since(last).as_secs_f32();

            // Clamp: reject system stalls (>100ms) and noise (<1ms)
            if (0.001..=0.100).contains(&elapsed) {
                self.dt = elapsed;
                self.total_time += elapsed as f64;

                // Exponential moving average (alpha = 0.05)
                let instant_hz = 1.0 / elapsed;
                self.refresh_hz = self.refresh_hz * 0.95 + instant_hz * 0.05;
            }
        }
        self.last_present = Some(now);
        self.dt
    }

    #[inline]
    pub fn dt(&self) -> f32 {
        self.dt
    }

    #[inline]
    pub fn refresh_hz(&self) -> f32 {
        self.refresh_hz
    }

    #[inline]
    pub fn total_time(&self) -> f64 {
        self.total_time
    }
}
