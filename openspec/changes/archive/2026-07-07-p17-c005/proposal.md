# p17-c005 — AuthzService gateway server

## Status: PROPOSED

## Why

AuthzService is proto-only. Add a pure-Rust tonic impl of Check/WriteRelation/DeleteRelation delegating to the existing KetoAuthzProvider, and register it. Backing logic already exists; this is a thin wiring change.

Phase: phase-17-plane-completion-and-release-audit · Goal: G3.2
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
