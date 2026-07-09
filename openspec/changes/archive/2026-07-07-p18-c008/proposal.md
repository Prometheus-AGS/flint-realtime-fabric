# p18-c008 — ATProto outbound PDS write

## Status: PROPOSED

## Why

ATProto outbound returns Err(unimplemented). Implement an authenticated PDS write (com.atproto.repo.createRecord) so the outbound direction functions.

Phase: phase-18-media-federation-and-auth-flow · Goal: G3
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
