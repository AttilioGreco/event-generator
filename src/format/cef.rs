use rand::Rng;

use super::EventContext;
use crate::config::model::FormatConfig;

pub struct CefFormatter {
    vendor: String,
    product: String,
    version: String,
    device_event_class_id: String,
}

impl CefFormatter {
    pub fn new(config: &FormatConfig) -> Self {
        Self {
            vendor: config.vendor.clone().unwrap_or_else(|| "SecurityVendor".into()),
            product: config.product.clone().unwrap_or_else(|| "Firewall".into()),
            version: config.version.clone().unwrap_or_else(|| "1.0".into()),
            device_event_class_id: config.device_event_class_id.clone().unwrap_or_else(|| "100".into()),
        }
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let mut rng = rand::rng();

        let event_names = [
            "Connection Allowed", "Connection Denied", "Connection Dropped",
            "Intrusion Detected", "Policy Violation", "Authentication Success",
            "Authentication Failure", "Port Scan Detected", "Malware Detected",
            "Traffic Anomaly",
        ];
        let name = event_names[rng.random_range(0..event_names.len())];
        let severity = rng.random_range(1u8..10);

        let src = ctx.fields.get("src_ip").map(|s| s.as_str()).unwrap_or("10.0.0.1");
        let dst = ctx.fields.get("dst_ip").map(|s| s.as_str()).unwrap_or("192.168.1.1");
        let dpt = ctx.fields.get("dst_port").map(|s| s.as_str()).unwrap_or("443");
        let spt = ctx.fields.get("src_port").map(|s| s.as_str()).unwrap_or("54321");
        let proto = ctx.fields.get("protocol").map(|s| s.as_str()).unwrap_or("TCP");
        let action = ctx.fields.get("action").map(|s| s.as_str()).unwrap_or("allow");
        let hostname = ctx.fields.get("hostname").map(|s| s.as_str()).unwrap_or("fw-01");

        // CEF:Version|Device Vendor|Device Product|Device Version|Device Event Class ID|Name|Severity|[Extension]
        format!(
            "CEF:0|{vendor}|{product}|{version}|{class_id}|{name}|{severity}|\
             src={src} dst={dst} spt={spt} dpt={dpt} proto={proto} act={action} \
             dhost={hostname} rt={timestamp} msg=Event sequence {seq}",
            vendor = self.vendor,
            product = self.product,
            version = self.version,
            class_id = self.device_event_class_id,
            timestamp = ctx.timestamp.format("%b %d %Y %H:%M:%S"),
            seq = ctx.sequence,
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
            format_type: "cef".into(),
            facility: None, severity: None, app_name: None,
            vendor: Some("TestVendor".into()),
            product: Some("TestFW".into()),
            version: Some("2.0".into()),
            device_event_class_id: Some("200".into()),
            extra_fields: None, template_file: None, template_inline: None,
        }
    }

    #[test]
    fn formats_valid_cef() {
        let formatter = CefFormatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 42);
        ctx.fields.insert("src_ip".into(), "10.0.0.5".into());
        ctx.fields.insert("dst_ip".into(), "192.168.1.10".into());
        let output = formatter.format(&ctx);

        assert!(output.starts_with("CEF:0|TestVendor|TestFW|2.0|200|"), "bad CEF header: {output}");
        assert!(output.contains("src=10.0.0.5"));
        assert!(output.contains("dst=192.168.1.10"));
    }
}
