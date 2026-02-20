use std::net::Ipv4Addr;

use fake::faker::internet::en::{IPv4, IPv6, UserAgent, Username};
use fake::Fake;
use rand::Rng;
use rhai::{Array, Engine};

/// Register all custom functions available to Rhai scripts.
pub fn register_all(engine: &mut Engine) {
    register_emit(engine);
    register_fake_data(engine);
    register_timestamps(engine);
    register_utilities(engine);
}

// ---------------------------------------------------------------------------
// emit() — pushes lines to the output buffer
// ---------------------------------------------------------------------------

fn register_emit(engine: &mut Engine) {
    // emit(line) is registered per-call with access to the output buffer,
    // so it's handled in ScriptEngine::run() via Engine::call_fn / Scope.
    // Here we register a placeholder that will be overridden.
    // Actually, emit is registered dynamically in mod.rs with closure over Rc<RefCell<Vec<String>>>.
    let _ = engine;
}

// ---------------------------------------------------------------------------
// Fake data generators
// ---------------------------------------------------------------------------

fn register_fake_data(engine: &mut Engine) {
    engine.register_fn("fake_ipv4", || -> String {
        let ip: Ipv4Addr = IPv4().fake();
        ip.to_string()
    });

    engine.register_fn("fake_ipv6", || -> String {
        let ip: String = IPv6().fake();
        ip
    });

    engine.register_fn("fake_hostname", || -> String {
        let mut rng = rand::rng();
        let prefixes = [
            "web", "app", "db", "srv", "gw", "fw", "proxy", "cache", "auth", "api",
        ];
        let idx = rng.random_range(0..prefixes.len());
        let num = rng.random_range(1u16..99);
        format!("{}-{num:02}", prefixes[idx])
    });

    engine.register_fn("fake_username", || -> String {
        let name: String = Username().fake();
        name
    });

    engine.register_fn("fake_user_agent", || -> String {
        let ua: String = UserAgent().fake();
        ua
    });

    engine.register_fn("fake_http_method", || -> String {
        let mut rng = rand::rng();
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        let weights: [u32; 7] = [50, 25, 10, 5, 5, 3, 2];
        let total: u32 = weights.iter().sum();
        let mut pick = rng.random_range(0..total);
        for (i, &w) in weights.iter().enumerate() {
            if pick < w {
                return methods[i].into();
            }
            pick -= w;
        }
        "GET".into()
    });

    engine.register_fn("fake_http_path", || -> String {
        let mut rng = rand::rng();
        let paths = [
            "/",
            "/index.html",
            "/api/v1/users",
            "/api/v1/orders",
            "/api/v2/products",
            "/login",
            "/logout",
            "/health",
            "/metrics",
            "/api/v1/search",
            "/static/css/main.css",
            "/static/js/app.js",
            "/favicon.ico",
            "/api/v1/auth/token",
            "/dashboard",
            "/admin/settings",
            "/api/v1/events",
            "/api/v1/notifications",
            "/profile",
            "/register",
        ];
        let idx = rng.random_range(0..paths.len());
        paths[idx].into()
    });

    engine.register_fn("fake_http_status", || -> i64 {
        let mut rng = rand::rng();
        let statuses: [i64; 17] = [
            200, 200, 200, 200, 201, 204, 301, 302, 304, 400, 401, 403, 404, 404, 500, 502, 503,
        ];
        let idx = rng.random_range(0..statuses.len());
        statuses[idx]
    });

    engine.register_fn("fake_log_level", || -> String {
        let mut rng = rand::rng();
        let levels = [
            "TRACE", "DEBUG", "INFO", "INFO", "INFO", "WARN", "ERROR", "FATAL",
        ];
        let idx = rng.random_range(0..levels.len());
        levels[idx].into()
    });

    engine.register_fn("fake_java_class", || -> String {
        let mut rng = rand::rng();
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
    });

    engine.register_fn("fake_thread_name", || -> String {
        let mut rng = rand::rng();
        let names = [
            "http-nio-8080-exec",
            "pool-1-thread",
            "main",
            "worker",
            "scheduler",
            "async-dispatch",
            "kafka-consumer",
            "grpc-server",
        ];
        let idx = rng.random_range(0..names.len());
        let num = rng.random_range(1u16..20);
        format!("{}-{num}", names[idx])
    });

    engine.register_fn("fake_message", || -> String {
        let mut rng = rand::rng();
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
    });

    engine.register_fn("fake_port", || -> i64 {
        let mut rng = rand::rng();
        rng.random_range(1024i64..65535)
    });

    engine.register_fn("fake_protocol", || -> String {
        let mut rng = rand::rng();
        let protocols = ["TCP", "UDP", "ICMP", "HTTP", "HTTPS", "DNS", "SSH", "TLS"];
        let idx = rng.random_range(0..protocols.len());
        protocols[idx].into()
    });

    engine.register_fn("fake_action", || -> String {
        let mut rng = rand::rng();
        let actions = [
            "allow", "deny", "drop", "reject", "accept", "block", "redirect",
        ];
        let idx = rng.random_range(0..actions.len());
        actions[idx].into()
    });

    engine.register_fn("fake_severity", || -> String {
        let mut rng = rand::rng();
        let names = ["Low", "Medium", "High", "Critical", "Informational"];
        let idx = rng.random_range(0..names.len());
        names[idx].into()
    });
}

// ---------------------------------------------------------------------------
// Timestamp functions
// ---------------------------------------------------------------------------

fn register_timestamps(engine: &mut Engine) {
    engine.register_fn("now_iso", || -> String {
        chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    });

    engine.register_fn("now_epoch", || -> i64 {
        chrono::Utc::now().timestamp()
    });

    engine.register_fn("now_rfc3339", || -> String {
        chrono::Utc::now().to_rfc3339()
    });
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

fn register_utilities(engine: &mut Engine) {
    // uuid() — v4 UUID
    engine.register_fn("uuid", || -> String {
        let mut rng = rand::rng();
        let bytes: [u8; 16] = rng.random();
        format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF,
            (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3FFF) | 0x8000,
            u64::from_be_bytes([
                0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            ]),
        )
    });

    // fake_int(min, max) — random integer in range
    engine.register_fn("fake_int", |min: i64, max: i64| -> i64 {
        let mut rng = rand::rng();
        rng.random_range(min..=max)
    });

    // int_range(min, max) — alias
    engine.register_fn("int_range", |min: i64, max: i64| -> i64 {
        let mut rng = rand::rng();
        rng.random_range(min..=max)
    });

    // pick(array) — pick random element from a Rhai array
    engine.register_fn("pick", |arr: Array| -> rhai::Dynamic {
        if arr.is_empty() {
            return rhai::Dynamic::UNIT;
        }
        let mut rng = rand::rng();
        let idx = rng.random_range(0..arr.len());
        arr[idx].clone()
    });

    // weighted_bool(probability) — returns true with given probability (0.0..1.0)
    engine.register_fn("weighted_bool", |p: f64| -> bool {
        let mut rng = rand::rng();
        rng.random_bool(p.clamp(0.0, 1.0))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_ipv4_returns_valid_ip() {
        let mut engine = Engine::new();
        register_all(&mut engine);
        let result: String = engine.eval("fake_ipv4()").unwrap();
        assert!(result.parse::<Ipv4Addr>().is_ok(), "invalid IP: {result}");
    }

    #[test]
    fn uuid_has_correct_format() {
        let mut engine = Engine::new();
        register_all(&mut engine);
        let result: String = engine.eval("uuid()").unwrap();
        assert_eq!(result.len(), 36);
        assert_eq!(result.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn pick_from_array() {
        let mut engine = Engine::new();
        register_all(&mut engine);
        let result: String = engine.eval(r#"pick(["a", "b", "c"])"#).unwrap();
        assert!(["a", "b", "c"].contains(&result.as_str()), "unexpected: {result}");
    }

    #[test]
    fn fake_int_in_range() {
        let mut engine = Engine::new();
        register_all(&mut engine);
        let result: i64 = engine.eval("fake_int(10, 20)").unwrap();
        assert!((10..=20).contains(&result), "out of range: {result}");
    }

    #[test]
    fn weighted_bool_works() {
        let mut engine = Engine::new();
        register_all(&mut engine);
        // Always true
        let result: bool = engine.eval("weighted_bool(1.0)").unwrap();
        assert!(result);
        // Always false
        let result: bool = engine.eval("weighted_bool(0.0)").unwrap();
        assert!(!result);
    }

    #[test]
    fn timestamp_functions_return_strings() {
        let mut engine = Engine::new();
        register_all(&mut engine);
        let iso: String = engine.eval("now_iso()").unwrap();
        assert!(iso.contains('T'), "ISO missing T: {iso}");
        let epoch: i64 = engine.eval("now_epoch()").unwrap();
        assert!(epoch > 0);
        let rfc: String = engine.eval("now_rfc3339()").unwrap();
        assert!(rfc.contains('T'), "RFC3339 missing T: {rfc}");
    }
}
