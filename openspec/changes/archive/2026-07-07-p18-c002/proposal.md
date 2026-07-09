# p18-c002 — Guard FEDERATION_CHANNEL_ID when enabled

## Status: PROPOSED

## Why

validate() requires FEDERATION_TENANT_ID when federation is enabled but has no guard for FEDERATION_CHANNEL_ID, so main.rs falls back to a per-boot random ChannelId even in production. Add the missing validate() guard.

Phase: phase-18-media-federation-and-auth-flow · Goal: G3
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
