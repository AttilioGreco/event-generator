use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

const MAX_RECENT_EVENTS: usize = 1000;
const BROADCAST_CAPACITY: usize = 1024;

pub struct StreamStats {
    pub name: String,
    pub destination: String,
    pub events_since_last: AtomicU64,
    pub total_events: AtomicU64,
    recent_events: Mutex<std::collections::VecDeque<String>>,
    event_tx: broadcast::Sender<String>,
}

impl StreamStats {
    pub fn new(name: String, destination: String) -> Self {
        let (event_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            name,
            destination,
            events_since_last: AtomicU64::new(0),
            total_events: AtomicU64::new(0),
            recent_events: Mutex::new(std::collections::VecDeque::with_capacity(MAX_RECENT_EVENTS)),
            event_tx,
        }
    }

    pub fn record_event(&self) {
        self.record_event_with_payload("");
    }

    pub fn record_event_with_payload(&self, event: &str) {
        self.events_since_last.fetch_add(1, Ordering::Relaxed);
        self.total_events.fetch_add(1, Ordering::Relaxed);

        if event.is_empty() {
            return;
        }

        if let Ok(mut recent) = self.recent_events.lock() {
            if recent.len() >= MAX_RECENT_EVENTS {
                recent.pop_front();
            }
            recent.push_back(event.to_string());
        }

        let _ = self.event_tx.send(event.to_string());
    }

    pub fn recent_events_snapshot(&self) -> Vec<String> {
        if let Ok(recent) = self.recent_events.lock() {
            return recent.iter().cloned().collect();
        }
        Vec::new()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<String> {
        self.event_tx.subscribe()
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
