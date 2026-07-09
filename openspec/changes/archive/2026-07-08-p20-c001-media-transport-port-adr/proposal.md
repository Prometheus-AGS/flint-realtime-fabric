# p20-c001 — ADR for the sovereign SFU media-plane port

## Why

Phase-20 builds the sovereign SFU media plane, but the assessment found the load-bearing
blocker: **`MediaSignaler` is signaling-only** and `StrOmSignaler` has no `Rtc`/UDP/RTP, so
the media engine has no home in the ports layer. Per the phase-18/19 discipline (an ADR for
the load-bearing decision before code — cf. ADR-004), the port architecture must be decided
before any engine change. This change records that decision.

## What Changes

Documentation only.

1. **`docs/decisions/adr-005-media-transport-port.md`** (new, Proposed) — states the
   constraint; presents **Option A (new `MediaTransport` port)** vs. B (extend
   `MediaSignaler`) vs. C (internal engine); **recommends A**; sketches the port surface
   (session create/answer, ICE candidate in/out, connection-state; **RTP → phase-21**);
   notes str0m implements two distinct ports (one-port-per-adapter) and that
   `SFU_MODE=sovereign` stays gated off.

## Non-goals

- Defining the trait (c002) or any implementation (c003+).
- Enabling `SFU_MODE=sovereign`.

## Impact

- Affected: `docs/decisions/adr-005-media-transport-port.md` (new).
- The port architecture is decided and reviewable; c002 implements the trait against it.
