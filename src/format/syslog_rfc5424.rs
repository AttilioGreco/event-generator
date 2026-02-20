use rand::Rng;

use super::EventContext;
use crate::config::model::FormatConfig;

pub struct SyslogRfc5424Formatter {
    facility: String,
    severity: String,
    app_name: String,
}

impl SyslogRfc5424Formatter {
    pub fn new(config: &FormatConfig) -> Self {
        Self {
            facility: config
                .facility
                .clone()
                .unwrap_or_else(|| "local0".into()),
            severity: config.severity.clone().unwrap_or_else(|| "info".into()),
            app_name: config
                .app_name
                .clone()
                .unwrap_or_else(|| "event-generator".into()),
        }
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let pri = compute_priority(&self.facility, &self.severity);
        let timestamp = ctx.timestamp.format("%Y-%m-%dT%H:%M:%S%.6f%:z");
        let hostname = ctx
            .fields
            .get("hostname")
            .cloned()
            .unwrap_or_else(|| "localhost".into());
        let pid = ctx
            .fields
            .get("pid")
            .cloned()
            .unwrap_or_else(|| random_pid().to_string());
        let msg_id = format!("ID{}", ctx.sequence);
        let message = ctx
            .fields
            .get("message")
            .cloned()
            .unwrap_or_else(|| random_message());

        // RFC 5424: <PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID STRUCTURED-DATA MSG
        format!(
            "<{pri}>1 {timestamp} {hostname} {app_name} {pid} {msg_id} - {message}",
            app_name = self.app_name,
        )
    }
}

fn compute_priority(facility: &str, severity: &str) -> u8 {
    let facility_code: u8 = match facility {
        "kern" => 0,
        "user" => 1,
        "mail" => 2,
        "daemon" => 3,
        "auth" => 4,
        "syslog" => 5,
        "lpr" => 6,
        "news" => 7,
        "uucp" => 8,
        "cron" => 9,
        "authpriv" => 10,
        "ftp" => 11,
        "local0" => 16,
        "local1" => 17,
        "local2" => 18,
        "local3" => 19,
        "local4" => 20,
        "local5" => 21,
        "local6" => 22,
        "local7" => 23,
        _ => 16, // default local0
    };

    let severity_code: u8 = match severity {
        "emerg" => 0,
        "alert" => 1,
        "crit" => 2,
        "err" | "error" => 3,
        "warning" | "warn" => 4,
        "notice" => 5,
        "info" => 6,
        "debug" => 7,
        _ => 6, // default info
    };

    facility_code * 8 + severity_code
}

fn random_pid() -> u32 {
    rand::rng().random_range(1000..65535)
}

fn random_message() -> String {
    let messages = [
        "Connection established successfully",
        "Request processed in 23ms",
        "User authentication completed",
        "Cache miss for key user:session",
        "Health check passed",
        "Scheduled task executed",
        "Configuration reloaded",
        "New connection from client",
        "Database query completed",
        "Service started successfully",
        "Incoming request received",
        "Response sent to client",
        "Worker thread spawned",
        "Memory usage within threshold",
        "Periodic cleanup completed",
    ];
    let idx = rand::rng().random_range(0..messages.len());
    messages[idx].into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::FormatConfig;
    use crate::format::EventContext;

    fn test_config() -> FormatConfig {
        FormatConfig {
            format_type: "syslog_rfc5424".into(),
            facility: Some("local0".into()),
            severity: Some("info".into()),
            app_name: Some("test-app".into()),
            vendor: None,
            product: None,
            version: None,
            device_event_class_id: None,
            extra_fields: None,
            template_file: None,
            template_inline: None,
        }
    }

    #[test]
    fn formats_valid_rfc5424() {
        let formatter = SyslogRfc5424Formatter::new(&test_config());
        let ctx = EventContext::new("test-stream".into(), 42);
        let output = formatter.format(&ctx);

        // RFC 5424: <PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID SD MSG
        assert!(output.starts_with("<134>1 "), "expected PRI 134 (local0.info), got: {output}");
        assert!(output.contains("test-app"), "expected app name in output");
        assert!(output.contains("ID42"), "expected msg ID with sequence");
    }

    #[test]
    fn priority_calculation() {
        assert_eq!(compute_priority("kern", "emerg"), 0);
        assert_eq!(compute_priority("kern", "info"), 6);
        assert_eq!(compute_priority("local0", "info"), 134);
        assert_eq!(compute_priority("local7", "debug"), 191);
        assert_eq!(compute_priority("auth", "err"), 35);
    }
}
