# p19-c002 — wire ATProto outbound into the gateway

## Why

The ATProto outbound write *capability* (`AtProtoBridge::with_writer` + `PdsClient`) is
built and tested (p18-c008, spec `atproto-outbound`), but the gateway never uses it:
`build_federation_bridges` in `main.rs:245-253` builds `AtProtoBridge::new(...)`
**inbound-only** and logs "OUTBOUND send is unsupported for v1." So the tested write path
is unreachable in a running gateway (technical debt flagged in the phase-18 reflection, G3).

## What Changes

1. **Config (`config.rs`):** add optional PDS-writer fields — `atproto_pds_url`,
   `atproto_pds_identifier`, `atproto_pds_app_password` (secret), and optional
   `atproto_write_collection`. Load them in `from_env` from `ATPROTO_PDS_URL`,
   `ATPROTO_PDS_IDENTIFIER`, `ATPROTO_PDS_APP_PASSWORD`, `ATPROTO_WRITE_COLLECTION`.
2. **Validation (`config.rs::validate`):** **all-or-none** guard (matching the CDC /
   federation pattern) — if any one PDS-writer var is set, all three of URL / identifier /
   app-password MUST be set, else boot fails fast (a half-configured writer would silently
   never authenticate).
3. **Wiring (`main.rs`):** in `build_federation_bridges`, when the three PDS-writer fields
   are present, build the ATProto bridge via `AtProtoBridge::with_writer(PdsConfig, write_collection)`
   instead of inbound-only; update the log to state OUTBOUND is now enabled. When absent,
   keep the existing inbound-only path unchanged.
4. **Docs (`ENVIRONMENT.md`):** document the 4 new env vars under the federation section;
   mark `ATPROTO_PDS_APP_PASSWORD` as a **secret** (secret manager, never committed).
5. **Test:** a config unit test — writer fields present ⇒ validate passes and the bridge
   would be outbound-capable; one-of-three set ⇒ validate fails naming the missing var.

## Security

`ATPROTO_PDS_APP_PASSWORD` is an app-password credential. It is read from the environment
only, never logged (the wiring log names the PDS URL, not the password) and never committed
(CLAUDE.md: "Never log JWT payloads … or tenant identifiers"; secrets rule).

## Non-goals

- The bridge write path itself (done in p18-c008) — this only wires it.
- Making federation on-by-default — outbound is still gated behind `FEDERATION_ENABLED`
  plus the PDS-writer vars being present.

## Impact

- Affected: `crates/frf-gateway/src/config.rs`, `crates/frf-gateway/src/main.rs`,
  `docs/ENVIRONMENT.md`.
- With the PDS vars set (and federation enabled), the gateway's ATProto bridge writes
  federated events to the PDS end-to-end; the p18-c008 write path is now reachable.
