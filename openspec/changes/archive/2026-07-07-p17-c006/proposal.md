# p17-c006 — SDK/FFI service + resilience parity

## Status: PROPOSED

## Why

frf-sdk-rust binds only SpineService (no Sync/Agent/Signal wrappers the TS/Go/C# SDKs have); the FFI/mobile path has no reconnection and omits ack. Bind the remaining services in frf-sdk-rust (incl. new Entity/Authz), and add resilient-subscribe + ack to frf-ffi for mobile reconnect/replay parity.

Phase: phase-17-plane-completion-and-release-audit · Goal: G3
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
