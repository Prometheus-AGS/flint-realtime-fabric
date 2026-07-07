# p17-c004 — EntityService gateway server

## Status: PROPOSED

## Why

EntityService is proto-only (no server). Add a pure-Rust tonic impl of GetEntity/WatchEntity backed by a store and register it in spawn_grpc_server, so the advertised Entity plane actually functions.

Phase: phase-17-plane-completion-and-release-audit · Goal: G3.1
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
