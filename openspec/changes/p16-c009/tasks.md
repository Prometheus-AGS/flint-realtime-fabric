# Tasks — p16-c009

- [x] Stop registering broken bridge directions as bidirectional
- [x] Return explicit unsupported signal (not empty/Err) where surfaced
- [x] Apply configured tenant/channel IDs where the bridge stays wired
- [x] Document Matrix/ATProto federation as deferred to a future phase

## The honest picture (each bridge is half-implemented, not fully broken)

| Bridge  | Inbound (subscribe)              | Outbound (send)                    |
|---------|----------------------------------|------------------------------------|
| Matrix  | `stream::empty()` — STUB         | real HTTP PUT — **implemented**    |
| ATProto | jetstream — **implemented**      | returns `Err` unsupported          |

So neither is fully bidirectional. Per operator decision, federation is DEFERRED for v1.

## Implementation

### Federation is now opt-in and off by default (tasks 1, 2)

New `FEDERATION_ENABLED` config (default false). `build_federation_bridges` returns no
bridges unless it is true — nothing is silently wired. If Matrix/ATProto env vars are set
but federation is disabled, the gateway logs a clear warning rather than half-activating.
When enabled, each bridge logs exactly which direction is supported and which is a
stub/unsupported (Matrix: "OUTBOUND supported; INBOUND is a stub"; ATProto: "INBOUND
supported; OUTBOUND unsupported for v1") — no silent empty/Err.

### Configured tenant/channel IDs (task 3 — closes H5)

Replaced the per-boot `TenantId::new()`/`ChannelId::new()` with configured
`FEDERATION_TENANT_ID` / `FEDERATION_CHANNEL_ID` (`resolve_federation_ids`). Without the
tenant set, it falls back to a random ID **with a loud warning** that ingested events
won't match any subscriber and change every restart — so the footgun is visible, not
silent. This fixes the H5 defect (federated events landing under a random tenant no JWT
matches).

### Documentation (task 4)

`.env.example` gained a federation section documenting the half-implemented status, the
opt-in flag, and the required tenant/channel IDs. The config field doc comments state the
direction limitations inline. Full deferral is also recorded in this change and the phase
goals (G2.5 was explicitly scoped as defer).

## Tests

`config.rs` test `federation_is_off_by_default` asserts the safe-by-default property
(federation disabled, no tenant/channel) in `test_default()`.

## Verification

- `cargo clippy -p frf-gateway --lib --bins` (default + dev-endpoints) → exit 0
- `cargo test -p frf-gateway --lib` → 8/8 pass
- `cargo fmt --check -p frf-gateway` → clean

## Follow-up for docs (G5)

`FEDERATION_ENABLED` / `FEDERATION_TENANT_ID` / `FEDERATION_CHANNEL_ID` to be included in
the full env reference (c022), and the federation-deferred status noted in the security
model / README updates (c024/c025).
