# p17-c008 — Finish Dart bindings

## Status: PROPOSED

## Why

sdks/dart exposes no generated API (only a pending marker + .gitkeep). Run build_dart.sh with uniffi-bindgen-dart, commit generated lib/src/rust/*.dart, and fix GENERATED.md drift (still names flutter_rust_bridge).

Phase: phase-17-plane-completion-and-release-audit · Goal: G3.5
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
