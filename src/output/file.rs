use anyhow::{Context, Result};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};

pub struct FileSink {
    writer: BufWriter<tokio::fs::File>,
    path: String,
    events_since_flush: u64,
}

const FLUSH_EVERY: u64 = 100;

impl FileSink {
    pub async fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .with_context(|| format!("failed to open file: {path}"))?;

        Ok(Self {
            writer: BufWriter::new(file),
            path: path.to_string(),
            events_since_flush: 0,
        })
    }

    pub async fn send(&mut self, event: &str) -> Result<()> {
        self.writer.write_all(event.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.events_since_flush += 1;

        if self.events_since_flush >= FLUSH_EVERY {
            self.flush().await?;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await?;
        self.events_since_flush = 0;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.flush().await?;
        self.writer.shutdown().await?;
        Ok(())
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
