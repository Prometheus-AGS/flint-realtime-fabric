# p20-c005 — DTLS-connected milestone

## Why

The async session engine (c003) + trickle ICE (c004) drive a session's transport but nothing
yet takes it to **`Connected`** (DTLS handshake complete) — the honest phase-20 milestone.
Two gaps: (1) str0m's DTLS needs a **crypto provider installed**, which the engine never
does, so a real handshake cannot complete; (2) there is no consumer-facing way to await the
connected state. This change closes both and proves the state path.

## What Changes

1. **Crypto provider install (correctness):** `StrOmTransport::new` installs the process
   default crypto provider (`str0m::crypto::from_feature_flags().install_process_default()`,
   once) — without it DTLS cannot key. This is required for any real media, not just tests.
2. **`wait_for_connected(session_id, timeout)`:** a helper that awaits the session's
   `ConnectionState` reaching `Connected` (via the state watch) or times out — the API a
   consumer/integration test uses to know the transport is up.
3. **State-path proof:** an in-process **two-session loopback** connectivity test — two
   `StrOmTransport` sessions on loopback exchange offer/answer + host candidates and drive
   real UDP between them until one reports `Connected`. This exercises str0m's real
   ICE-connectivity + DTLS handshake through the async engine, **no browser**. If loopback
   ICE proves timing-flaky in CI, the test is `#[ignore]`-gated (integration) with the
   reason documented — the honest fallback, not a fake pass.

## Non-goals (phase-21)

- RTP `MediaData` forwarding, per-room fan-out, PLI. Reaching `Connected` ≠ media flowing.
- Enabling `SFU_MODE=sovereign` — it stays gated off (connected, not media-forwarded).

## Impact

- Affected: `crates/frf-media-str0m/src/session.rs`, `SPIKE-FINDINGS.md` (record the
  milestone).
- The engine installs crypto and exposes `wait_for_connected`; the DTLS-connected path is
  proven in-process (or honestly integration-gated). `SFU_MODE=sovereign` stays off.
