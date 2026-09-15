# c007 acceptance A — crash-safe checkpoint and restart replay

Captured: `2026-09-15T17:28:40Z`

Status: **PASS**

## Executed topology

`scripts/run-broker-replay-integration.sh` created the isolated Compose project
`frf-broker-replay-20260915171812-5090`, used an ephemeral host port and owned
volume, and removed both after the run. It executed exact ignored test names;
ordinary test discovery could not turn a missing scenario into a skip.

The running server was the immutable, SDK-compatible image
`iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5`
with local image ID
`sha256:5ed1de369f7d563d5da04093e2d04beb980d2ce965efaaa683bf9a3c72aa5c5f`.

## Crash before checkpoint

The core scenario published two stable events, polled the first with
auto-commit disabled, verified the named consumer had no stored position, then
dropped the receiver before acknowledgement. A new broker client subscribed at
the unacknowledged position and received the same event ID at the same broker
offset. This demonstrates at-least-once replay across the application crash
window.

It then proved the explicit cursor rule:

- subscribing at the delivered broker offset replays that event because the
  Iggy explicit-offset strategy is inclusive;
- subscribing at `offset.next()` yields the following event;
- acknowledging that following event stores its broker offset for the named
  consumer.

Core marker: `BROKER_REPLAY_CORE_PASS`. Core log SHA-256:
`99ca0646436b121d79968050920ca0c45eb10cab7fe33134aa746306a21a1701`.

## Restart after checkpoint

The seed process published three events, consumed the first two, and explicitly
stored the second broker offset (`1`). The runner restarted the Iggy container,
waited for an authenticated protocol login and host TCP readiness, then launched
a new verification process. That process read durable checkpoint `1`, resumed
at `2`, received the third event at broker offset `2`, and appended the next
event at offset `3`.

Retrying the original pre-restart event ID created no new broker position, which
proves the duplicate rule survives the same server restart.

Markers:

- `BROKER_RESTART_SEED_PASS offset=1`
- `BROKER_RESTART_VERIFY_CHECKPOINT offset=1`
- `BROKER_RESTART_VERIFY_PASS checkpoint=1 replayed=2 appended=3`

Seed log SHA-256:
`eeb420223ec00c898b7fe6ffbc387f41e4bdf40aba110a02fad235a0158146db`.
Verify log SHA-256:
`df17c6b809172c8fdbdc5e14e2b12b5acefbdf99f7e4c33d4ad44fd0006475eb`.

All scenarios passed locally. No CI test run supplied evidence.
