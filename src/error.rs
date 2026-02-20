use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(String),

    #[error("output error for sink '{sink}': {source}")]
    Output { sink: String, source: io::Error },

    #[error("format error: {0}")]
    Format(String),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}
