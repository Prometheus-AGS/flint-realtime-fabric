# p24-c003-keto-view-seed

## Why

The ADR-007 media view-check authorizes on `subject = from_session` — a **fresh random
`SessionId` minted per WS connection** (`routes/signal.rs`). A seeded Keto tuple cannot predict
that ephemeral id, so a live authenticated run would ALWAYS be denied. The subject must be the
**authenticated identity** (the JWT `subject`), matching how publish/subscribe already build
their Keto tuple (`claims.subject`). Then a stable `(subject, "view", room)` tuple is seedable and
the live decode run (c004) can pass the real fail-closed check — no bypass.

## What Changes

- **Thread the verified subject into the media path.** `routes/signal.rs`: `resolve_tenant` →
  `resolve_identity` returning `(TenantId, subject)`; carry `subject` into `handle_signal_socket`
  and stamp it on the inbound `SignalEnvelope`. Add `SignalEnvelope.subject: Option<String>`
  (server-stamped; `None` for legacy paths).
- **Bridge uses the authenticated subject** for the `view` check when present, falling back to
  `from_session` when absent (existing tests unaffected).
- **Seed script** `scripts/seed-media-view.sh`: writes `(subject=<E2E_SUBJECT>, view,
  <E2E_ROOM>)` to Keto (write API) for the harness — a real grant, not a bypass.

## Impact

- `crates/frf-domain/src/signal.rs` (+ `subject` field), `crates/frf-gateway/src/routes/signal.rs`,
  `crates/frf-gateway/src/media_bridge.rs` (use `env.subject`), + a seed script.
- Rust tests: bridge authorizes on `subject` when set; falls back to `from_session` otherwise.
- No gate flip. Makes the live authenticated decode run (c004) actually reachable.
