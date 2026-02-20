use anyhow::Result;
use tokio::io::{self, AsyncWriteExt};

pub struct StdoutSink;

impl StdoutSink {
    pub async fn send(&mut self, event: &str) -> Result<()> {
        let mut out = io::stdout();
        out.write_all(event.as_bytes()).await?;
        out.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        let mut out = io::stdout();
        out.flush().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.flush().await
    }
}
