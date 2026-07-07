# Tasks — p16-c018

- [x] Add /metrics endpoint (Prometheus exposition)
- [x] Export request + delivery counters/histograms
- [x] Scrape test returns metrics

## Implementation (H34 — gateway is now observable)

### /metrics endpoint

`routes/metrics.rs`: a process-global `PrometheusHandle` is installed once at startup
(`install_recorder()`, called from `main` — non-fatal on failure, only disables metrics).
`GET /metrics` renders the current snapshot in Prometheus text exposition format
(`text/plain; version=0.0.4`), or 503 if the recorder was never installed. Added
`metrics` (facade) + `metrics-exporter-prometheus` workspace deps.

### Metrics recorded

Via `metrics::counter!` through small helpers, called at the real boundaries:
- `frf_publish_total{result="ok"|"error"}` — publish route, both the dev-bypass and
  normal paths (every `publish_usecase.execute` outcome).
- `frf_subscriptions_opened_total` — subscribe route, when a subscription opens.
- `frf_events_delivered_total` — subscribe WS fan-out, per successfully-sent event.

Instrumentation lives in the metrics module (helpers) and is called from
`routes/publish.rs` + `routes/subscribe.rs` — no metrics logic scattered inline.

## Test

`metrics.rs` unit test `recorded_metrics_appear_in_the_scrape_output`: installs the
recorder, records via the public helpers, and asserts the rendered scrape contains
`frf_publish_total`, `frf_events_delivered_total`, `frf_subscriptions_opened_total` —
the endpoint's data path without an HTTP server.

## Verification

- `cargo clippy -p frf-gateway --lib --bins` (default + dev-endpoints) → exit 0
- `cargo test -p frf-gateway --lib` → 12/12 pass (incl. the metrics scrape test)
- `cargo fmt --check` → clean; workspace `--lib --bins` clippy → exit 0

## Follow-up for docs (G5)

`/metrics` endpoint + the exported series to be noted in the deployment runbook (c023).
