# p17-c007 — CLI: CDC slot mgmt + broker-offset inspect

## Status: PROPOSED

## Why

The CLI advertises 'manage CDC slots' and 'inspect broker offsets' but cdc status is inspect-only and there is no broker-offset read (only checkpoint-write). Build the missing commands so the CLI matches its advertised surface.

Phase: phase-17-plane-completion-and-release-audit · Goal: G3
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
