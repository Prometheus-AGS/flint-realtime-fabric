# p16-c026 — LICENSE + release artifacts + ADR

## Status: DONE

## Goal
G5.5 (#54/#57/#58/#59) — phase-16-production-hardening

## Problem
No LICENSE file (MIT is declared in Cargo.toml), no CONTRIBUTING/CHANGELOG/SECURITY.md, no proto-derived API reference, and the UniFFI/frb/Connect/tonic version decision is captured in no ADR.

## Solution
1. Add LICENSE (MIT), CONTRIBUTING, CHANGELOG, SECURITY.md
2. Generate a proto-derived API reference
3. Write an ADR for the UniFFI/frb/Connect/tonic version decision

## Files Changed
- `.`
- `docs/decisions/`

## Risk
LOW.
