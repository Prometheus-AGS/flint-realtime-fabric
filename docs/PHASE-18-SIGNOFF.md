# Phase-18 Sign-off — media, federation & auth-flow

> Date: 2026-07-07 · Closing verification for phase-18 (p18-c010). Extends the phase-17
> release sign-off (`docs/RELEASE-SIGNOFF.md`) with the deferred-plane work.

## Gates (clean-checkout re-run)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ |
| `cargo clippy --workspace --tests` (pedantic) | ✅ |
| `cargo test --workspace --lib` | ✅ 22 suites, no failures |
| admin-ui `dart analyze` (Dart SDK) / `tsc --noEmit` (admin-ui) | ✅ |

> Full `cargo test --workspace` exceeds the local link budget (str0m + httpmock pull large
> test-binary trees); it runs in CI. Lib + per-crate integration suites pass locally.

## What now functions end-to-end (phase-18)

- **str0m signaling routing** — unicast to `to_session` / room fan-out on `room_id`
  (the HIGH bug that broke all signaling is fixed).
- **Matrix inbound** — real `/sync` long-poll → spine (no Tuwunel dep).
- **Matrix outbound**, **ATProto inbound** — already functional; unchanged.
- **ATProto outbound** — authenticated PDS write *capability* (bridge), tested against a
  mocked PDS. (Gateway wiring pending — see deferred.)
- **str0m negotiation** — offer→answer round-trip proven on str0m 0.21 (spike).
- **admin-ui token hardening** — expiry decode, auto-logout, 401-clears-token.
- **Dart SDK** — compiles; stable `FrfTransport` shim + working `FrfCrdt` surface.
- **Federation config safety** — `FEDERATION_CHANNEL_ID` now required when enabled.

## Re-affirmed deferred (documented, gated off — NOT release-blocking)

- **Full str0m sovereign SFU** — live UDP/ICE/DTLS media loop + RTP fan-out. `SFU_MODE`
  defaults to `hosted`; sovereign warns and moves no media.
- **LiveKit cross-node inbound relay** — inbound is in-process only.
- **Full admin-ui OIDC login** — blocked on an IdP (Kratos/Hydra) + a flint-gate login
  endpoint; the token gate is the hardened interim.
- **ATProto outbound gateway wiring** — the bridge write capability exists and is tested,
  but `main.rs` builds the bridge inbound-only; enabling outbound needs PDS identity config.
- **Dart async-transport bindings** — blocked on an upstream `uniffi-bindgen-dart` fix
  (0.1.3 is latest); the shim surfaces this honestly.

See `docs/SECURITY.md` §6 for the security posture of each deferred item.

## Sign-off

Phase-18 completes the scoped deferred-plane work with no new CRITICAL/HIGH and all gates
green. The hosted deployment shape signed off in phase-17 is unchanged; the sovereign SFU,
LiveKit cross-node relay, full OIDC login, and ATProto outbound wiring remain deferred and
must not be advertised as shipped until they function end-to-end.
