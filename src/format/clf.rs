use super::EventContext;
use crate::config::model::FormatConfig;

pub struct ClfFormatter;

impl ClfFormatter {
    pub fn new(_config: &FormatConfig) -> Self {
        Self
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let ip = ctx
            .fields
            .get("src_ip")
            .map(|s| s.as_str())
            .unwrap_or("127.0.0.1");
        let user = ctx
            .fields
            .get("username")
            .map(|s| s.as_str())
            .unwrap_or("-");
        let method = ctx
            .fields
            .get("http_method")
            .map(|s| s.as_str())
            .unwrap_or("GET");
        let path = ctx
            .fields
            .get("http_path")
            .map(|s| s.as_str())
            .unwrap_or("/");
        let status = ctx
            .fields
            .get("http_status")
            .map(|s| s.as_str())
            .unwrap_or("200");
        let bytes = ctx
            .fields
            .get("http_bytes")
            .map(|s| s.as_str())
            .unwrap_or("0");

        // CLF/NCSA Combined Log Format:
        // host ident authuser [date] "request" status bytes "referer" "user-agent"
        let timestamp = ctx.timestamp.format("%d/%b/%Y:%H:%M:%S %z");
        let user_agent = ctx
            .fields
            .get("user_agent")
            .map(|s| s.as_str())
            .unwrap_or("-");

        format!(
            "{ip} - {user} [{timestamp}] \"{method} {path} HTTP/1.1\" {status} {bytes} \"-\" \"{user_agent}\""
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
            format_type: "clf".into(),
            facility: None,
            severity: None,
            app_name: None,
            vendor: None,
            product: None,
            version: None,
            device_event_class_id: None,
            extra_fields: None,
            template_file: None,
            template_inline: None,
            script_file: None,
            script_inline: None,
            max_operations: None,
        }
    }

    #[test]
    fn formats_valid_clf() {
        let formatter = ClfFormatter::new(&test_config());
        let mut ctx = EventContext::new("test".into(), 1);
        ctx.fields.insert("src_ip".into(), "192.168.1.55".into());
        ctx.fields.insert("username".into(), "jsmith".into());
        ctx.fields.insert("http_method".into(), "GET".into());
        ctx.fields.insert("http_path".into(), "/index.html".into());
        ctx.fields.insert("http_status".into(), "200".into());
        ctx.fields.insert("http_bytes".into(), "2326".into());
        let output = formatter.format(&ctx);

        assert!(
            output.starts_with("192.168.1.55 - jsmith ["),
            "bad CLF start: {output}"
        );
        assert!(output.contains("\"GET /index.html HTTP/1.1\" 200 2326"));
    }
}
