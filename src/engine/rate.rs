use std::time::Duration;

use tokio::time::{Interval, interval};

pub struct RateController {
    interval: Interval,
    eps: f64,
}

impl RateController {
    pub fn new(eps: f64) -> Self {
        let duration = Duration::from_secs_f64(1.0 / eps);
        Self {
            interval: interval(duration),
            eps,
        }
    }

    pub async fn tick(&mut self) {
        self.interval.tick().await;
    }

    pub fn eps(&self) -> f64 {
        self.eps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn interval_calculation() {
        let rate = RateController::new(100.0);
        assert_eq!(rate.eps(), 100.0);
    }

    #[tokio::test]
    async fn high_eps_interval() {
        let rate = RateController::new(10_000.0);
        assert_eq!(rate.eps(), 10_000.0);
    }

    #[tokio::test]
    async fn tick_produces_events_at_rate() {
        let mut rate = RateController::new(1000.0);
        let start = tokio::time::Instant::now();

        // Tick 100 times at 1000 EPS should take ~100ms
        for _ in 0..100 {
            rate.tick().await;
        }

        let elapsed = start.elapsed();
        // Allow generous tolerance for CI: 50ms to 300ms for what should be ~100ms
        assert!(
            elapsed >= Duration::from_millis(50),
            "too fast: {elapsed:?}"
        );
        assert!(
            elapsed <= Duration::from_millis(300),
            "too slow: {elapsed:?}"
        );
    }
}
