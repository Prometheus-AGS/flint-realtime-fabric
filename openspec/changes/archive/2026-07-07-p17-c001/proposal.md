# p17-c001 — Fix compose.override.yml footgun

## Status: PROPOSED

## Why

compose.override.yml is committed and Docker auto-merges it on a bare 'docker compose up', hardcoding a dev JWT secret + DEV_NO_AUTH + dev-endpoints. Cannot affect the prod image, but is a deploy-base footgun and a committed credential literal. Gitignore it (ship a .example) or add a top-of-file deploy-base guard, and remove the committed secret string.

Phase: phase-17-plane-completion-and-release-audit · Goal: G1
Source: assessment.md gap analysis (independent re-audit, 2026-07-06).

## What changes

See tasks.md. Scope is limited to this change's goal; do not pull in deferred
str0m-WebRTC / federation / admin-ui-login work (future phase-18).
