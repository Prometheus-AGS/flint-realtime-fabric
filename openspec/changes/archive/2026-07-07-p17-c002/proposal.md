# p17-c002 — Enforce JWT_ISSUER in production

## Status: PROPOSED

## Why

JWT_ISSUER is optional today; if unset, iss is unvalidated (only a startup warning). Make it a hard boot-time validation error outside dev, and document the requirement.

Phase: phase-17-plane-completion-and-release-audit · Goal: G1
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
