use std::collections::HashMap;
use std::net::Ipv4Addr;

use fake::faker::internet::en::{IPv4, IPv6, UserAgent, Username};
use fake::faker::name::en::Name;
use fake::Fake;
use rand::Rng;

pub struct FakeDataProvider;

impl FakeDataProvider {
    pub fn populate(fields: &mut HashMap<String, String>) {
        let mut rng = rand::rng();

        // fake-rs generated fields (use .fake() which uses its own internal rng)
        let ip: Ipv4Addr = IPv4().fake();
        fields.insert("src_ip".into(), ip.to_string());
        let ip: Ipv4Addr = IPv4().fake();
        fields.insert("dst_ip".into(), ip.to_string());
        let username: String = Username().fake();
        fields.insert("username".into(), username);
        let fullname: String = Name().fake();
        fields.insert("user_fullname".into(), fullname);
        let ua: String = UserAgent().fake();
        fields.insert("user_agent".into(), ua);
        let ipv6: String = IPv6().fake();
        fields.insert("ipv6".into(), ipv6);

        // Custom generated fields (use our rand 0.9 rng)
        fields.insert("hostname".into(), Self::hostname(&mut rng));
        fields.insert("http_method".into(), Self::http_method(&mut rng));
        fields.insert("http_path".into(), Self::http_path(&mut rng));
        fields.insert("http_status".into(), Self::http_status(&mut rng).to_string());
        fields.insert("http_bytes".into(), rng.random_range(64u32..65535).to_string());
        fields.insert("log_level".into(), Self::log_level(&mut rng));
        fields.insert("src_port".into(), rng.random_range(1024u16..65535).to_string());
        fields.insert("dst_port".into(), Self::well_known_port(&mut rng).to_string());
        fields.insert("pid".into(), rng.random_range(1000u32..65535).to_string());
        fields.insert("thread_name".into(), Self::thread_name(&mut rng));
        fields.insert("java_class".into(), Self::java_class(&mut rng));
        fields.insert("message".into(), Self::log_message(&mut rng));
        fields.insert("protocol".into(), Self::protocol(&mut rng));
        fields.insert("action".into(), Self::action(&mut rng));
        fields.insert("severity_name".into(), Self::severity_name(&mut rng));
    }

    pub fn ipv4() -> String {
        let ip: Ipv4Addr = IPv4().fake();
        ip.to_string()
    }

    fn hostname(rng: &mut impl Rng) -> String {
        let prefixes = ["web", "app", "db", "srv", "gw", "fw", "proxy", "cache", "auth", "api"];
        let idx = rng.random_range(0..prefixes.len());
        let num = rng.random_range(1u16..99);
        format!("{}-{num:02}", prefixes[idx])
    }

    fn http_method(rng: &mut impl Rng) -> String {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        let weights = [50, 25, 10, 5, 5, 3, 2];
        let total: u32 = weights.iter().sum();
        let mut pick = rng.random_range(0..total);
        for (i, &w) in weights.iter().enumerate() {
            if pick < w {
                return methods[i].into();
            }
            pick -= w;
        }
        "GET".into()
    }

    fn http_path(rng: &mut impl Rng) -> String {
        let paths = [
            "/", "/index.html", "/api/v1/users", "/api/v1/orders",
            "/api/v2/products", "/login", "/logout", "/health",
            "/metrics", "/api/v1/search", "/static/css/main.css",
            "/static/js/app.js", "/favicon.ico", "/api/v1/auth/token",
            "/dashboard", "/admin/settings", "/api/v1/events",
            "/api/v1/notifications", "/profile", "/register",
        ];
        let idx = rng.random_range(0..paths.len());
        paths[idx].into()
    }

    fn http_status(rng: &mut impl Rng) -> u16 {
        let statuses = [200, 200, 200, 200, 201, 204, 301, 302, 304, 400, 401, 403, 404, 404, 500, 502, 503];
        let idx = rng.random_range(0..statuses.len());
        statuses[idx]
    }

    fn log_level(rng: &mut impl Rng) -> String {
        let levels = ["TRACE", "DEBUG", "INFO", "INFO", "INFO", "WARN", "ERROR", "FATAL"];
        let idx = rng.random_range(0..levels.len());
        levels[idx].into()
    }

    fn well_known_port(rng: &mut impl Rng) -> u16 {
        let ports = [22, 53, 80, 443, 514, 1433, 3306, 3389, 5432, 6379, 8080, 8443, 9200, 27017];
        let idx = rng.random_range(0..ports.len());
        ports[idx]
    }

    fn thread_name(rng: &mut impl Rng) -> String {
        let names = [
            "http-nio-8080-exec", "pool-1-thread", "main", "worker",
            "scheduler", "async-dispatch", "kafka-consumer", "grpc-server",
        ];
        let idx = rng.random_range(0..names.len());
        let num = rng.random_range(1u16..20);
        format!("{}-{num}", names[idx])
    }

    fn java_class(rng: &mut impl Rng) -> String {
        let classes = [
            "com.app.service.UserService",
            "com.app.service.OrderService",
            "com.app.controller.AuthController",
            "com.app.repository.UserRepository",
            "com.app.security.JwtFilter",
            "com.app.config.DataSourceConfig",
            "com.app.handler.ExceptionHandler",
            "com.app.service.NotificationService",
            "com.app.gateway.PaymentGateway",
            "com.app.scheduler.CleanupTask",
        ];
        let idx = rng.random_range(0..classes.len());
        classes[idx].into()
    }

    fn log_message(rng: &mut impl Rng) -> String {
        let messages = [
            "Connection established successfully",
            "Request processed in 23ms",
            "User authentication completed",
            "Cache miss for key user:session",
            "Health check passed",
            "Scheduled task executed",
            "Configuration reloaded",
            "New connection from client",
            "Database query completed in 45ms",
            "Service started successfully",
            "Incoming request received",
            "Response sent to client",
            "Worker thread spawned",
            "Memory usage within threshold",
            "Periodic cleanup completed",
            "Session expired for user",
            "Rate limit exceeded for client",
            "SSL handshake completed",
            "DNS resolution completed",
            "Retry attempt 2 of 3",
            "Transaction committed successfully",
            "File upload completed",
            "Notification sent to queue",
            "Circuit breaker state: CLOSED",
            "Metrics exported successfully",
        ];
        let idx = rng.random_range(0..messages.len());
        messages[idx].into()
    }

    fn protocol(rng: &mut impl Rng) -> String {
        let protocols = ["TCP", "UDP", "ICMP", "HTTP", "HTTPS", "DNS", "SSH", "TLS"];
        let idx = rng.random_range(0..protocols.len());
        protocols[idx].into()
    }

    fn action(rng: &mut impl Rng) -> String {
        let actions = ["allow", "deny", "drop", "reject", "accept", "block", "redirect"];
        let idx = rng.random_range(0..actions.len());
        actions[idx].into()
    }

    fn severity_name(rng: &mut impl Rng) -> String {
        let names = ["Low", "Medium", "High", "Critical", "Informational"];
        let idx = rng.random_range(0..names.len());
        names[idx].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_fills_all_fields() {
        let mut fields = HashMap::new();
        FakeDataProvider::populate(&mut fields);

        let expected_keys = [
            "src_ip", "dst_ip", "hostname", "username", "http_method",
            "http_path", "http_status", "user_agent", "log_level",
            "src_port", "dst_port", "pid", "thread_name", "java_class",
            "message", "protocol", "action", "http_bytes",
        ];

        for key in expected_keys {
            assert!(fields.contains_key(key), "missing field: {key}");
            assert!(!fields[key].is_empty(), "empty field: {key}");
        }
    }

    #[test]
    fn ipv4_is_valid_format() {
        for _ in 0..100 {
            let ip = FakeDataProvider::ipv4();
            assert!(ip.parse::<Ipv4Addr>().is_ok(), "invalid IPv4: {ip}");
        }
    }
}
