# p18-c009 — Dart async-transport shim

## Status: PROPOSED

## Why

uniffi-bindgen-dart 0.1.3 (latest) emits broken async/callback codegen, so FrfFfiClient connect/subscribe don't work. Write a thin hand-written Dart shim over the working sync FFI for connect/subscribe/ack, wired through the package entry point.

Phase: phase-18-media-federation-and-auth-flow · Goal: G4
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
