# p18-c004 — Fix Dart generator doc drift

## Status: PROPOSED

## Why

sdks/dart/pubspec.yaml still credits flutter_rust_bridge 2.11.1 and lists it as a dependency, and GENERATED.md describes a stale connect shape — contradicting ADR-003 and the regenerated frf.dart. Correct the docs to match reality (uniffi-bindgen-dart).

Phase: phase-18-media-federation-and-auth-flow · Goal: G4
Source: assessment.md (code-grounded audit, 2026-07-07).

## What changes

See tasks.md. Scope is limited to this change's goal; do NOT pull in deferred
phase-19 work (full str0m SFU, LiveKit cross-node inbound, full OIDC login).
