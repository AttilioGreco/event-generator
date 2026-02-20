use std::collections::HashMap;
use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use fake::faker::internet::en::{IPv4, IPv6, UserAgent, Username};
use fake::Fake;
use rand::Rng;
use tera::Tera;

use super::EventContext;

pub struct TemplateFormatter {
    tera: Tera,
    template_name: String,
}

impl TemplateFormatter {
    pub fn from_inline(template: &str) -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("inline", template)
            .with_context(|| "failed to parse inline template")?;
        register_functions(&mut tera);
        Ok(Self {
            tera,
            template_name: "inline".into(),
        })
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read template file: {path}"))?;
        let mut tera = Tera::default();
        tera.add_raw_template(path, &content)
            .with_context(|| format!("failed to parse template file: {path}"))?;
        register_functions(&mut tera);
        Ok(Self {
            tera,
            template_name: path.into(),
        })
    }

    pub fn format(&self, ctx: &EventContext) -> String {
        let mut tera_ctx = tera::Context::new();

        // Insert all EventContext fields as template variables
        for (k, v) in &ctx.fields {
            tera_ctx.insert(k, v);
        }
        tera_ctx.insert("sequence", &ctx.sequence);
        tera_ctx.insert("stream_name", &ctx.stream_name);
        tera_ctx.insert("timestamp", &ctx.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());

        match self.tera.render(&self.template_name, &tera_ctx) {
            Ok(rendered) => rendered.trim_end_matches('\n').to_string(),
            Err(e) => {
                eprintln!("[template] render error: {e}");
                format!("TEMPLATE_ERROR: {e}")
            }
        }
    }
}

fn register_functions(tera: &mut Tera) {
    tera.register_function("fake_ipv4", make_fake_ipv4());
    tera.register_function("fake_ipv6", make_fake_ipv6());
    tera.register_function("fake_hostname", make_fake_hostname());
    tera.register_function("fake_username", make_fake_username());
    tera.register_function("fake_user_agent", make_fake_user_agent());
    tera.register_function("fake_http_method", make_fake_http_method());
    tera.register_function("fake_http_path", make_fake_http_path());
    tera.register_function("fake_http_status", make_fake_http_status());
    tera.register_function("fake_uuid", make_fake_uuid());
    tera.register_function("fake_int", make_fake_int());
    tera.register_function("timestamp_iso", make_timestamp_iso());
    tera.register_function("timestamp_epoch", make_timestamp_epoch());
    tera.register_function("timestamp_rfc3339", make_timestamp_rfc3339());
    tera.register_function("pick", make_pick());
}

fn make_fake_ipv4() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let ip: Ipv4Addr = IPv4().fake();
        Ok(tera::Value::String(ip.to_string()))
    }
}

fn make_fake_ipv6() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let ip: String = IPv6().fake();
        Ok(tera::Value::String(ip))
    }
}

fn make_fake_hostname() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let mut rng = rand::rng();
        let prefixes = ["web", "app", "db", "srv", "gw", "fw", "proxy", "cache", "auth", "api"];
        let idx = rng.random_range(0..prefixes.len());
        let num = rng.random_range(1u16..99);
        Ok(tera::Value::String(format!("{}-{num:02}", prefixes[idx])))
    }
}

fn make_fake_username() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let name: String = Username().fake();
        Ok(tera::Value::String(name))
    }
}

fn make_fake_user_agent() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let ua: String = UserAgent().fake();
        Ok(tera::Value::String(ua))
    }
}

fn make_fake_http_method() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let mut rng = rand::rng();
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
        let idx = rng.random_range(0..methods.len());
        Ok(tera::Value::String(methods[idx].into()))
    }
}

fn make_fake_http_path() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let mut rng = rand::rng();
        let paths = ["/", "/api/v1/users", "/api/v1/orders", "/login", "/health", "/metrics"];
        let idx = rng.random_range(0..paths.len());
        Ok(tera::Value::String(paths[idx].into()))
    }
}

fn make_fake_http_status() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let mut rng = rand::rng();
        let statuses = [200, 200, 200, 201, 301, 400, 401, 403, 404, 500, 502];
        let idx = rng.random_range(0..statuses.len());
        Ok(tera::Value::Number(statuses[idx].into()))
    }
}

fn make_fake_uuid() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let mut rng = rand::rng();
        let bytes: [u8; 16] = rng.random();
        let uuid = format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF,
            (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3FFF) | 0x8000,
            u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]),
        );
        Ok(tera::Value::String(uuid))
    }
}

fn make_fake_int() -> impl tera::Function {
    |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let min = args.get("min")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let max = args.get("max")
            .and_then(|v| v.as_i64())
            .unwrap_or(65535);
        let mut rng = rand::rng();
        let val = rng.random_range(min..=max);
        Ok(tera::Value::Number(val.into()))
    }
}

fn make_timestamp_iso() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        Ok(tera::Value::String(
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
        ))
    }
}

fn make_timestamp_epoch() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        Ok(tera::Value::Number(chrono::Utc::now().timestamp().into()))
    }
}

fn make_timestamp_rfc3339() -> impl tera::Function {
    |_args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        Ok(tera::Value::String(
            chrono::Utc::now().to_rfc3339()
        ))
    }
}

fn make_pick() -> impl tera::Function {
    |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let values_str = args.get("values")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("pick() requires a 'values' argument (comma-separated string)"))?;
        let options: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();
        if options.is_empty() {
            return Err(tera::Error::msg("pick() values list is empty"));
        }
        let mut rng = rand::rng();
        let idx = rng.random_range(0..options.len());
        Ok(tera::Value::String(options[idx].into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_template_renders() {
        let formatter = TemplateFormatter::from_inline(
            "{{ timestamp }} {{ src_ip }} {{ message }}"
        ).unwrap();
        let ctx = EventContext::new("test".into(), 0);
        let output = formatter.format(&ctx);

        assert!(!output.contains("{{"), "unresolved placeholder: {output}");
        assert!(!output.is_empty());
    }

    #[test]
    fn custom_functions_work() {
        let formatter = TemplateFormatter::from_inline(
            "{{ fake_ipv4() }} {{ fake_hostname() }} {{ fake_int(min=1, max=100) }} {{ pick(values=\"a,b,c\") }}"
        ).unwrap();
        let ctx = EventContext::new("test".into(), 0);
        let output = formatter.format(&ctx);

        assert!(!output.contains("{{"), "unresolved placeholder: {output}");
        let parts: Vec<&str> = output.split_whitespace().collect();
        assert_eq!(parts.len(), 4, "expected 4 parts: {output}");

        // First part should be an IP
        assert!(parts[0].parse::<Ipv4Addr>().is_ok(), "not an IP: {}", parts[0]);
    }

    #[test]
    fn timestamp_functions_work() {
        let formatter = TemplateFormatter::from_inline(
            "{{ timestamp_iso() }}|{{ timestamp_epoch() }}|{{ timestamp_rfc3339() }}"
        ).unwrap();
        let ctx = EventContext::new("test".into(), 0);
        let output = formatter.format(&ctx);

        let parts: Vec<&str> = output.split('|').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[0].contains('T'), "ISO should contain T: {}", parts[0]);
        assert!(parts[1].parse::<i64>().is_ok(), "epoch should be numeric: {}", parts[1]);
    }

    #[test]
    fn uuid_function_works() {
        let formatter = TemplateFormatter::from_inline("{{ fake_uuid() }}").unwrap();
        let ctx = EventContext::new("test".into(), 0);
        let output = formatter.format(&ctx);

        assert_eq!(output.len(), 36, "UUID should be 36 chars: {output}");
        assert_eq!(output.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn invalid_template_fails_at_creation() {
        let result = TemplateFormatter::from_inline("{{ unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn sequence_and_stream_name_available() {
        let formatter = TemplateFormatter::from_inline(
            "seq={{ sequence }} stream={{ stream_name }}"
        ).unwrap();
        let ctx = EventContext::new("my-stream".into(), 42);
        let output = formatter.format(&ctx);

        assert!(output.contains("seq=42"), "missing sequence: {output}");
        assert!(output.contains("stream=my-stream"), "missing stream_name: {output}");
    }
}
