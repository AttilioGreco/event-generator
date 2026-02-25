use std::net::Ipv4Addr;

use fake::Fake;
use fake::faker::internet::en::{IPv4, IPv6, UserAgent, Username};
use mlua::{Lua, Result, Table};
use rand::Rng;

/// Register all custom Lua globals (except `emit`, which is registered per-run).
pub fn register_all(lua: &Lua) -> Result<()> {
    register_fake_data(lua)?;
    register_timestamps(lua)?;
    register_utilities(lua)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Fake data generators
// ---------------------------------------------------------------------------

fn register_fake_data(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "fake_ipv4",
        lua.create_function(|_, ()| {
            let ip: Ipv4Addr = IPv4().fake();
            Ok(ip.to_string())
        })?,
    )?;

    globals.set(
        "fake_ipv6",
        lua.create_function(|_, ()| {
            let ip: String = IPv6().fake();
            Ok(ip)
        })?,
    )?;

    globals.set(
        "fake_hostname",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let prefixes = [
                "web", "app", "db", "srv", "gw", "fw", "proxy", "cache", "auth", "api",
            ];
            let idx = rng.random_range(0..prefixes.len());
            let num = rng.random_range(1u16..99);
            Ok(format!("{}-{num:02}", prefixes[idx]))
        })?,
    )?;

    globals.set(
        "fake_username",
        lua.create_function(|_, ()| {
            let name: String = Username().fake();
            Ok(name)
        })?,
    )?;

    globals.set(
        "fake_user_agent",
        lua.create_function(|_, ()| {
            let ua: String = UserAgent().fake();
            Ok(ua)
        })?,
    )?;

    globals.set(
        "fake_http_method",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
            let weights: [u32; 7] = [50, 25, 10, 5, 5, 3, 2];
            let total: u32 = weights.iter().sum();
            let mut pick = rng.random_range(0..total);
            for (i, &w) in weights.iter().enumerate() {
                if pick < w {
                    return Ok(methods[i].to_string());
                }
                pick -= w;
            }
            Ok("GET".to_string())
        })?,
    )?;

    globals.set(
        "fake_http_path",
        lua.create_function(|_, ()| {
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
            Ok(paths[idx].to_string())
        })?,
    )?;

    globals.set(
        "fake_http_status",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let statuses: [i64; 17] = [
                200, 200, 200, 200, 201, 204, 301, 302, 304, 400, 401, 403, 404, 404, 500, 502, 503,
            ];
            let idx = rng.random_range(0..statuses.len());
            Ok(statuses[idx])
        })?,
    )?;

    globals.set(
        "fake_log_level",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let levels = [
                "TRACE", "DEBUG", "INFO", "INFO", "INFO", "WARN", "ERROR", "FATAL",
            ];
            let idx = rng.random_range(0..levels.len());
            Ok(levels[idx].to_string())
        })?,
    )?;

    globals.set(
        "fake_java_class",
        lua.create_function(|_, ()| {
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
            Ok(classes[idx].to_string())
        })?,
    )?;

    globals.set(
        "fake_thread_name",
        lua.create_function(|_, ()| {
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
            Ok(format!("{}-{num}", names[idx]))
        })?,
    )?;

    globals.set(
        "fake_message",
        lua.create_function(|_, ()| {
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
            Ok(messages[idx].to_string())
        })?,
    )?;

    globals.set(
        "fake_port",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            Ok(rng.random_range(1024i64..65535))
        })?,
    )?;

    globals.set(
        "fake_protocol",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let protocols = ["TCP", "UDP", "ICMP", "HTTP", "HTTPS", "DNS", "SSH", "TLS"];
            let idx = rng.random_range(0..protocols.len());
            Ok(protocols[idx].to_string())
        })?,
    )?;

    globals.set(
        "fake_action",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let actions = [
                "allow", "deny", "drop", "reject", "accept", "block", "redirect",
            ];
            let idx = rng.random_range(0..actions.len());
            Ok(actions[idx].to_string())
        })?,
    )?;

    globals.set(
        "fake_severity",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let names = ["Low", "Medium", "High", "Critical", "Informational"];
            let idx = rng.random_range(0..names.len());
            Ok(names[idx].to_string())
        })?,
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Timestamp functions
// ---------------------------------------------------------------------------

fn register_timestamps(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "now_iso",
        lua.create_function(|_, ()| {
            Ok(chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string())
        })?,
    )?;

    globals.set(
        "now_epoch",
        lua.create_function(|_, ()| Ok(chrono::Utc::now().timestamp()))?,
    )?;

    globals.set(
        "now_rfc3339",
        lua.create_function(|_, ()| Ok(chrono::Utc::now().to_rfc3339()))?,
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

fn register_utilities(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals.set(
        "uuid",
        lua.create_function(|_, ()| {
            let mut rng = rand::rng();
            let bytes: [u8; 16] = rng.random();
            Ok(format!(
                "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                u16::from_be_bytes([bytes[4], bytes[5]]),
                u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF,
                (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3FFF) | 0x8000,
                u64::from_be_bytes([
                    0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
                ]),
            ))
        })?,
    )?;

    globals.set(
        "fake_int",
        lua.create_function(|_, (min, max): (i64, i64)| {
            let mut rng = rand::rng();
            Ok(rng.random_range(min..=max))
        })?,
    )?;

    globals.set(
        "int_range",
        lua.create_function(|_, (min, max): (i64, i64)| {
            let mut rng = rand::rng();
            Ok(rng.random_range(min..=max))
        })?,
    )?;

    // pick(table) — pick a random element from a Lua table (1-based array)
    globals.set(
        "pick",
        lua.create_function(|_, table: Table| {
            let len = table.raw_len();
            if len == 0 {
                return Ok(mlua::Value::Nil);
            }
            let mut rng = rand::rng();
            let idx = rng.random_range(1..=len as i64);
            table.get::<mlua::Value>(idx)
        })?,
    )?;

    globals.set(
        "weighted_bool",
        lua.create_function(|_, p: f64| {
            let mut rng = rand::rng();
            Ok(rng.random_bool(p.clamp(0.0, 1.0)))
        })?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};

    fn make_lua() -> Lua {
        let lua = Lua::new_with(
            StdLib::STRING | StdLib::TABLE | StdLib::MATH,
            LuaOptions::default(),
        )
        .unwrap();
        register_all(&lua).unwrap();
        lua
    }

    #[test]
    fn fake_ipv4_returns_valid_ip() {
        let lua = make_lua();
        let result: String = lua.load("return fake_ipv4()").eval().unwrap();
        assert!(result.parse::<Ipv4Addr>().is_ok(), "invalid IP: {result}");
    }

    #[test]
    fn uuid_has_correct_format() {
        let lua = make_lua();
        let result: String = lua.load("return uuid()").eval().unwrap();
        assert_eq!(result.len(), 36);
        assert_eq!(result.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn pick_from_table() {
        let lua = make_lua();
        let result: String = lua.load(r#"return pick({"a", "b", "c"})"#).eval().unwrap();
        assert!(
            ["a", "b", "c"].contains(&result.as_str()),
            "unexpected: {result}"
        );
    }

    #[test]
    fn fake_int_in_range() {
        let lua = make_lua();
        let result: i64 = lua.load("return fake_int(10, 20)").eval().unwrap();
        assert!((10..=20).contains(&result), "out of range: {result}");
    }

    #[test]
    fn weighted_bool_works() {
        let lua = make_lua();
        let t: bool = lua.load("return weighted_bool(1.0)").eval().unwrap();
        assert!(t);
        let f: bool = lua.load("return weighted_bool(0.0)").eval().unwrap();
        assert!(!f);
    }

    #[test]
    fn timestamp_functions_return_strings() {
        let lua = make_lua();
        let iso: String = lua.load("return now_iso()").eval().unwrap();
        assert!(iso.contains('T'), "ISO missing T: {iso}");
        let epoch: i64 = lua.load("return now_epoch()").eval().unwrap();
        assert!(epoch > 0);
        let rfc: String = lua.load("return now_rfc3339()").eval().unwrap();
        assert!(rfc.contains('T'), "RFC3339 missing T: {rfc}");
    }
}
