mod dashboard;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{HeaderMap, Response, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use tokio_util::sync::CancellationToken;

use crate::config::model::WebConfig;
use crate::stats::reporter::StreamStats;

#[derive(Clone)]
struct AppState {
    streams: Vec<Arc<StreamStats>>,
    start_time: Instant,
    auth: Option<(String, String)>,
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
        streams,
        start_time: Instant::now(),
        auth,
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen).await?;
    eprintln!("[web] dashboard at http://{}", config.listen);

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
