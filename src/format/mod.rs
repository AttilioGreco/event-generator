pub mod cef;
pub mod clf;
pub mod java_log4j;
pub mod json_structured;
pub mod leef;
pub mod syslog_rfc3164;
pub mod syslog_rfc5424;
pub mod template;

use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::config::model::FormatConfig;
use crate::data::fake::FakeDataProvider;

pub struct EventContext {
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub stream_name: String,
    pub fields: HashMap<String, String>,
}

impl EventContext {
    pub fn new(stream_name: String, sequence: u64) -> Self {
        let mut fields = HashMap::new();
        FakeDataProvider::populate(&mut fields);

        Self {
            timestamp: Utc::now(),
            sequence,
            stream_name,
            fields,
        }
    }
}

pub enum LogFormatter {
    SyslogRfc5424(syslog_rfc5424::SyslogRfc5424Formatter),
    SyslogRfc3164(syslog_rfc3164::SyslogRfc3164Formatter),
    Cef(cef::CefFormatter),
    Leef(leef::LeefFormatter),
    Clf(clf::ClfFormatter),
    Json(json_structured::JsonStructuredFormatter),
    JavaLog4j(java_log4j::JavaLog4jFormatter),
    JavaLogback(java_log4j::JavaLogbackFormatter),
    Template(template::TemplateFormatter),
}

impl LogFormatter {
    pub fn format(&self, ctx: &EventContext) -> String {
        match self {
            Self::SyslogRfc5424(f) => f.format(ctx),
            Self::SyslogRfc3164(f) => f.format(ctx),
            Self::Cef(f) => f.format(ctx),
            Self::Leef(f) => f.format(ctx),
            Self::Clf(f) => f.format(ctx),
            Self::Json(f) => f.format(ctx),
            Self::JavaLog4j(f) => f.format(ctx),
            Self::JavaLogback(f) => f.format(ctx),
            Self::Template(f) => f.format(ctx),
        }
    }
}

pub fn build_formatter(config: &FormatConfig) -> Result<LogFormatter> {
    match config.format_type.as_str() {
        "syslog_rfc5424" => Ok(LogFormatter::SyslogRfc5424(
            syslog_rfc5424::SyslogRfc5424Formatter::new(config),
        )),
        "syslog_rfc3164" => Ok(LogFormatter::SyslogRfc3164(
            syslog_rfc3164::SyslogRfc3164Formatter::new(config),
        )),
        "cef" => Ok(LogFormatter::Cef(cef::CefFormatter::new(config))),
        "leef" => Ok(LogFormatter::Leef(leef::LeefFormatter::new(config))),
        "clf" => Ok(LogFormatter::Clf(clf::ClfFormatter::new(config))),
        "json" => Ok(LogFormatter::Json(
            json_structured::JsonStructuredFormatter::new(config),
        )),
        "java_log4j" => Ok(LogFormatter::JavaLog4j(
            java_log4j::JavaLog4jFormatter::new(config),
        )),
        "java_logback" => Ok(LogFormatter::JavaLogback(
            java_log4j::JavaLogbackFormatter::new(config),
        )),
        "template" => {
            if let Some(inline) = &config.template_inline {
                Ok(LogFormatter::Template(
                    template::TemplateFormatter::from_inline(inline)?,
                ))
            } else if let Some(path) = &config.template_file {
                Ok(LogFormatter::Template(
                    template::TemplateFormatter::from_file(path)?,
                ))
            } else {
                anyhow::bail!("template format requires 'template_inline' or 'template_file'")
            }
        }
        other => anyhow::bail!("format type '{other}' not yet implemented"),
    }
}
