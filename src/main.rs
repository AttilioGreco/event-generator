use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use tokio_util::sync::CancellationToken;

use event_generator::config::model::AppConfig;
use event_generator::config::validate::validate;
use event_generator::engine::rate::RateController;
use event_generator::engine::stream::run_stream;
use event_generator::engine::wave::{WaveModulator, WaveShape};
use event_generator::format::build_formatter;
use event_generator::output::build_sink;
use event_generator::stats::reporter::{StreamStats, run_stats_reporter};
use event_generator::web::run_web_server;

#[derive(Parser)]
#[command(name = "event-generator", about = "Generate simulated log events")]
struct Cli {
    #[arg(short, long, default_value = "config/example.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_content = std::fs::read_to_string(&cli.config)
        .with_context(|| format!("failed to read config file: {}", cli.config.display()))?;

    let config = AppConfig::from_toml(&config_content)
        .with_context(|| "failed to parse TOML config")?;

    validate(&config).with_context(|| "config validation failed")?;

    let cancel = CancellationToken::new();
    let mut handles = Vec::new();
    let mut all_stats: Vec<Arc<StreamStats>> = Vec::new();

    for stream_config in config.streams.iter().filter(|s| s.enabled) {
        let formatter = build_formatter(&stream_config.format)
            .with_context(|| format!("failed to build formatter for stream '{}'", stream_config.name))?;

        let sink = build_sink(&stream_config.output).await
            .with_context(|| format!("failed to build sink for stream '{}'", stream_config.name))?;

        let default_eps = config.defaults.as_ref().and_then(|d| d.rate).unwrap_or(10.0);
        let eps = stream_config
            .rate
            .as_ref()
            .and_then(|r| r.eps)
            .unwrap_or(default_eps);

        let mut rate = RateController::new(eps);

        // Apply wave modulation if configured
        if let Some(wave_config) = stream_config.rate.as_ref().and_then(|r| r.wave.as_ref()) {
            let shape = match wave_config.shape.as_str() {
                "sine" => WaveShape::Sine,
                "sawtooth" => WaveShape::Sawtooth,
                "square" => WaveShape::Square,
                other => anyhow::bail!("unknown wave shape: {other}"),
            };
            let wave = WaveModulator::new(shape, wave_config.period_secs, wave_config.min, wave_config.max);
            rate = rate.with_wave(wave);
        }

        let destination = match stream_config.output.output_type.as_str() {
            "file" => format!("file:{}", stream_config.output.path.as_deref().unwrap_or("?")),
            "tcp" => format!("tcp://{}:{}", stream_config.output.host.as_deref().unwrap_or("?"), stream_config.output.port.unwrap_or(0)),
            "udp" => format!("udp://{}:{}", stream_config.output.host.as_deref().unwrap_or("?"), stream_config.output.port.unwrap_or(0)),
            "http" => stream_config.output.url.clone().unwrap_or_else(|| "http://?".into()),
            other => other.into(),
        };
        let stats = Arc::new(StreamStats::new(stream_config.name.clone(), destination));
        all_stats.push(Arc::clone(&stats));

        let name = stream_config.name.clone();
        let token = cancel.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = run_stream(name.clone(), formatter, sink, rate, stats, token).await {
                eprintln!("[{name}] stream error: {e}");
            }
        });

        handles.push(handle);
    }

    // Spawn stats reporter
    let stats_cancel = cancel.clone();
    let stats_for_reporter = all_stats.clone();
    let stats_handle = tokio::spawn(run_stats_reporter(stats_for_reporter, 5, stats_cancel));

    // Spawn web dashboard if configured
    let web_handle = if let Some(web_config) = config.web.as_ref().filter(|w| w.enabled) {
        let wc = web_config.clone();
        let ws = all_stats.clone();
        let wt = cancel.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = run_web_server(wc, ws, wt).await {
                eprintln!("[web] server error: {e}");
            }
        }))
    } else {
        None
    };

    eprintln!(
        "event-generator running with {} stream(s). Press Ctrl+C to stop.",
        handles.len()
    );

    tokio::signal::ctrl_c().await?;
    eprintln!("\nShutting down...");
    cancel.cancel();

    for handle in handles {
        let _ = handle.await;
    }
    let _ = stats_handle.await;
    if let Some(wh) = web_handle {
        let _ = wh.await;
    }

    eprintln!("Done.");
    Ok(())
}
