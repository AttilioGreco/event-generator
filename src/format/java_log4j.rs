use super::EventContext;
use crate::config::model::FormatConfig;

pub struct JavaLog4jFormatter;

impl JavaLog4jFormatter {
    pub fn new(_config: &FormatConfig) -> Self {
        Self
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let level = ctx.fields.get("log_level").map(|s| s.as_str()).unwrap_or("INFO");
        let thread = ctx.fields.get("thread_name").map(|s| s.as_str()).unwrap_or("main");
        let class = ctx.fields.get("java_class").map(|s| s.as_str()).unwrap_or("com.app.Main");
        let message = ctx.fields.get("message").map(|s| s.as_str()).unwrap_or("event generated");

        // log4j default pattern: %d [%t] %-5p %c - %m%n
        // Example: 2026-02-20 14:23:01,456 [http-nio-8080-exec-3] INFO  com.app.service.UserService - Message
        let timestamp = ctx.timestamp.format("%Y-%m-%d %H:%M:%S,%3f");

        format!("{timestamp} [{thread}] {level:<5} {class} - {message}")
    }
}

pub struct JavaLogbackFormatter;

impl JavaLogbackFormatter {
    pub fn new(_config: &FormatConfig) -> Self {
        Self
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let level = ctx.fields.get("log_level").map(|s| s.as_str()).unwrap_or("INFO");
        let thread = ctx.fields.get("thread_name").map(|s| s.as_str()).unwrap_or("main");
        let class = ctx.fields.get("java_class").map(|s| s.as_str()).unwrap_or("com.app.Main");
        let message = ctx.fields.get("message").map(|s| s.as_str()).unwrap_or("event generated");

        // Logback default pattern: %d{HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n
        let timestamp = ctx.timestamp.format("%H:%M:%S%.3f");

        format!("{timestamp} [{thread}] {level:<5} {class} - {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::FormatConfig;
    use crate::format::EventContext;

    fn test_config() -> FormatConfig {
        FormatConfig {
            format_type: "java_log4j".into(),
            facility: None, severity: None,
            app_name: Some("myapp".into()),
            vendor: None, product: None, version: None,
            device_event_class_id: None, extra_fields: None,
            template_file: None, template_inline: None,
        }
    }

    #[test]
    fn formats_valid_log4j() {
        let formatter = JavaLog4jFormatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 1);
        ctx.fields.insert("log_level".into(), "ERROR".into());
        ctx.fields.insert("thread_name".into(), "http-nio-8080-exec-3".into());
        ctx.fields.insert("java_class".into(), "com.app.service.UserService".into());
        ctx.fields.insert("message".into(), "NullPointerException".into());
        let output = formatter.format(&ctx);

        assert!(output.contains("[http-nio-8080-exec-3]"), "missing thread: {output}");
        assert!(output.contains("ERROR"), "missing level: {output}");
        assert!(output.contains("com.app.service.UserService"), "missing class: {output}");
        assert!(output.contains("NullPointerException"), "missing message: {output}");
    }

    #[test]
    fn formats_valid_logback() {
        let formatter = JavaLogbackFormatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 1);
        ctx.fields.insert("log_level".into(), "INFO".into());
        ctx.fields.insert("message".into(), "Started".into());
        let output = formatter.format(&ctx);

        // Logback uses HH:MM:SS.mmm format (no date prefix like 2026-02-20)
        // The first char should be a digit (hour), not a date
        assert!(output.chars().next().unwrap().is_ascii_digit(), "should start with time: {output}");
        assert!(!output.starts_with("20"), "should not start with year: {output}");
        assert!(output.contains("Started"));
    }
}
