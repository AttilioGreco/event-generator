use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::config::model::{AppConfig, StreamConfig};
use crate::config::validate::validate;
use crate::engine::rate::RateController;
use crate::engine::stream::run_stream;
use crate::engine::wave::{WaveModulator, WaveShape};
use crate::format::build_formatter;
use crate::output::build_sink;
use crate::stats::reporter::StreamStats;

/// Per-stream status visible to the UI.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamStatus {
    Running,
    Stopped,
    Error,
}

struct StreamHandle {
    join_handle: Option<JoinHandle<()>>,
    cancel_token: CancellationToken,
    stats: Arc<StreamStats>,
    status: StreamStatus,
    /// Last error message (set when build/spawn fails).
    error: Option<String>,
}

struct StreamManagerInner {
    stream_order: Vec<String>,
    streams: HashMap<String, StreamHandle>,
    config_text: String,
    config: AppConfig,
    global_cancel: CancellationToken,
}

#[derive(Clone)]
pub struct StreamManager {
    inner: Arc<RwLock<StreamManagerInner>>,
}

/// Info returned from stream_infos(), carries the Arc<StreamStats>.
pub struct StreamInfo {
    pub name: String,
    pub destination: String,
    pub status: StreamStatus,
    pub error: Option<String>,
    pub stats: Arc<StreamStats>,
}

fn destination_label(config: &crate::config::model::OutputConfig) -> String {
    match config.output_type.as_str() {
        "file" => format!("file:{}", config.path.as_deref().unwrap_or("?")),
        "tcp" => format!(
            "tcp://{}:{}",
            config.host.as_deref().unwrap_or("?"),
            config.port.unwrap_or(0)
        ),
        "udp" => format!(
            "udp://{}:{}",
            config.host.as_deref().unwrap_or("?"),
            config.port.unwrap_or(0)
        ),
        "http" => config.url.clone().unwrap_or_else(|| "http://?".into()),
        other => other.into(),
    }
}

impl StreamManager {
    pub fn new(global_cancel: CancellationToken) -> Self {
        Self {
            inner: Arc::new(RwLock::new(StreamManagerInner {
                stream_order: Vec::new(),
                streams: HashMap::new(),
                config_text: String::new(),
                config: AppConfig {
                    defaults: None,
                    web: None,
                    streams: Vec::new(),
                },
                global_cancel,
            })),
        }
    }

    /// Parse, validate, and start all enabled streams.
    /// Streams that fail to start are marked as error — the system continues.
    pub async fn load_and_start(&self, config_text: String) -> Result<()> {
        let config =
            AppConfig::from_toml(&config_text).with_context(|| "failed to parse TOML config")?;
        validate(&config).with_context(|| "config validation failed")?;

        let default_eps = config
            .defaults
            .as_ref()
            .and_then(|d| d.rate)
            .unwrap_or(10.0);

        let stream_order: Vec<String> = config.streams.iter().map(|s| s.name.clone()).collect();

        // Try to start each enabled stream independently
        let mut entries: Vec<(String, StreamHandle)> = Vec::new();
        for sc in &config.streams {
            if sc.enabled {
                let handle = self.try_build_and_spawn(sc, default_eps).await;
                entries.push((sc.name.clone(), handle));
            } else {
                entries.push((sc.name.clone(), self.make_stopped_handle(sc).await));
            }
        }

        let mut inner = self.inner.write().await;
        inner.config_text = config_text;
        inner.config = config;
        inner.stream_order = stream_order;
        inner.streams.clear();
        for (name, handle) in entries {
            inner.streams.insert(name, handle);
        }

        Ok(())
    }

    /// Stop a single stream by name.
    pub async fn stop_stream(&self, name: &str) -> Result<()> {
        let mut inner = self.inner.write().await;
        let handle = inner
            .streams
            .get_mut(name)
            .ok_or_else(|| anyhow!("stream '{}' not found", name))?;

        if handle.status != StreamStatus::Running {
            return Ok(());
        }

        handle.cancel_token.cancel();
        if let Some(jh) = handle.join_handle.take() {
            let _ = jh.await;
        }
        handle.status = StreamStatus::Stopped;
        handle.error = None;
        eprintln!("[manager] stopped stream '{name}'");
        Ok(())
    }

    /// Start a previously stopped/errored stream by name.
    pub async fn start_stream(&self, name: &str) -> Result<()> {
        let (stream_config, default_eps, current_status) = {
            let inner = self.inner.read().await;
            let sc = inner
                .config
                .streams
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| anyhow!("stream '{}' not found in config", name))?
                .clone();
            let default_eps = inner
                .config
                .defaults
                .as_ref()
                .and_then(|d| d.rate)
                .unwrap_or(10.0);
            let status = inner
                .streams
                .get(name)
                .map(|h| h.status.clone())
                .unwrap_or(StreamStatus::Stopped);
            (sc, default_eps, status)
        };

        if current_status == StreamStatus::Running {
            return Ok(());
        }

        let new_handle = self.try_build_and_spawn(&stream_config, default_eps).await;
        let started = new_handle.status == StreamStatus::Running;

        let mut inner = self.inner.write().await;
        inner.streams.insert(name.to_string(), new_handle);

        if !started {
            // Return error so the API can report it, but the stream is still in the map as Error
            let err_msg = inner
                .streams
                .get(name)
                .and_then(|h| h.error.clone())
                .unwrap_or_else(|| "unknown error".into());
            return Err(anyhow!("{}", err_msg));
        }

        Ok(())
    }

    /// Stop all running streams.
    pub async fn stop_all(&self) -> Result<()> {
        let names: Vec<String> = {
            let inner = self.inner.read().await;
            inner
                .streams
                .iter()
                .filter(|(_, h)| h.status == StreamStatus::Running)
                .map(|(n, _)| n.clone())
                .collect()
        };

        for name in names {
            self.stop_stream(&name).await?;
        }
        Ok(())
    }

    /// Start all configured (enabled) streams that are currently stopped/errored.
    /// Errors are logged per-stream but do not stop other streams from starting.
    pub async fn start_all(&self) -> Result<()> {
        let to_start: Vec<(String, StreamConfig, f64)> = {
            let inner = self.inner.read().await;
            let default_eps = inner
                .config
                .defaults
                .as_ref()
                .and_then(|d| d.rate)
                .unwrap_or(10.0);
            inner
                .config
                .streams
                .iter()
                .filter(|s| s.enabled)
                .filter(|s| {
                    inner
                        .streams
                        .get(&s.name)
                        .map(|h| h.status != StreamStatus::Running)
                        .unwrap_or(true)
                })
                .map(|s| (s.name.clone(), s.clone(), default_eps))
                .collect()
        };

        for (name, sc, default_eps) in to_start {
            let new_handle = self.try_build_and_spawn(&sc, default_eps).await;
            if new_handle.status != StreamStatus::Running {
                eprintln!(
                    "[manager] stream '{}' failed to start: {}",
                    name,
                    new_handle.error.as_deref().unwrap_or("unknown")
                );
            }
            let mut inner = self.inner.write().await;
            inner.streams.insert(name, new_handle);
        }
        Ok(())
    }

    /// Validate and apply a new TOML config.
    /// Stops all current streams, replaces config, starts new ones.
    /// Individual stream failures are logged but don't abort the whole operation.
    pub async fn apply_config(&self, new_toml: String) -> Result<()> {
        let new_config =
            AppConfig::from_toml(&new_toml).with_context(|| "failed to parse TOML config")?;
        validate(&new_config).with_context(|| "config validation failed")?;

        // Stop all current streams
        self.stop_all().await?;

        let default_eps = new_config
            .defaults
            .as_ref()
            .and_then(|d| d.rate)
            .unwrap_or(10.0);

        let stream_order: Vec<String> = new_config.streams.iter().map(|s| s.name.clone()).collect();

        let mut entries: Vec<(String, StreamHandle)> = Vec::new();
        for sc in &new_config.streams {
            if sc.enabled {
                let handle = self.try_build_and_spawn(sc, default_eps).await;
                entries.push((sc.name.clone(), handle));
            } else {
                entries.push((sc.name.clone(), self.make_stopped_handle(sc).await));
            }
        }

        let mut inner = self.inner.write().await;
        inner.config_text = new_toml;
        inner.config = new_config;
        inner.stream_order = stream_order;
        inner.streams.clear();
        for (name, handle) in entries {
            inner.streams.insert(name, handle);
        }

        eprintln!("[manager] config reloaded successfully");
        Ok(())
    }

    /// Get the current raw TOML config text.
    pub async fn config_text(&self) -> String {
        self.inner.read().await.config_text.clone()
    }

    /// Get a snapshot of all streams with their stats (ordered by config).
    pub async fn stream_infos(&self) -> Vec<StreamInfo> {
        let inner = self.inner.read().await;
        inner
            .stream_order
            .iter()
            .filter_map(|name| {
                inner.streams.get(name).map(|h| StreamInfo {
                    name: name.clone(),
                    destination: h.stats.destination.clone(),
                    status: h.status.clone(),
                    error: h.error.clone(),
                    stats: Arc::clone(&h.stats),
                })
            })
            .collect()
    }

    /// Get a specific stream's stats by name.
    pub async fn stream_by_name(&self, name: &str) -> Option<Arc<StreamStats>> {
        let inner = self.inner.read().await;
        inner.streams.get(name).map(|h| Arc::clone(&h.stats))
    }

    /// Wait for all running stream tasks to complete (used during shutdown).
    pub async fn wait_all(&self) {
        let handles: Vec<JoinHandle<()>> = {
            let mut inner = self.inner.write().await;
            inner
                .streams
                .values_mut()
                .filter_map(|h| h.join_handle.take())
                .collect()
        };

        for handle in handles {
            let _ = handle.await;
        }
    }

    /// Get the count of currently running streams.
    pub async fn running_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner
            .streams
            .values()
            .filter(|h| h.status == StreamStatus::Running)
            .count()
    }

    // --- internal helpers ---

    /// Try to build and spawn a stream. On failure, returns a handle with Error status.
    async fn try_build_and_spawn(
        &self,
        stream_config: &StreamConfig,
        default_eps: f64,
    ) -> StreamHandle {
        match self
            .build_and_spawn_stream(stream_config, default_eps)
            .await
        {
            Ok(handle) => handle,
            Err(e) => {
                let err_msg = format!("{e:#}");
                eprintln!(
                    "[manager] stream '{}' failed to start: {err_msg}",
                    stream_config.name
                );
                let dest = destination_label(&stream_config.output);
                let stats =
                    Arc::new(StreamStats::new(stream_config.name.clone(), dest));
                let token = {
                    let inner = self.inner.read().await;
                    inner.global_cancel.child_token()
                };
                StreamHandle {
                    join_handle: None,
                    cancel_token: token,
                    stats,
                    status: StreamStatus::Error,
                    error: Some(err_msg),
                }
            }
        }
    }

    /// Create a stopped placeholder handle (for disabled streams).
    async fn make_stopped_handle(&self, sc: &StreamConfig) -> StreamHandle {
        let dest = destination_label(&sc.output);
        let stats = Arc::new(StreamStats::new(sc.name.clone(), dest));
        let token = {
            let inner = self.inner.read().await;
            inner.global_cancel.child_token()
        };
        StreamHandle {
            join_handle: None,
            cancel_token: token,
            stats,
            status: StreamStatus::Stopped,
            error: None,
        }
    }

    async fn build_and_spawn_stream(
        &self,
        stream_config: &StreamConfig,
        default_eps: f64,
    ) -> Result<StreamHandle> {
        let formatter = build_formatter(&stream_config.format).with_context(|| {
            format!(
                "failed to build formatter for stream '{}'",
                stream_config.name
            )
        })?;

        let sink = build_sink(&stream_config.output).await.with_context(|| {
            format!("failed to build sink for stream '{}'", stream_config.name)
        })?;

        let eps = stream_config
            .rate
            .as_ref()
            .and_then(|r| r.eps)
            .unwrap_or(default_eps);

        let mut rate = RateController::new(eps);

        if let Some(wave_config) = stream_config.rate.as_ref().and_then(|r| r.wave.as_ref()) {
            let shape = match wave_config.shape.as_str() {
                "sine" => WaveShape::Sine,
                "sawtooth" => WaveShape::Sawtooth,
                "square" => WaveShape::Square,
                other => anyhow::bail!("unknown wave shape: {other}"),
            };
            let wave = WaveModulator::new(
                shape,
                wave_config.period_secs,
                wave_config.min,
                wave_config.max,
            );
            rate = rate.with_wave(wave);
        }

        let destination = destination_label(&stream_config.output);
        let stats = Arc::new(StreamStats::new(stream_config.name.clone(), destination));

        let name = stream_config.name.clone();
        let cancel_token = {
            let inner = self.inner.read().await;
            inner.global_cancel.child_token()
        };
        let token_clone = cancel_token.clone();
        let stats_clone = Arc::clone(&stats);

        let join_handle = tokio::spawn(async move {
            if let Err(e) = run_stream(name.clone(), formatter, sink, rate, stats_clone, token_clone)
                .await
            {
                eprintln!("[{name}] stream error: {e}");
            }
        });

        Ok(StreamHandle {
            join_handle: Some(join_handle),
            cancel_token,
            stats,
            status: StreamStatus::Running,
            error: None,
        })
    }
}
