use anyhow::Result;
use tokio_util::sync::CancellationToken;

use crate::format::{EventContext, LogFormatter};
use crate::output::OutputSink;

use super::rate::RateController;

pub async fn run_stream(
    name: String,
    formatter: LogFormatter,
    mut sink: OutputSink,
    mut rate: RateController,
    cancel: CancellationToken,
) -> Result<()> {
    let mut sequence: u64 = 0;

    eprintln!("[{name}] started at {} eps", rate.eps());

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                eprintln!("[{name}] shutting down after {sequence} events");
                sink.flush().await?;
                sink.close().await?;
                return Ok(());
            }
            _ = rate.tick() => {
                let ctx = EventContext::new(name.clone(), sequence);
                let event = formatter.format(&ctx);

                if let Err(e) = sink.send(&event).await {
                    eprintln!("[{name}] send error: {e}");
                }

                sequence += 1;
            }
        }
    }
}
