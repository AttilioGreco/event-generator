use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub defaults: Option<Defaults>,
    pub web: Option<WebConfig>,
    pub streams: Vec<StreamConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default)]
    pub auto_open_browser: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}

fn default_listen() -> String {
    "0.0.0.0:8080".into()
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct StreamConfig {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub format: FormatConfig,
    pub output: OutputConfig,
    pub rate: Option<RateConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FormatConfig {
    #[serde(rename = "type")]
    pub format_type: String,

    // Syslog options
    pub facility: Option<String>,
    pub severity: Option<String>,
    pub app_name: Option<String>,

    // CEF/LEEF options
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub device_event_class_id: Option<String>,

    // JSON options
    pub extra_fields: Option<HashMap<String, String>>,

    // Template options (Phase 4)
    pub template_file: Option<String>,
    pub template_inline: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(rename = "type")]
    pub output_type: String,

    // File options
    pub path: Option<String>,

    // TCP/UDP options
    pub host: Option<String>,
    pub port: Option<u16>,

    // HTTP options (Phase 6)
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub batch_size: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RateConfig {
    pub eps: Option<f64>,
    pub wave: Option<WaveConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WaveConfig {
    pub shape: String,
    pub period_secs: f64,
    pub min: f64,
    pub max: f64,
}

fn default_true() -> bool {
    true
}

impl AppConfig {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}
