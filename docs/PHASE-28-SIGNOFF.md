# Phase-28 signoff — sovereign SFU decode-harness timing & proof

> Date: 2026-07-09. Phase-28 restored the decode harness's diagnostics, fixed the ICE candidate-IP
> bug they surfaced, and re-ran the proof. **`SFU_MODE=sovereign` stays gated OFF** — the run now
> reaches the media exchange but produces no decoded frame; two precise blockers remain.

## Gate decision: OFF (honest gate held)

`framesDecoded == 0`. Per the phase-16→27 discipline, re-confirmed by the operator twice
(phase-24, phase-27): **the gate does not flip until a real receiver observes `framesDecoded > 0`.**
`crates/frf-gateway/src/main.rs` is untouched (still boots `hosted` with a gate-off warning under
`SFU_MODE=sovereign`).

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p28-c001 | Harness diagnostics restored (drop connect pre-check; 60s timeout; log dump on failure) | none (harness) |
| p28-c002 | ICE candidate-IP fix — `advertise_host: Option<String>` + `resolve_advertised_ip`; valid `127.0.0.1` candidate | none (adapter) |
| p28-c003 | Decode re-run + honest gate decision; `PHASE-28-DECODE-RESULT.md` | **held OFF** |

## Evidence

- Browser: `framesDecoded=0 bytes=0 reason=timeout ice=new localCandidates=2 remoteCandidates=0`.
- Gateway: `sovereign: session negotiated … advertised=candidate:… 127.0.0.1 40000 typ host`
  (the `0.0.0.0` rejection is gone — real progress), then:
  - **B1** `bad remote candidate … <uuid>.local … invalid IP address syntax` (Chrome mDNS host
    candidates rejected by str0m).
  - **B2** `create_session failed … udp bind: Address already in use (os error 98)` →
    `add_remote_candidate failed … unknown session` (fixed single UDP port, per-session bind).

## Carried to next phase

1. **B2 (blocks everything):** one shared demuxing UDP socket in `StrOmTransport`, routing inbound
   datagrams to the owning `Rtc` by remote 5-tuple / ICE ufrag — no per-session re-bind.
2. **B1:** a STUN server-reflexive path (or `.local` mDNS resolution) so a routable browser candidate
   reaches the gateway.
3. Re-run the decode proof; flip only on `framesDecoded > 0`.

## Verification

- `cargo check -p frf-media-str0m -p frf-gateway`, `cargo clippy … -D warnings -W clippy::pedantic`,
  `cargo test -p frf-media-str0m` — green (see the release gate run in the c003 QA record).
- Docs-only + no-flip: `main.rs` unchanged; SECURITY §6 and CHANGELOG record the honest status.
