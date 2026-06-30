# p7-c004 — Gateway str0m Wiring + Dev Inject Spine + Registry Config

## Summary

Wire `frf-media-str0m` into the gateway (runtime selection via `SFU_MODE`
env var). Fix `/dev/inject-federation-event` to publish to the `LogBroker`
spine. Make `TenantActorRegistry` sweep interval configurable via
`GatewayConfig`.

## Changes

### `crates/frf-gateway/Cargo.toml`
- Add `frf-media-str0m = { path = "../frf-media-str0m" }`

### `crates/frf-gateway/src/config.rs`
- Add `sfu_mode: String` — reads `SFU_MODE` env var, default `"livekit"`
- Add `registry_sweep_interval_secs: u64` — reads `REGISTRY_SWEEP_INTERVAL_SECS`, default 60

### `crates/frf-gateway/src/main.rs`
- In `main()`, after reading config, match on `config.sfu_mode`:
  - `"str0m"` → construct `StrOmSignaler`, wrap in `Arc`
  - `"livekit"` (default) → construct `LiveKitSignaling`, wrap in `Arc`
- Pass chosen signaler into `AppState.media_signaler`
- Pass `config.registry_sweep_interval_secs` to `TenantActorRegistry::spawn_eviction_task`

### `crates/frf-gateway/src/routes/dev.rs`

Upgrade `inject_federation_event` to actually publish:

```rust
#[cfg(debug_assertions)]
pub mod inject {
    use axum::{Json, extract::State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use frf_ports::{LogBroker, AuthzProvider, IdentityVerifier, MediaSignaler, AgentEventBus};
    use frf_domain::EventEnvelope;
    use serde::Deserialize;
    use crate::AppStateArc;

    #[derive(Debug, Deserialize)]
    pub struct InjectFederationEventRequest {
        pub protocol: String,
        pub source: String,
        pub content: serde_json::Value,
    }

    pub async fn inject_federation_event<L, A, I, M, B>(
        State(state): State<AppStateArc<L, A, I, M, B>>,
        Json(body): Json<InjectFederationEventRequest>,
    ) -> impl IntoResponse
    where
        L: LogBroker + Send + Sync + 'static,
        A: AuthzProvider + Send + Sync + 'static,
        I: IdentityVerifier + Send + Sync + 'static,
        M: MediaSignaler + 'static,
        B: AgentEventBus + 'static,
    {
        let envelope = EventEnvelope::new_dev_injection(body.protocol, body.source, body.content);
        match state.log_broker.publish(envelope).await {
            Ok(()) => {
                tracing::debug!(protocol = %body.protocol, source = %body.source, "dev: federation event injected into spine");
                StatusCode::ACCEPTED
            }
            Err(e) => {
                tracing::error!(error = %e, "dev: federation event injection failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
```

### `crates/frf-gateway/src/lib.rs`
- Update `#[cfg(debug_assertions)]` router block to pass the typed state generic to `inject_federation_event`

### `crates/frf-domain/src/events.rs` (or appropriate file)
- Add `EventEnvelope::new_dev_injection(protocol, source, content)` constructor
  (builds a minimal `EventEnvelope` for dev injection; uses a nil tenant_id for now)

### `crates/frf-librefang/src/registry.rs`
- Add `sweep_interval: Duration` parameter to `spawn_eviction_task`
- Callers in gateway `main.rs` pass `Duration::from_secs(config.registry_sweep_interval_secs)`

## Quality Gates

- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
- `cargo test -p frf-gateway`
