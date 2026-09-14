# Tasks — p16-c010

- [x] Gate SFU_MODE=sovereign off by default with explicit unimplemented log
- [~] Fix LiveKit adapter to listen to the LiveKit server for inbound signals
- [x] Cross-node test or documented limitation
- [x] Document LiveKit-hosted as the v1 media path; str0m deferred

## Decision context

Per operator decision, str0m sovereign SFU is DEFERRED. Investigation of the LiveKit
adapter clarified the actual cross-node gap (audit H4):

| Direction | LiveKit adapter status |
|-----------|------------------------|
| Outbound `send_signal` | **Cross-node — real.** Publishes to the LiveKit server via `send_data`; fans out to all room participants across nodes. |
| Inbound `subscribe_signals` | **In-process only.** Serves each session from a local broadcast channel of *this* process's outbound signals; does not subscribe to the LiveKit server data channel. |

## Task 1 — str0m gated with explicit unimplemented log

`build_media_signaler` now emits a loud `warn!` when `SFU_MODE=sovereign` is selected:
str0m does no real WebRTC, so no media flows — use `SFU_MODE=hosted`. `SFU_MODE` already
defaulted to `hosted` in config; additionally changed `compose.yml` and `compose.ci.yml`
from `sovereign` → `hosted` so the shipped stacks point at the working path.

## Task 2 — LiveKit inbound listener: DEFERRED (marked `[~]`), documented instead

Full cross-node **inbound** relay requires the LiveKit realtime SDK (a WebRTC
data-channel client) to listen for server-originated data events and feed them into
`subscribe_signals`. That is a substantial adapter addition (heavy new dependency) and is
out of scope for v1 hardening — so task 2 is honestly marked not-done (`[~]`) rather than
faked. Task 3 explicitly permits "documented limitation" as the alternative, which is
what was delivered.

## Task 3 — documented limitation (the honest option)

The adapter doc (`LiveKitSignaling`) was rewritten to state directionality precisely:
outbound is fully cross-node; inbound is in-process only; full cross-node ingress is a
KNOWN LIMITATION deferred to a future phase. This replaces the vague prior note with an
accurate one — no false impression that cross-node inbound works.

## Task 4 — documentation

`.env.example` gained a Media/SFU section: `SFU_MODE=hosted` is the supported v1 path
(LiveKit); `sovereign` (str0m) is deferred and flows no media. LiveKit credential env
vars listed.

## Verification

- `cargo clippy -p frf-gateway -p frf-media-livekit --lib --bins` (default + dev-endpoints) → exit 0
- `cargo test -p frf-gateway --lib` → 8/8 · `cargo test -p frf-media-livekit` → pass
- `cargo fmt --check` → clean

## Follow-up

- LiveKit cross-node inbound listener (realtime SDK) — a future media phase, alongside
  the deferred str0m SFU.
- `SFU_MODE` + LiveKit credentials in the full env reference (c022); limitations noted in
  security model / README (c024/c025).
