# p22-c002 — PLI / keyframe-request forwarding

## Why

A receiver that loses sync needs a keyframe from the sender. In an SFU the request flows
**receiver → SFU → sender** (opposite direction to media): when a peer's `Rtc` emits
`Event::KeyframeRequest`, the SFU must relay it to the room's other member(s) and call
`request_keyframe` on their `Rtc` so they produce a keyframe. Phase-21 forwarded media; this
adds the reverse keyframe-request path so a late/desynced peer can be served.

## Design

str0m: inbound `Event::KeyframeRequest(KeyframeRequest { mid, rid, kind })` (kind = `Pli`|`Fir`);
send via `Writer::request_keyframe(rid, kind)` (self-guards with `is_request_keyframe_possible`).
Both are `Copy`. To reuse the existing per-session forwarding channel (not add a second one),
the channel payload becomes an enum **`ForwardedFrame { Media(ForwardedMedia),
KeyframeRequest(KeyframeRequest) }`** — the driver's existing `forward_rx` arm handles both.

## What Changes

1. **`room.rs`:** `ForwardedFrame` enum (Media | KeyframeRequest); the forwarding channel is
   `mpsc::Sender<ForwardedFrame>`. `forward` wraps media as `ForwardedFrame::Media`; a new
   `forward_keyframe_request(from, req)` fans a `KeyframeRequest` to co-room peers (same
   topology, opposite direction). Unit-test: a keyframe request routes to the other member(s),
   not the sender.
2. **`driver.rs`:** on `Event::KeyframeRequest`, call `router.forward_keyframe_request`; the
   `forward_rx` arm matches `ForwardedFrame::Media` → `writer.write` and
   `ForwardedFrame::KeyframeRequest` → `writer(mid).request_keyframe(rid, kind)` (guarded).
3. **`session.rs`:** the `forward_tx`/`forward_rx` channel type becomes `ForwardedFrame` (a
   type-only change at the create-session wiring).

## Non-goals (browser-gated / phase follow-on)

- Renegotiation on membership/track change (browser-gated).
- Proving a real late peer *decodes* after a PLI (browser-gated); `SFU_MODE=sovereign` stays off.

## Impact

- Affected: `crates/frf-media-str0m/src/{room.rs,driver.rs,session.rs}`.
- Keyframe requests route receiver→sender through the SFU; unit-tested. No lib
  `unwrap`/`expect`; files ≤500; gate stays off.
