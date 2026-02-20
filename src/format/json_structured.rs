use std::collections::HashMap;

use super::EventContext;
use crate::config::model::FormatConfig;

pub struct JsonStructuredFormatter {
    extra_fields: HashMap<String, String>,
}

impl JsonStructuredFormatter {
    pub fn new(config: &FormatConfig) -> Self {
        Self {
            extra_fields: config.extra_fields.clone().unwrap_or_default(),
        }
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let level = ctx.fields.get("log_level").map(|s| s.as_str()).unwrap_or("INFO");
        let message = ctx.fields.get("message").map(|s| s.as_str()).unwrap_or("event generated");
        let logger = ctx.fields.get("java_class").map(|s| s.as_str()).unwrap_or("app.main");
        let thread = ctx.fields.get("thread_name").map(|s| s.as_str()).unwrap_or("main");
        let hostname = ctx.fields.get("hostname").map(|s| s.as_str()).unwrap_or("localhost");
        let src_ip = ctx.fields.get("src_ip").map(|s| s.as_str()).unwrap_or("127.0.0.1");

        let mut obj = serde_json::Map::new();
        obj.insert("@timestamp".into(), serde_json::Value::String(
            ctx.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
        ));
        obj.insert("level".into(), serde_json::Value::String(level.into()));
        obj.insert("logger".into(), serde_json::Value::String(logger.into()));
        obj.insert("thread".into(), serde_json::Value::String(thread.into()));
        obj.insert("message".into(), serde_json::Value::String(message.into()));
        obj.insert("hostname".into(), serde_json::Value::String(hostname.into()));
        obj.insert("src_ip".into(), serde_json::Value::String(src_ip.into()));
        obj.insert("sequence".into(), serde_json::Value::Number(ctx.sequence.into()));

        for (k, v) in &self.extra_fields {
            obj.insert(k.clone(), serde_json::Value::String(v.clone()));
        }

        serde_json::Value::Object(obj).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::FormatConfig;
    use crate::format::EventContext;

    fn test_config() -> FormatConfig {
        let mut extra = HashMap::new();
        extra.insert("environment".into(), "production".into());
        FormatConfig {
            format_type: "json".into(),
            facility: None, severity: None, app_name: None,
            vendor: None, product: None, version: None,
            device_event_class_id: None,
            extra_fields: Some(extra),
            template_file: None, template_inline: None,
            script_file: None, script_inline: None, max_operations: None,
        }
    }

    #[test]
    fn formats_valid_json() {
        let formatter = JsonStructuredFormatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 5);
        ctx.fields.insert("log_level".into(), "WARN".into());
        ctx.fields.insert("message".into(), "test message".into());
        let output = formatter.format(&ctx);

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("should be valid JSON");
        assert_eq!(parsed["level"], "WARN");
        assert_eq!(parsed["message"], "test message");
        assert_eq!(parsed["sequence"], 5);
        assert_eq!(parsed["environment"], "production");
    }
}
