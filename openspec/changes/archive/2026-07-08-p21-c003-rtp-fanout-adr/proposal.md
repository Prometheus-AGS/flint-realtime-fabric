# p21-c003 — ADR for the sovereign SFU RTP fan-out

## Why

RTP forwarding (c004) is blocked on an architecture decision: the sovereign engine runs one
**isolated driver task per session**, so forwarding A→B is **cross-task message passing**, not
a local write. Per the phase-18/19/20 discipline (an ADR for the load-bearing unknown before
code — cf. ADR-004/005), this must be decided first. This change records ADR-006.

## What Changes

Documentation only.

1. **`docs/decisions/adr-006-rtp-fanout.md`** (new, Proposed) — states the cross-task
   constraint; presents **Option A (central room registry in `StrOmTransport` + per-session
   forwarding `mpsc`)** vs. B (direct peer channels) vs. C (shared-`Rtc` actor); **recommends
   A**; sketches `ForwardedMedia` + the `MediaData`→`writer(mid).write` mapping; documents the
   `pt`-negotiation caveat and bounded-channel back-pressure; scopes c004 to **1-to-1**
   forwarding (N-peer + PLI = phase-22); keeps `SFU_MODE=sovereign` gated off.

## Non-goals

- Implementing forwarding (c004) or N-peer/PLI (phase-22).
- Enabling `SFU_MODE=sovereign`.

## Impact

- Affected: `docs/decisions/adr-006-rtp-fanout.md` (new).
- The RTP fan-out architecture is decided; c004 implements Option A for 1-to-1 forwarding.
