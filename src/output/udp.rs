use anyhow::{Context, Result};
use tokio::net::UdpSocket;

pub struct UdpSink {
    socket: UdpSocket,
    target: String,
}

impl UdpSink {
    pub async fn new(host: &str, port: u16) -> Result<Self> {
        let target = format!("{host}:{port}");
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .context("failed to bind UDP socket")?;

        socket
            .connect(&target)
            .await
            .with_context(|| format!("failed to connect UDP to {target}"))?;

        Ok(Self { socket, target })
    }

    pub async fn send(&mut self, event: &str) -> Result<()> {
        let data = format!("{event}\n");
        self.socket.send(data.as_bytes()).await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}
