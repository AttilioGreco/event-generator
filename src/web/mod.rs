mod dashboard;

use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::Instant;

use anyhow::Result;
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::config::model::FormatConfig;
use crate::config::model::WebConfig;
use crate::engine::manager::{StreamManager, StreamStatus};
use crate::format::{EventContext, build_formatter};

const MAX_SAMPLES: usize = 20;

#[derive(Clone)]
struct AppState {
    manager: StreamManager,
    start_time: Instant,
    auth: Option<(String, String)>,
}

#[derive(Debug, Deserialize)]
struct DebugRenderRequest {
    format_type: String,
    template: Option<String>,
    samples: Option<usize>,
}

#[derive(Debug, Serialize)]
struct DebugRenderResponse {
    output: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScriptRunRequest {
    code: String,
    samples: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ScriptRunResponse {
    output: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfigUpdateRequest {
    config: String,
}

#[derive(Debug, Serialize)]
struct ConfigUpdateResponse {
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct StreamStatusResponse {
    name: String,
    destination: String,
    status: StreamStatus,
    error: Option<String>,
    total_events: u64,
}

pub async fn run_web_server(
    config: WebConfig,
    manager: StreamManager,
    cancel: CancellationToken,
) -> Result<()> {
    let auth = match (&config.username, &config.password) {
        (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => Some((u.clone(), p.clone())),
        _ => None,
    };

    let state = AppState {
        manager,
        start_time: Instant::now(),
        auth,
    };

    let app = Router::new()
        .route("/api/debug/render", post(debug_render_handler))
        .route("/api/script/run", post(script_run_handler))
        .route("/api/config", get(get_config_handler))
        .route("/api/config", put(put_config_handler))
        .route("/api/streams", get(list_streams_handler))
        .route("/api/streams/start-all", post(start_all_handler))
        .route("/api/streams/stop-all", post(stop_all_handler))
        .route("/api/streams/{name}/start", post(start_stream_handler))
        .route("/api/streams/{name}/stop", post(stop_stream_handler))
        .route("/ws", get(ws_handler))
        .route("/ws/stream/{name}", get(ws_stream_handler))
        .fallback(get(static_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await?;
    eprintln!("[web] dashboard at http://{}", config.listen);

    if config.auto_open_browser {
        let local_addr = listener.local_addr()?;
        let browser_url = dashboard_browser_url(local_addr);
        if let Err(err) = open_browser(&browser_url) {
            eprintln!("[web] failed to open browser automatically: {err}");
        } else {
            eprintln!("[web] opened browser at {browser_url}");
        }
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancel.cancelled().await;
        })
        .await?;

    Ok(())
}

async fn static_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let path = uri.path().trim_start_matches('/');

    let (file, effective_path) = if path.is_empty() {
        (dashboard::DashboardAssets::get("index.html"), "index.html")
    } else {
        (dashboard::DashboardAssets::get(path), path)
    };

    match file {
        Some(content) => {
            let mime = mime_from_path(effective_path);
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime)
                .body(Body::from(content.data.to_vec()))
                .unwrap()
        }
        None => match dashboard::DashboardAssets::get("index.html") {
            Some(index) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Body::from(index.data.to_vec()))
                .unwrap(),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Dashboard not built"))
                .unwrap(),
        },
    }
}

fn mime_from_path(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        _ => "application/octet-stream",
    }
}

// --- Config API ---

async fn get_config_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let config_text = state.manager.config_text().await;
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(config_text))
        .unwrap()
}

async fn put_config_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ConfigUpdateRequest>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    match state.manager.apply_config(req.config).await {
        Ok(()) => Json(ConfigUpdateResponse {
            success: true,
            error: None,
        })
        .into_response(),
        Err(e) => Json(ConfigUpdateResponse {
            success: false,
            error: Some(format!("{e:#}")),
        })
        .into_response(),
    }
}

// --- Streams API ---

async fn list_streams_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let infos = state.manager.stream_infos().await;
    let list: Vec<StreamStatusResponse> = infos
        .into_iter()
        .map(|i| StreamStatusResponse {
            name: i.name,
            destination: i.destination,
            status: i.status,
            error: i.error,
            total_events: i.stats.total_events.load(Ordering::Relaxed),
        })
        .collect();

    Json(list).into_response()
}

async fn start_stream_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    match state.manager.start_stream(&name).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            eprintln!("[web] start stream '{}' failed: {:#}", name, e);
            (StatusCode::BAD_REQUEST, format!("{e:#}")).into_response()
        }
    }
}

async fn stop_stream_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    match state.manager.stop_stream(&name).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            eprintln!("[web] stop stream '{}' failed: {:#}", name, e);
            (StatusCode::BAD_REQUEST, format!("{e:#}")).into_response()
        }
    }
}

async fn start_all_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    match state.manager.start_all().await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn stop_all_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    match state.manager.stop_all().await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- WebSocket handlers ---

async fn ws_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }
    ws.on_upgrade(move |socket| handle_ws(socket, state))
        .into_response()
}

async fn ws_stream_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let Some(stream) = state.manager.stream_by_name(&name).await else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Stream not found"))
            .unwrap();
    };

    ws.on_upgrade(move |socket| handle_ws_stream(socket, stream))
        .into_response()
}

// --- Debug/Script handlers ---

async fn debug_render_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DebugRenderRequest>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let format_type = req.format_type.trim();
    if format_type.is_empty() {
        return Json(DebugRenderResponse {
            output: Vec::new(),
            error: Some("format_type is required".into()),
        })
        .into_response();
    }

    let template_inline = req.template.and_then(|t| {
        let trimmed = t.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    if format_type == "template" && template_inline.is_none() {
        return Json(DebugRenderResponse {
            output: Vec::new(),
            error: Some("template content is required for type=template".into()),
        })
        .into_response();
    }

    let config = FormatConfig {
        format_type: format_type.to_string(),
        facility: None,
        severity: None,
        app_name: None,
        vendor: None,
        product: None,
        version: None,
        device_event_class_id: None,
        extra_fields: None,
        template_file: None,
        template_inline,
        script_file: None,
        script_inline: None,
        max_operations: None,
    };

    let formatter = match build_formatter(&config) {
        Ok(f) => f,
        Err(e) => {
            return Json(DebugRenderResponse {
                output: Vec::new(),
                error: Some(e.to_string()),
            })
            .into_response();
        }
    };

    let sample_count = req.samples.unwrap_or(1).clamp(1, MAX_SAMPLES);
    let mut output = Vec::with_capacity(sample_count);

    for idx in 0..sample_count {
        let ctx = EventContext::new("debug".into(), idx as u64);
        output.push(formatter.format(&ctx));
    }

    Json(DebugRenderResponse {
        output,
        error: None,
    })
    .into_response()
}

async fn script_run_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ScriptRunRequest>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let code = req.code.trim();
    if code.is_empty() {
        return Json(ScriptRunResponse {
            output: Vec::new(),
            error: Some("code is required".into()),
        })
        .into_response();
    }

    let engine = match crate::script::ScriptEngine::from_inline(code, 10_000) {
        Ok(e) => e,
        Err(e) => {
            return Json(ScriptRunResponse {
                output: Vec::new(),
                error: Some(e.to_string()),
            })
            .into_response();
        }
    };

    let sample_count = req.samples.unwrap_or(1).clamp(1, MAX_SAMPLES);
    let mut output = Vec::with_capacity(sample_count);

    for _ in 0..sample_count {
        let result = engine.run();
        output.push(result);
    }

    Json(ScriptRunResponse {
        output,
        error: None,
    })
    .into_response()
}

// --- WebSocket implementation ---

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut prev_totals: HashMap<String, u64> = HashMap::new();

    loop {
        interval.tick().await;

        let elapsed = state.start_time.elapsed();
        let uptime_secs = elapsed.as_secs();

        let infos = state.manager.stream_infos().await;

        let mut stream_data = Vec::new();
        let mut total_eps: u64 = 0;
        let mut total_events: u64 = 0;

        for info in &infos {
            let total = info.stats.total_events.load(Ordering::Relaxed);
            let prev = prev_totals.get(&info.name).copied().unwrap_or(0);
            let eps = total.saturating_sub(prev);
            prev_totals.insert(info.name.clone(), total);
            total_eps += eps;
            total_events += total;
            let status_str = match &info.status {
                StreamStatus::Running => "running",
                StreamStatus::Stopped => "stopped",
                StreamStatus::Error => "error",
            };
            stream_data.push(serde_json::json!({
                "name": info.name,
                "destination": info.destination,
                "eps": eps,
                "total": total,
                "status": status_str,
                "error": info.error,
            }));
        }

        let payload = serde_json::json!({
            "uptime_secs": uptime_secs,
            "total_eps": total_eps,
            "total_events": total_events,
            "streams": stream_data,
        });

        let msg = Message::Text(payload.to_string().into());
        if socket.send(msg).await.is_err() {
            return;
        }
    }
}

async fn handle_ws_stream(
    mut socket: WebSocket,
    stream: std::sync::Arc<crate::stats::reporter::StreamStats>,
) {
    let snapshot = serde_json::json!({
        "type": "snapshot",
        "events": stream.recent_events_snapshot(),
    });

    if socket
        .send(Message::Text(snapshot.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    let mut rx = stream.subscribe_events();

    loop {
        match rx.recv().await {
            Ok(event) => {
                let payload = serde_json::json!({
                    "type": "event",
                    "event": event,
                });
                if socket
                    .send(Message::Text(payload.to_string().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                return;
            }
        }
    }
}

// --- Auth ---

#[allow(clippy::result_large_err)]
fn check_auth(auth: &Option<(String, String)>, headers: &HeaderMap) -> Result<(), Response<Body>> {
    let Some((expected_user, expected_pass)) = auth else {
        return Ok(());
    };

    let Some(auth_header) = headers.get("authorization") else {
        return Err(unauthorized_response());
    };

    let Ok(auth_str) = auth_header.to_str() else {
        return Err(unauthorized_response());
    };

    if !auth_str.starts_with("Basic ") {
        return Err(unauthorized_response());
    }

    let Ok(decoded) = BASE64.decode(&auth_str[6..]) else {
        return Err(unauthorized_response());
    };

    let Ok(credentials) = String::from_utf8(decoded) else {
        return Err(unauthorized_response());
    };

    let expected = format!("{expected_user}:{expected_pass}");
    if credentials == expected {
        Ok(())
    } else {
        Err(unauthorized_response())
    }
}

fn unauthorized_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic realm=\"event-generator\"")
        .body(Body::from("Unauthorized"))
        .unwrap()
}

fn dashboard_browser_url(addr: std::net::SocketAddr) -> String {
    if addr.ip().is_unspecified() {
        format!("http://localhost:{}", addr.port())
    } else {
        format!("http://{addr}")
    }
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", "", url]).spawn()?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let has_gui =
            std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();

        if !has_gui {
            anyhow::bail!(
                "no graphical session detected (DISPLAY/WAYLAND_DISPLAY missing). Open manually: {url}"
            );
        }

        if let Ok(status) = Command::new("xdg-open").arg(url).status()
            && status.success()
        {
            return Ok(());
        }

        if let Ok(status) = Command::new("gio").args(["open", url]).status()
            && status.success()
        {
            return Ok(());
        }

        anyhow::bail!("could not open browser with xdg-open/gio. Open manually: {url}")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        anyhow::bail!("automatic browser open is not supported on this platform")
    }
}
