# Tasks — p16-c019

- [x] Add SIGTERM/SIGINT shutdown signal to the server
- [x] Drain in-flight requests before exit
- [x] Close WS streams cleanly
- [x] Test: SIGTERM drains rather than drops

## Implementation (#35 — graceful shutdown)

### The defect

`main` used `tokio::select! { serve, ctrl_c }` — on ctrl-c this ABORTS
`axum::serve` mid-request, dropping in-flight requests and WS streams. It also
never handled SIGTERM (the signal orchestrators send on stop).

### The fix

- **`shutdown_signal()`** (new, in `frf-gateway` lib): resolves on **SIGTERM**
  (`tokio::signal::unix`) OR **SIGINT** (ctrl-c). On non-Unix, ctrl-c only. A
  failed SIGTERM handler install logs a warning and falls back to ctrl-c rather
  than resolving spuriously.
- **`main`** now calls
  `axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await`.
  On signal, axum stops accepting new connections and **drains** in-flight
  requests AND WebSocket streams before `serve` returns — tasks 2 and 3 are both
  handled by axum's graceful shutdown (no manual per-connection tracking needed).
  After draining, the existing `shutdown_tx.send(true)` + task cleanup runs.

## Test (`tests/graceful_shutdown.rs`)

Driving a real SIGTERM in a test risks killing the runner, so the tests exercise
the same drain path via a controllable `oneshot` shutdown future:
- `in_flight_request_drains_on_shutdown` — a slow (300ms) handler; a request is
  started, shutdown is triggered 50ms in (handler mid-flight), and the request
  must still **complete successfully** ("done") — proving drain, not drop.
- `server_stops_accepting_after_shutdown` — a request works before shutdown, and
  the server exits cleanly within 5s after.

## Verification

- `cargo clippy -p frf-gateway --lib --bins` (default + dev-endpoints) → exit 0
- `cargo clippy -p frf-gateway --tests` → exit 0 (also fixed a pre-existing
  pedantic nit in admin_ui.rs surfaced by the tests run)
- `cargo test -p frf-gateway --test graceful_shutdown` → 2/2 pass
- `cargo test -p frf-gateway --lib` → 12/12; `cargo fmt --check` → clean
