# p18-c001 — Fix str0m signaling routing bug

## Status: PROPOSED

## Why

send_signal keys delivery on (tenant, from_session) — the SENDER's own session — and never reads to_session or room_id, so a signal from peer A is delivered back to A's own channel, never to peer B. Even plain signaling is broken; a same-session test masks it. Route unicast on to_session and add room_id fan-out; fix the test.

Phase: phase-18-media-federation-and-auth-flow · Goal: G2
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
