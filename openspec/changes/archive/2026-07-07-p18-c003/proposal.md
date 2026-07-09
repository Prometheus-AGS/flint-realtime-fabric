# p18-c003 — Fix str0m sfu_mode wire inconsistency

## Status: PROPOSED

## Why

The gRPC signal mapping forces sfu_mode=Hosted regardless of config, and Default for GatewayConfig uses Sovereign while from_env defaults Hosted. Align these so the reported mode matches the configured mode.

Phase: phase-18-media-federation-and-auth-flow · Goal: G2
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
