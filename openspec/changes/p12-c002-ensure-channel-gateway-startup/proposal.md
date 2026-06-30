# p12-c002 — Ensure Fixture Channel at Gateway Startup

## Phase
phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Summary

Call `broker.ensure_channel(fixture_channel)` during gateway startup (after
`IggyBroker::new` succeeds) to pre-create the Iggy stream and topic used by
Layer 3 E2E tests. Without this, the first `POST /v1/publish` in Stage 10 fails
with "stream not found" because `IggyBroker::publish` does not auto-create the
stream/topic.

## Files to Modify

- `crates/frf-gateway/src/main.rs` — after line 83 (`IggyBroker::new`), add:
  ```rust
  // Pre-create the fixture channel used by integration and Layer 3 E2E tests.
  // IggyBroker::publish requires the stream + topic to exist first.
  {
      use frf_domain::Channel;
      use uuid::Uuid;
      let fixture_tenant = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
          .expect("fixture tenant UUID is static");
      let ch = Channel { tenant_id: fixture_tenant, path: "entities".into() };
      if let Err(e) = broker.ensure_channel(ch).await {
          tracing::warn!(error = %e, "fixture channel pre-creation failed (non-fatal)");
      }
  }
  ```

## Design Notes

`LogBroker::ensure_channel` is idempotent — calling it on an already-existing
stream/topic is a no-op (Iggy returns an "already exists" response which the
adapter ignores). Making the error non-fatal means a gateway restart after
topic pre-creation doesn't crash.

The fixture tenant UUID (`00000000-0000-0000-0000-000000000001`) matches the
`CDC_TENANT_ID` and `CDC_CHANNEL_PATH` set in compose.yml. This ensures CDC
events and publish events land in the same Iggy stream/topic combination.

**Security note**: This pre-creation is a startup optimization for the compose
development stack. In production, channels are created by tenant provisioning
flows, not gateway startup. The fixture channel is identifiable by its all-zeros
UUID prefix — it is not a production tenant.

## Exit Criteria

- Gateway starts without panic when Iggy is reachable
- `docker compose exec iggy-server iggy --transport tcp topics list --stream-id 1` shows the fixture topic after gateway startup
- `POST /v1/publish` with fixture tenant + `entities` path succeeds (201) on first attempt without a preceding `ensure_channel` call from the client
