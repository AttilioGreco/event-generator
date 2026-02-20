mod dashboard;

use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Response, StatusCode};
use axum::Json;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::config::model::FormatConfig;
use crate::format::{EventContext, build_formatter};
use crate::config::model::WebConfig;
use crate::stats::reporter::StreamStats;

const DEBUG_MAX_SAMPLES: usize = 20;

#[derive(Clone)]
struct AppState {
    streams: Vec<Arc<StreamStats>>,
    streams_by_name: HashMap<String, Arc<StreamStats>>,
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

pub async fn run_web_server(
    config: WebConfig,
    streams: Vec<Arc<StreamStats>>,
    cancel: CancellationToken,
) -> Result<()> {
    let auth = match (&config.username, &config.password) {
        (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => Some((u.clone(), p.clone())),
        _ => None,
    };

    let state = AppState {
        streams_by_name: streams
            .iter()
            .map(|s| (s.name.clone(), Arc::clone(s)))
            .collect(),
        streams,
        start_time: Instant::now(),
        auth,
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/assets/{*path}", get(asset_handler))
        .route("/api/debug/render", post(debug_render_handler))
        .route("/ws", get(ws_handler))
        .route("/ws/stream/{name}", get(ws_stream_handler))
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

async fn index_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }
    Html(dashboard::HTML).into_response()
}

async fn asset_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = check_auth(&state.auth, &headers) {
        return resp;
    }

    let (content_type, body) = match path.as_str() {
        "dashboard.css" => ("text/css; charset=utf-8", dashboard::CSS),
        "dashboard.js" => ("application/javascript; charset=utf-8", dashboard::JS),
        "alpinejs.min.js" => ("application/javascript; charset=utf-8", dashboard::ALPINE_JS),
        "debug_presets.txt" => ("text/plain; charset=utf-8", dashboard::DEBUG_PRESETS),
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap();
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .body(Body::from(body))
        .unwrap()
}

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

    let Some(stream) = state.streams_by_name.get(&name).cloned() else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Stream not found"))
            .unwrap();
    };

    ws.on_upgrade(move |socket| handle_ws_stream(socket, stream))
        .into_response()
}

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

    let sample_count = req.samples.unwrap_or(1).max(1).min(DEBUG_MAX_SAMPLES);
    let mut output = Vec::with_capacity(sample_count);

    for idx in 0..sample_count {
        let ctx = EventContext::new("debug".into(), idx as u64);
        output.push(formatter.format(&ctx));
    }

    Json(DebugRenderResponse { output, error: None }).into_response()
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut prev_totals: Vec<u64> = state.streams.iter().map(|_| 0).collect();

    loop {
        interval.tick().await;

        let elapsed = state.start_time.elapsed();
        let uptime_secs = elapsed.as_secs();

        let mut stream_data = Vec::new();
        let mut total_eps: u64 = 0;
        let mut total_events: u64 = 0;

        for (i, s) in state.streams.iter().enumerate() {
            let total = s.total_events.load(Ordering::Relaxed);
            let eps = total.saturating_sub(prev_totals[i]);
            prev_totals[i] = total;
            total_eps += eps;
            total_events += total;
            stream_data.push(serde_json::json!({
                "name": s.name,
                "destination": s.destination,
                "eps": eps,
                "total": total,
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

async fn handle_ws_stream(mut socket: WebSocket, stream: Arc<StreamStats>) {
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

fn check_auth(
    auth: &Option<(String, String)>,
    headers: &HeaderMap,
) -> Result<(), Response<Body>> {
    let Some((expected_user, expected_pass)) = auth else {
        return Ok(()); // No auth configured
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
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let has_gui = std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some();

        if !has_gui {
            anyhow::bail!(
                "no graphical session detected (DISPLAY/WAYLAND_DISPLAY missing). Open manually: {url}"
            );
        }

        if let Ok(status) = Command::new("xdg-open").arg(url).status() {
            if status.success() {
                return Ok(());
            }
        }

        if let Ok(status) = Command::new("gio").args(["open", url]).status() {
            if status.success() {
                return Ok(());
            }
        }

        anyhow::bail!(
            "could not open browser with xdg-open/gio. Open manually: {url}"
        )
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        anyhow::bail!("automatic browser open is not supported on this platform")
    }
}
