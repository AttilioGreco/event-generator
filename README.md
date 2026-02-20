# event-generator

Synthetic log event generator for testing log pipelines, parsers, SIEM ingestion, alerting rules, and transport integrations.


## Purpose
`event-generator` produces continuous event streams with configurable rate, format, and destination. It is designed for:

- validating ingestion endpoints and buffering behavior
- testing parser compatibility across multiple log standards
- simulating realistic traffic/load patterns
- debugging custom templates before deployment

## Core capabilities

- Multiple stream definitions in one config file
- Independent rate per stream (`eps`) with optional wave modulation
- Multiple output transports (`stdout`, file, TCP, UDP, HTTP)
- Multiple log formats (syslog, CEF, LEEF, CLF, JSON, Java patterns, custom template)
- Web dashboard with live stats and per-stream live tail view
- Built-in debug page for template preview and preset examples

## Configuration model
Configuration is TOML-based (see `config/example.toml`).

### Top-level sections

- `[defaults]`
  - `rate` (optional): default events/sec when stream rate is omitted

- `[web]` (optional)
  - `enabled` (bool, default `true`)
  - `listen` (string, default `0.0.0.0:8080`)
  - `auto_open_browser` (bool, default `false`)
  - `username` / `password` (optional BasicAuth; both required)

- `[[streams]]` (required, at least one)
  - `name` (required)
  - `enabled` (bool, default `true`)
  - `[streams.format]` (required)
  - `[streams.output]` (required)
  - `[streams.rate]` (optional if covered by defaults or wave)

### `streams.format`

- `type`: one of
  - `syslog_rfc5424`
  - `syslog_rfc3164`
  - `cef`
  - `leef`
  - `clf`
  - `json`
  - `java_log4j`
  - `java_logback`
  - `template`

Optional format-specific fields:

- Syslog: `facility`, `severity`, `app_name`
- CEF/LEEF: `vendor`, `product`, `version`, `device_event_class_id`
- JSON: `extra_fields`
- Template: `template_file` or `template_inline`

### `streams.output`

- `type`: one of `stdout`, `file`, `tcp`, `udp`, `http`

Required by output type:

- `file`: `path`
- `tcp` / `udp`: `host`, `port`
- `http`: `url`

Optional HTTP fields:

- `method` (`POST`/`PUT`)
- `headers`
- `batch_size`
- `timeout_ms`

### `streams.rate`

- `eps` (events/sec)
- optional `[streams.rate.wave]`
  - `shape`: `sine`, `sawtooth`, `square`
  - `period_secs`
  - `min`
  - `max`

## Quick start

### Run with example config

```bash
cargo run -- --config config/example.toml
```

### With just

```bash
just run
```

### Build

```bash
just build
```

## Dashboard and debug UI

When web is enabled, dashboard is served on `http://<listen>`.

The UI includes:

- global throughput and counters
- per-stream live accordion with in-browser tail and search
- bounded in-memory UI buffer per stream (prevents browser memory growth)
- debug tab to preview generated output from templates and format presets

## Docker

### Build local image

```bash
just docker-build
```

### Build debug image (linux/amd64)

```bash
just docker-build-debug
```

### Run with mounted config

```bash
just docker-run CONFIG=config/example.toml
```

```bash
just docker-run-debug CONFIG=config/example.toml
```

## Release workflow

### Interactive local release

```bash
just release
```

This flow can run tests, create a commit, create a tag, and push branch/tag.

### GitHub Actions

- Tag-based release workflow publishes versioned artifacts on `v*` tags
- Continuous debug Docker workflow builds `event-generator-debug:latest` (linux/amd64)

## Operational notes

- If `web.auto_open_browser = true`, browser opening depends on desktop session availability.
- In headless/container environments, open the dashboard URL manually.
- For template format, invalid templates are reported during render/build of formatter.
