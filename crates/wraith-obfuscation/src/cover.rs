//! Cover traffic generation.

use std::time::{Duration, Instant};

/// Cover traffic generator
pub struct CoverTrafficGenerator {
    /// Minimum packets per second
    pub min_rate: u32,
    /// Maximum time between packets
    pub max_idle: Duration,
    /// Last send time
    last_send: Instant,
}

impl CoverTrafficGenerator {
    /// Create a new cover traffic generator
    pub fn new(min_rate: u32, max_idle: Duration) -> Self {
        Self {
            min_rate,
            max_idle,
            last_send: Instant::now(),
        }
    }

    /// Check if cover traffic should be sent
    pub fn should_send_cover(&self, pending_data: bool) -> bool {
        if pending_data {
            return false;
        }
        self.last_send.elapsed() > self.max_idle
    }

    /// Mark that a packet was sent
    pub fn packet_sent(&mut self) {
        self.last_send = Instant::now();
    }
}

impl Default for CoverTrafficGenerator {
    fn default() -> Self {
        Self::new(10, Duration::from_millis(100))
    }
}
