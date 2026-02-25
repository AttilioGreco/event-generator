# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
just build           # Dashboard + release binary (default)
just dev             # Debug binary only
just dev-full        # Dashboard + debug binary

# Run
just run             # Run with config/example.toml
just run CONFIG=path/to/config.toml

# Test & Lint
just test            # cargo test
just lint            # cargo clippy -- -D warnings
just fmt             # cargo fmt --check
just fmt-fix         # cargo fmt
just check           # fmt + lint + test

# Dashboard (frontend)
just dashboard-install   # Install bun dependencies
just dashboard-build     # Production React build
just dashboard-dev       # Dev server with hot reload (proxies to backend)

# Docker
just docker-build    # Multi-stage production image
just docker-run      # Run container with config mount

# Pre-release (runs all 6 checks: fmt, clippy, test, release build, dashboard build, docker build)
just pre-release
just release         # pre-release + publish script
```

**Run a single test:**
```bash
cargo test test_name
cargo test --lib engine::rate  # test a specific module
```

**Dashboard dev setup:** start the Rust binary first (`just dev`), then `just dashboard-dev` — the Vite dev server proxies API calls to `localhost:8080`.

## Architecture

### High-Level Flow

```
main.rs → load TOML config → StreamManager::load_and_start()
                                        ↓
                         per-stream task: run_stream()
                              RateController tick
                                    ↓
                         EventContext (fake data fields)
                                    ↓
                         LogFormatter → formatted string
                                    ↓
                         OutputSink::send()

Web server (Axum) ← REST API → StreamManager (start/stop/reload)
Stats reporter task → stderr every 5s + WebSocket broadcast
```

### Key Design Decisions

**Enum dispatch, not trait objects** — `LogFormatter` and `OutputSink` are enums. This avoids async trait object-safety issues with Tokio. Adding a new format/output means adding a variant and matching in `format()` / `send()`.

**CancellationToken hierarchy** — Global token in `main.rs`, per-stream child tokens via `global_cancel.child_token()`. Cancelling global stops everything; individual streams can be stopped independently without touching others.

**StreamManager** (`src/engine/manager.rs`) — Arc<RwLock<StreamManagerInner>> wrapping all stream state. The `Inner` struct holds `streams: HashMap<String, StreamHandle>` plus `stream_order: Vec<String>` to preserve config order. Methods: `load_and_start`, `start_stream`, `stop_stream`, `apply_config` (hot reload), `wait_all`.

**EventContext** — Created per-event, carries timestamp, sequence, stream_name, and a `fields: HashMap<String, String>` with 16 pre-populated fake fields (src_ip, dst_ip, username, hostname, http_method, http_status, log_level, etc.). Formatters read from this map.

### Module Map

| Path | Responsibility |
|------|---------------|
| `src/config/model.rs` | TOML data structures (AppConfig, StreamConfig, FormatConfig, OutputConfig, WaveConfig) |
| `src/config/validate.rs` | Eager validation — collects all errors before returning |
| `src/data/fake.rs` | FakeDataProvider — generates 16 fields per event using fake-rs + rand |
| `src/engine/manager.rs` | StreamManager — stream lifecycle (start/stop/reload/error recovery) |
| `src/engine/stream.rs` | `run_stream()` — main event loop per stream |
| `src/engine/rate.rs` | RateController — tokio Interval + optional WaveModulator |
| `src/engine/wave.rs` | WaveModulator — sine/sawtooth/square EPS curves |
| `src/format/mod.rs` | LogFormatter enum + EventContext + `build_formatter()` |
| `src/output/mod.rs` | OutputSink enum + `build_sink()` |
| `src/stats/reporter.rs` | StreamStats (AtomicU64 counters), periodic reporter, WebSocket broadcast |
| `src/web/mod.rs` | Axum REST API + WebSocket routes, optional BasicAuth |
| `src/web/dashboard.rs` | Embeds React build via rust-embed |
| `dashboard/` | React 19 + React Router 7 + Tailwind + CodeMirror frontend |

### Web API Routes

- `GET /api/config` / `PUT /api/config` — get/apply TOML config (hot reload)
- `GET /api/stats` — all stream stats snapshot
- `POST /api/streams/{name}/control` — `{"action": "start"|"stop"}`
- `GET /api/streams/{name}/tail?limit=N` — recent events buffer
- `WS /api/streams/{name}/events` — live event tail
- `POST /api/debug/render` — preview formatter output without running stream

### Formats & Outputs

**9 formats:** `syslog_rfc5424`, `syslog_rfc3164`, `cef`, `leef`, `clf`, `json`, `java_log4j`, `java_logback`, `template` (Tera with 14 custom functions), `script` (Lua via mlua)

**5 outputs:** `stdout`, `file`, `tcp`, `udp`, `http` (batched, configurable headers)

### Dashboard (React)

Three routes: `/` (live dashboard with stream controls), `/config` (TOML editor with validate/apply), `/studio` (Lua script preview). Built with Vite, embedded into the binary at compile time via rust-embed. `dashboard/build/` must exist before `cargo build` when web feature is used.

## Critical Gotchas

- **fake-rs v3 uses rand 0.8 internally** — always use `.fake()` (no rng arg), never `fake_with_rng()` with our rand 0.9 instance. It will fail to compile or produce wrong results.
- **`tokio::time::interval` requires the Tokio reactor** — all tests touching rate/wave/stream must use `#[tokio::test]`, not plain `#[test]`.
- **`stdout` output + pipe with `head`/`timeout`** — causes "Broken pipe" errors. This is expected behavior, not a bug.
- **Docker dashboard build** — Dockerfile uses `node` to run Vite (not `bun run build`) because `react-dom/server.bun.js` lacks `renderToPipeableStream`. Bun is used only for `bun install`.
- **`eprintln!` for internal logging** — stdout is reserved exclusively for generated events (stdout sink). All diagnostic output goes to stderr.
