use std::time::Duration;

use tokio::time::{Interval, interval};

use super::wave::WaveModulator;

pub struct RateController {
    interval: Interval,
    eps: f64,
    wave: Option<WaveModulator>,
    ticks_since_update: u32,
}

const WAVE_UPDATE_TICKS: u32 = 50;

impl RateController {
    pub fn new(eps: f64) -> Self {
        let duration = Duration::from_secs_f64(1.0 / eps);
        Self {
            interval: interval(duration),
            eps,
            wave: None,
            ticks_since_update: 0,
        }
    }

    pub fn with_wave(mut self, wave: WaveModulator) -> Self {
        // Set initial EPS from wave
        let initial_eps = wave.current_eps().max(0.1);
        self.eps = initial_eps;
        self.interval = interval(Duration::from_secs_f64(1.0 / initial_eps));
        self.wave = Some(wave);
        self
    }

    pub async fn tick(&mut self) {
        self.interval.tick().await;

        // Periodically update the interval based on wave modulation
        if let Some(ref wave) = self.wave {
            self.ticks_since_update += 1;
            if self.ticks_since_update >= WAVE_UPDATE_TICKS {
                self.ticks_since_update = 0;
                let new_eps = wave.current_eps().max(0.1); // Floor at 0.1 to avoid zero/negative
                if (new_eps - self.eps).abs() > 0.5 {
                    self.eps = new_eps;
                    let new_duration = Duration::from_secs_f64(1.0 / new_eps);
                    self.interval = interval(new_duration);
                }
            }
        }
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
