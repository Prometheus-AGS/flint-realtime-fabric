# p18-c005 — Harden admin-ui token flow

## Status: PROPOSED

## Why

flint-gate has no interactive login endpoint and Kratos is not deployed, so full OIDC is deferred. Harden the existing paste-a-JWT model honestly: decode exp, warn/logout on expiry, re-auth on 401, secure-storage guidance, and make the LoginGate copy accurate (token gate, not OIDC).

Phase: phase-18-media-federation-and-auth-flow · Goal: G1
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
