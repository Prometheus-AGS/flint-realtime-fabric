# p18-c007 — Matrix inbound sync loop

## Status: PROPOSED

## Why

Matrix inbound is stream::empty() (stub). Implement a real /sync long-poll loop over reqwest (no Tuwunel crate needed) that projects room events into the bridge's inbound stream.

Phase: phase-18-media-federation-and-auth-flow · Goal: G3
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
