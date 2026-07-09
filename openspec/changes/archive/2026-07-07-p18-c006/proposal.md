# p18-c006 — str0m Rtc round-trip spike

## Status: PROPOSED

## Why

The str0m crate (0.7, latest 0.21) is declared but never used; the adapter is signaling-only. Bump to 0.21 and prove ONE real Rtc round-trip (SDP accept_offer + ICE + DTLS connect) adapter-only. Deliverable is a proven round-trip or a documented finding — not a production SFU. SFU_MODE=sovereign stays gated off unless media flows.

Phase: phase-18-media-federation-and-auth-flow · Goal: G2
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
