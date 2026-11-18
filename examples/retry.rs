#![allow(dead_code)]

use rand::RngExt;
use std::time::Duration;

trait Retryable {
    fn is_retryable(&self) -> bool;
}

struct BackoffPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
        }
    }
}

impl BackoffPolicy {
    fn delay_for(&self, attempt: u32) -> Duration {
        let exp_ms =
            (self.base_delay.as_millis() as u64).saturating_mul(2u64.saturating_pow(attempt));
        let capped_ms = exp_ms.min(self.max_delay.as_millis() as u64);

        let jitter_factor = rand::rng().random_range(0.75..1.25);
        let jittered_ms =
            ((capped_ms as f64 * jitter_factor) as u64).min(self.max_delay.as_millis() as u64);

        Duration::from_millis(jittered_ms)
    }
}

fn main() {}
