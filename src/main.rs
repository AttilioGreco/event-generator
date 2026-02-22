use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tokio_util::sync::CancellationToken;

use event_generator::config::model::AppConfig;
use event_generator::config::validate::validate;
use event_generator::engine::manager::StreamManager;
use event_generator::stats::reporter::run_stats_reporter_dynamic;
use event_generator::web::run_web_server;

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\ngit:   ",
    env!("BUILD_GIT_SHA"),
    " (",
    env!("BUILD_GIT_TAG"),
    ")",
    "\ndate:  ",
    env!("BUILD_DATE"),
    "\nrustc: ",
    env!("BUILD_RUSTC"),
);

#[derive(Parser)]
#[command(name = "event-generator", about = "Generate simulated log events", version, long_version = LONG_VERSION)]
struct Cli {
    #[arg(short, long, default_value = "config/example.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_content = std::fs::read_to_string(&cli.config)
        .with_context(|| format!("failed to read config file: {}", cli.config.display()))?;

    // Validate config early to fail fast
    let config =
        AppConfig::from_toml(&config_content).with_context(|| "failed to parse TOML config")?;
    validate(&config).with_context(|| "config validation failed")?;

    let cancel = CancellationToken::new();

    // Create stream manager and load config
    let manager = StreamManager::new(cancel.clone());
    manager.load_and_start(config_content).await?;

    let running = manager.running_count().await;

    // Spawn stats reporter
    let stats_handle = tokio::spawn(run_stats_reporter_dynamic(
        manager.clone(),
        5,
        cancel.clone(),
    ));

    // Spawn web dashboard if configured
    let web_handle = if let Some(web_config) = config.web.as_ref().filter(|w| w.enabled) {
        let wc = web_config.clone();
        let wm = manager.clone();
        let wt = cancel.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = run_web_server(wc, wm, wt).await {
                eprintln!("[web] server error: {e}");
            }
        }))
    } else {
        None
    };

    eprintln!(
        "event-generator running with {} stream(s). Press Ctrl+C to stop.",
        running
    );

    tokio::signal::ctrl_c().await?;
    eprintln!("\nShutting down...");
    cancel.cancel();

    manager.wait_all().await;
    let _ = stats_handle.await;
    if let Some(wh) = web_handle {
        let _ = wh.await;
    }

    eprintln!("Done.");
    Ok(())
}
