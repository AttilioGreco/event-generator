use std::path::PathBuf;

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

        let name = stream_config.name.clone();
        let token = cancel.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = run_stream(name.clone(), formatter, sink, rate, token).await {
                eprintln!("[{name}] stream error: {e}");
            }
        });

        handles.push(handle);
    }

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

    eprintln!("Done.");
    Ok(())
}
