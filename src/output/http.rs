use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};

pub struct HttpSink {
    client: Client,
    url: String,
    method: Method,
    headers: HeaderMap,
    batch_size: usize,
    buffer: Vec<String>,
}

impl HttpSink {
    pub fn new(
        url: &str,
        method: Option<&str>,
        headers: Option<&HashMap<String, String>>,
        batch_size: Option<usize>,
        timeout_ms: Option<u64>,
    ) -> Result<Self> {
        let timeout = Duration::from_millis(timeout_ms.unwrap_or(5000));
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("failed to build HTTP client")?;

        let method = match method.unwrap_or("POST").to_uppercase().as_str() {
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            other => anyhow::bail!("unsupported HTTP method: {other}"),
        };

        let mut header_map = HeaderMap::new();
        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                let name = HeaderName::from_bytes(k.as_bytes())
                    .with_context(|| format!("invalid header name: {k}"))?;
                let value = HeaderValue::from_str(v)
                    .with_context(|| format!("invalid header value for {k}"))?;
                header_map.insert(name, value);
            }
        }

        Ok(Self {
            client,
            url: url.to_string(),
            method,
            headers: header_map,
            batch_size: batch_size.unwrap_or(10),
            buffer: Vec::new(),
        })
    }

    pub async fn send(&mut self, event: &str) -> Result<()> {
        self.buffer.push(event.to_string());

        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let body = self.buffer.join("\n");
        self.buffer.clear();

        let request = self
            .client
            .request(self.method.clone(), &self.url)
            .headers(self.headers.clone())
            .body(body);

        match request.send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!("[{}] server returned {}", self.url, response.status());
                }
            }
            Err(e) => {
                eprintln!("[{}] request failed: {e}", self.url);
            }
        }

        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.flush().await
    }
}
