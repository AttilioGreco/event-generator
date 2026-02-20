use std::f64::consts::PI;
use std::time::{Duration, Instant};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WaveShape {
    Sine,
    Sawtooth,
    Square,
}

pub struct WaveModulator {
    shape: WaveShape,
    period: Duration,
    min_eps: f64,
    max_eps: f64,
    start_time: Instant,
}

impl WaveModulator {
    pub fn new(shape: WaveShape, period_secs: f64, min_eps: f64, max_eps: f64) -> Self {
        Self {
            shape,
            period: Duration::from_secs_f64(period_secs),
            min_eps,
            max_eps,
            start_time: Instant::now(),
        }
    }

    /// Returns the current target EPS based on elapsed time in the wave cycle.
    pub fn current_eps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let period_secs = self.period.as_secs_f64();
        let t = (elapsed % period_secs) / period_secs; // 0.0..1.0 position in cycle

        let normalized = match self.shape {
            WaveShape::Sine => (1.0 + (t * 2.0 * PI).sin()) / 2.0,
            WaveShape::Sawtooth => t,
            WaveShape::Square => {
                if t < 0.5 { 1.0 } else { 0.0 }
            }
        };

        self.min_eps + normalized * (self.max_eps - self.min_eps)
    }

    #[cfg(test)]
    fn with_start_time(mut self, start: Instant) -> Self {
        self.start_time = start;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_wave_midpoint_at_start() {
        let wave = WaveModulator::new(WaveShape::Sine, 10.0, 100.0, 1000.0);
        // At t=0, sine(0) = 0, normalized = 0.5, eps = 100 + 0.5 * 900 = 550
        let eps = wave.current_eps();
        // Allow some tolerance since time has elapsed slightly
        assert!((eps - 550.0).abs() < 50.0, "expected ~550, got {eps}");
    }

    #[test]
    fn sine_wave_known_points() {
        let start = Instant::now();
        let wave = WaveModulator::new(WaveShape::Sine, 4.0, 0.0, 100.0)
            .with_start_time(start - Duration::from_secs(1));
        // t=1s in 4s period -> t_norm=0.25, sin(PI/2)=1, normalized=1.0, eps=100
        let eps = wave.current_eps();
        assert!((eps - 100.0).abs() < 5.0, "at peak expected ~100, got {eps}");

        let wave = WaveModulator::new(WaveShape::Sine, 4.0, 0.0, 100.0)
            .with_start_time(start - Duration::from_secs(3));
        // t=3s in 4s period -> t_norm=0.75, sin(3PI/2)=-1, normalized=0.0, eps=0
        let eps = wave.current_eps();
        assert!((eps - 0.0).abs() < 5.0, "at trough expected ~0, got {eps}");
    }

    #[test]
    fn sawtooth_wave() {
        let start = Instant::now();

        let wave = WaveModulator::new(WaveShape::Sawtooth, 10.0, 0.0, 100.0)
            .with_start_time(start - Duration::from_millis(500));
        // t=0.5s in 10s -> t_norm=0.05, eps = 5
        let eps = wave.current_eps();
        assert!((eps - 5.0).abs() < 3.0, "expected ~5, got {eps}");

        let wave = WaveModulator::new(WaveShape::Sawtooth, 10.0, 0.0, 100.0)
            .with_start_time(start - Duration::from_secs(5));
        // t=5s in 10s -> t_norm=0.5, eps = 50
        let eps = wave.current_eps();
        assert!((eps - 50.0).abs() < 3.0, "expected ~50, got {eps}");
    }

    #[test]
    fn square_wave() {
        let start = Instant::now();

        let wave = WaveModulator::new(WaveShape::Square, 10.0, 100.0, 1000.0)
            .with_start_time(start - Duration::from_secs(2));
        // t=2s in 10s -> t_norm=0.2 < 0.5, normalized=1.0, eps=1000
        let eps = wave.current_eps();
        assert!((eps - 1000.0).abs() < 5.0, "first half expected ~1000, got {eps}");

        let wave = WaveModulator::new(WaveShape::Square, 10.0, 100.0, 1000.0)
            .with_start_time(start - Duration::from_secs(7));
        // t=7s in 10s -> t_norm=0.7 >= 0.5, normalized=0.0, eps=100
        let eps = wave.current_eps();
        assert!((eps - 100.0).abs() < 5.0, "second half expected ~100, got {eps}");
    }

    #[test]
    fn constant_when_min_equals_max() {
        let wave = WaveModulator::new(WaveShape::Sine, 10.0, 500.0, 500.0);
        for _ in 0..10 {
            let eps = wave.current_eps();
            assert!((eps - 500.0).abs() < 0.01, "expected constant 500, got {eps}");
        }
    }
}
