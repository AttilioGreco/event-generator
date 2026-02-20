pub mod file;
pub mod http;
pub mod stdout;
pub mod tcp;
pub mod udp;

use anyhow::Result;

use crate::config::model::OutputConfig;

pub enum OutputSink {
    Stdout(stdout::StdoutSink),
    File(file::FileSink),
    Tcp(tcp::TcpSink),
    Udp(udp::UdpSink),
    Http(http::HttpSink),
}

impl OutputSink {
    pub async fn send(&mut self, event: &str) -> Result<()> {
        match self {
            Self::Stdout(s) => s.send(event).await,
            Self::File(s) => s.send(event).await,
            Self::Tcp(s) => s.send(event).await,
            Self::Udp(s) => s.send(event).await,
            Self::Http(s) => s.send(event).await,
        }
    }

    pub async fn flush(&mut self) -> Result<()> {
        match self {
            Self::Stdout(s) => s.flush().await,
            Self::File(s) => s.flush().await,
            Self::Tcp(s) => s.flush().await,
            Self::Udp(s) => s.flush().await,
            Self::Http(s) => s.flush().await,
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        match self {
            Self::Stdout(s) => s.close().await,
            Self::File(s) => s.close().await,
            Self::Tcp(s) => s.close().await,
            Self::Udp(s) => s.close().await,
            Self::Http(s) => s.close().await,
        }
    }
}

pub async fn build_sink(config: &OutputConfig) -> Result<OutputSink> {
    match config.output_type.as_str() {
        "stdout" => Ok(OutputSink::Stdout(stdout::StdoutSink)),
        "file" => {
            let path = config
                .path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("file output requires 'path'"))?;
            Ok(OutputSink::File(file::FileSink::new(path).await?))
        }
        "tcp" => {
            let host = config
                .host
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("tcp output requires 'host'"))?;
            let port = config
                .port
                .ok_or_else(|| anyhow::anyhow!("tcp output requires 'port'"))?;
            Ok(OutputSink::Tcp(tcp::TcpSink::new(host, port).await?))
        }
        "udp" => {
            let host = config
                .host
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("udp output requires 'host'"))?;
            let port = config
                .port
                .ok_or_else(|| anyhow::anyhow!("udp output requires 'port'"))?;
            Ok(OutputSink::Udp(udp::UdpSink::new(host, port).await?))
        }
        "http" => {
            let url = config
                .url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("http output requires 'url'"))?;
            Ok(OutputSink::Http(http::HttpSink::new(
                url,
                config.method.as_deref(),
                config.headers.as_ref(),
                config.batch_size,
                config.timeout_ms,
            )?))
        }
        other => anyhow::bail!("output type '{other}' not yet implemented"),
    }
}
