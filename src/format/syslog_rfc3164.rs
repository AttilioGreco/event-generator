use super::EventContext;
use crate::config::model::FormatConfig;

pub struct SyslogRfc3164Formatter {
    facility: String,
    severity: String,
    app_name: String,
}

impl SyslogRfc3164Formatter {
    pub fn new(config: &FormatConfig) -> Self {
        Self {
            facility: config.facility.clone().unwrap_or_else(|| "local0".into()),
            severity: config.severity.clone().unwrap_or_else(|| "info".into()),
            app_name: config.app_name.clone().unwrap_or_else(|| "event-generator".into()),
        }
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let pri = compute_priority(&self.facility, &self.severity);
        // RFC 3164: Mmm dd HH:MM:SS (note: day is space-padded, not zero-padded)
        let timestamp = ctx.timestamp.format("%b %e %H:%M:%S");
        let hostname = ctx.fields.get("hostname").map(|s| s.as_str()).unwrap_or("localhost");
        let pid = ctx.fields.get("pid").map(|s| s.as_str()).unwrap_or("1000");
        let message = ctx.fields.get("message").map(|s| s.as_str()).unwrap_or("event generated");

        // RFC 3164: <PRI>TIMESTAMP HOSTNAME APP[PID]: MSG
        format!("<{pri}>{timestamp} {hostname} {app}[{pid}]: {message}", app = self.app_name)
    }
}

fn compute_priority(facility: &str, severity: &str) -> u8 {
    let facility_code: u8 = match facility {
        "kern" => 0, "user" => 1, "mail" => 2, "daemon" => 3,
        "auth" => 4, "syslog" => 5, "lpr" => 6, "news" => 7,
        "uucp" => 8, "cron" => 9, "authpriv" => 10, "ftp" => 11,
        "local0" => 16, "local1" => 17, "local2" => 18, "local3" => 19,
        "local4" => 20, "local5" => 21, "local6" => 22, "local7" => 23,
        _ => 16,
    };

    let severity_code: u8 = match severity {
        "emerg" => 0, "alert" => 1, "crit" => 2, "err" | "error" => 3,
        "warning" | "warn" => 4, "notice" => 5, "info" => 6, "debug" => 7,
        _ => 6,
    };

    facility_code * 8 + severity_code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::FormatConfig;
    use crate::format::EventContext;

    fn test_config() -> FormatConfig {
        FormatConfig {
            format_type: "syslog_rfc3164".into(),
            facility: Some("daemon".into()),
            severity: Some("err".into()),
            app_name: Some("myapp".into()),
            vendor: None, product: None, version: None,
            device_event_class_id: None, extra_fields: None,
            template_file: None, template_inline: None,
            script_file: None, script_inline: None, max_operations: None,
        }
    }

    #[test]
    fn formats_valid_rfc3164() {
        let formatter = SyslogRfc3164Formatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 1);
        ctx.fields.insert("hostname".into(), "web-01".into());
        ctx.fields.insert("pid".into(), "12345".into());
        ctx.fields.insert("message".into(), "Something happened".into());
        let output = formatter.format(&ctx);

        // PRI for daemon.err = 3*8 + 3 = 27
        assert!(output.starts_with("<27>"), "expected PRI 27, got: {output}");
        assert!(output.contains("web-01"));
        assert!(output.contains("myapp[12345]:"));
        assert!(output.contains("Something happened"));
    }
}
