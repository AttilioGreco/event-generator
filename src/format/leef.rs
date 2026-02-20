use rand::Rng;

use super::EventContext;
use crate::config::model::FormatConfig;

pub struct LeefFormatter {
    vendor: String,
    product: String,
    version: String,
}

impl LeefFormatter {
    pub fn new(config: &FormatConfig) -> Self {
        Self {
            vendor: config.vendor.clone().unwrap_or_else(|| "SecurityVendor".into()),
            product: config.product.clone().unwrap_or_else(|| "IDS".into()),
            version: config.version.clone().unwrap_or_else(|| "1.0".into()),
        }
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let mut rng = rand::rng();

        let event_ids = [
            "LOGIN_SUCCESS", "LOGIN_FAILURE", "CONN_ALLOWED", "CONN_DENIED",
            "POLICY_MATCH", "SCAN_DETECTED", "THREAT_FOUND", "SESSION_START",
            "SESSION_END", "CONFIG_CHANGE",
        ];
        let event_id = event_ids[rng.random_range(0..event_ids.len())];

        let src = ctx.fields.get("src_ip").map(|s| s.as_str()).unwrap_or("10.0.0.1");
        let dst = ctx.fields.get("dst_ip").map(|s| s.as_str()).unwrap_or("192.168.1.1");
        let src_port = ctx.fields.get("src_port").map(|s| s.as_str()).unwrap_or("54321");
        let dst_port = ctx.fields.get("dst_port").map(|s| s.as_str()).unwrap_or("443");
        let proto = ctx.fields.get("protocol").map(|s| s.as_str()).unwrap_or("TCP");
        let username = ctx.fields.get("username").map(|s| s.as_str()).unwrap_or("unknown");
        let sev = ctx.fields.get("severity_name").map(|s| s.as_str()).unwrap_or("Medium");

        // LEEF:Version|Vendor|Product|Version|EventID|
        // Key=Value pairs separated by tab
        format!(
            "LEEF:2.0|{vendor}|{product}|{version}|{event_id}|\t\
             src={src}\tdst={dst}\tsrcPort={src_port}\tdstPort={dst_port}\t\
             proto={proto}\tusrName={username}\tsev={sev}\t\
             devTime={timestamp}\tdevTimeFormat=yyyy-MM-dd'T'HH:mm:ss.SSSZ",
            vendor = self.vendor,
            product = self.product,
            version = self.version,
            timestamp = ctx.timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%z"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::FormatConfig;
    use crate::format::EventContext;

    fn test_config() -> FormatConfig {
        FormatConfig {
            format_type: "leef".into(),
            facility: None, severity: None, app_name: None,
            vendor: Some("TestCo".into()),
            product: Some("TestIDS".into()),
            version: Some("3.0".into()),
            device_event_class_id: None, extra_fields: None,
            template_file: None, template_inline: None,
        }
    }

    #[test]
    fn formats_valid_leef() {
        let formatter = LeefFormatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 1);
        ctx.fields.insert("src_ip".into(), "10.0.0.5".into());
        ctx.fields.insert("username".into(), "admin".into());
        let output = formatter.format(&ctx);

        assert!(output.starts_with("LEEF:2.0|TestCo|TestIDS|3.0|"), "bad LEEF header: {output}");
        assert!(output.contains("src=10.0.0.5"));
        assert!(output.contains("usrName=admin"));
    }
}
