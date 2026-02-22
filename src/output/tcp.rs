use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct TcpSink {
    stream: Option<TcpStream>,
    host: String,
    port: u16,
}

const MAX_RETRIES: u32 = 3;
const BASE_BACKOFF_MS: u64 = 1000;

impl TcpSink {
    pub async fn new(host: &str, port: u16) -> Result<Self> {
        let addr = format!("{host}:{port}");
        let stream = TcpStream::connect(&addr)
            .await
            .with_context(|| format!("failed to connect to TCP {addr}"))?;

        Ok(Self {
            stream: Some(stream),
            host: host.to_string(),
            port,
        })
    }

    pub async fn send(&mut self, event: &str) -> Result<()> {
        let data = format!("{event}\n");

        if let Some(ref mut stream) = self.stream {
            match stream.write_all(data.as_bytes()).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    eprintln!(
                        "[tcp://{}:{}] write failed: {e}, attempting reconnect",
                        self.host, self.port
                    );
                    self.stream = None;
                }
            }
        }

        // Reconnect with exponential backoff
        self.reconnect().await?;

        if let Some(ref mut stream) = self.stream {
            stream.write_all(data.as_bytes()).await?;
        }

        Ok(())
    }

    async fn reconnect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);

        for attempt in 0..MAX_RETRIES {
            let backoff = Duration::from_millis(BASE_BACKOFF_MS * 2u64.pow(attempt));
            eprintln!(
                "[tcp://{addr}] reconnect attempt {} in {backoff:?}",
                attempt + 1
            );
            tokio::time::sleep(backoff).await;

            match TcpStream::connect(&addr).await {
                Ok(stream) => {
                    eprintln!("[tcp://{addr}] reconnected");
                    self.stream = Some(stream);
                    return Ok(());
                }
                Err(e) => {
                    eprintln!(
                        "[tcp://{addr}] reconnect attempt {} failed: {e}",
                        attempt + 1
                    );
                }
            }
        }

        anyhow::bail!("failed to reconnect to TCP {addr} after {MAX_RETRIES} attempts")
    }

    pub async fn flush(&mut self) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.flush().await?;
        }
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.shutdown().await?;
        }
        Ok(())
    }
}
