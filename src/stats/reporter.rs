use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

pub struct StreamStats {
    pub name: String,
    pub destination: String,
    pub events_since_last: AtomicU64,
    pub total_events: AtomicU64,
}

impl StreamStats {
    pub fn new(name: String, destination: String) -> Self {
        Self {
            name,
            destination,
            events_since_last: AtomicU64::new(0),
            total_events: AtomicU64::new(0),
        }
    }

    pub fn record_event(&self) {
        self.events_since_last.fetch_add(1, Ordering::Relaxed);
        self.total_events.fetch_add(1, Ordering::Relaxed);
    }
}

pub async fn run_stats_reporter(
    streams: Vec<Arc<StreamStats>>,
    interval_secs: u64,
    cancel: CancellationToken,
) {
    let start = Instant::now();
    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
    ticker.tick().await; // First tick is immediate, skip it

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                print_stats(&streams, start.elapsed());
                return;
            }
            _ = ticker.tick() => {
                print_stats(&streams, start.elapsed());
            }
        }
    }
}

fn print_stats(streams: &[Arc<StreamStats>], elapsed: Duration) {
    let elapsed_secs = elapsed.as_secs();
    let hours = elapsed_secs / 3600;
    let minutes = (elapsed_secs % 3600) / 60;
    let seconds = elapsed_secs % 60;

    let mut parts = Vec::new();
    let mut total_recent: u64 = 0;
    let mut total_all: u64 = 0;

    for stream in streams {
        let recent = stream.events_since_last.swap(0, Ordering::Relaxed);
        let total = stream.total_events.load(Ordering::Relaxed);
        total_recent += recent;
        total_all += total;
        parts.push(format!(
            "{}: {} eps ({} total)",
            stream.name, recent, total
        ));
    }

    let streams_str = parts.join(" | ");
    eprintln!(
        "[stats] {:02}:{:02}:{:02} | {streams_str} | TOTAL: {total_recent} eps ({total_all} total)",
        hours, minutes, seconds
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_event_increments_counters() {
        let stats = StreamStats::new("test".into(), "stdout".into());
        stats.record_event();
        stats.record_event();
        stats.record_event();

        assert_eq!(stats.total_events.load(Ordering::Relaxed), 3);
        assert_eq!(stats.events_since_last.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn swap_resets_recent_counter() {
        let stats = StreamStats::new("test".into(), "stdout".into());
        stats.record_event();
        stats.record_event();

        let recent = stats.events_since_last.swap(0, Ordering::Relaxed);
        assert_eq!(recent, 2);
        assert_eq!(stats.events_since_last.load(Ordering::Relaxed), 0);
        assert_eq!(stats.total_events.load(Ordering::Relaxed), 2);
    }
}
