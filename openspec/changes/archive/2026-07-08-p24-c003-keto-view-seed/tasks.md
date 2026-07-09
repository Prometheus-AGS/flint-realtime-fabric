# Tasks — p24-c003-keto-view-seed

- [x] 1. Add `SignalEnvelope.subject: Option<String>`; in `routes/signal.rs` verify the token to a `(TenantId, subject)` and stamp `subject` on the inbound envelope; `media_bridge` uses `env.subject` for the `view` tuple, falling back to `from_session`. Rust tests (authz on subject-when-set; fallback otherwise). Keep files <500 lines.
- [x] 2. `scripts/seed-media-view.sh`: PUT `(subject=$E2E_SUBJECT, relation=view, object=$E2E_ROOM, namespace)` to the Keto write API; document usage for the c004 harness. QA gate.
