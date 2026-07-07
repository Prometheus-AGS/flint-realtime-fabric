# p16-c003 — Enforce clippy::unwrap_used workspace-wide

## Status: DONE

## Goal
G1.3 (H12) — phase-16-production-hardening

## Problem
The documented `clippy::unwrap_used` hard gate is enforced nowhere; `pedantic` does not include the restriction group. ~50 real library unwrap/expect (frf-crdt 36, frf-store-redb 16) can panic the gateway and pass CI green.

## Solution
1. Add `unwrap_used = "deny"` to workspace [lints.clippy]
2. Fix the ~50 library sites, concentrated in frf-crdt and frf-store-redb
3. #[allow(clippy::unwrap_used)] with a // justification: comment only for truly-unreachable cases
4. Wire the lint into CI (ci.yml) and dagger clippy stage so both enforce it

## Files Changed
- `Cargo.toml`
- `crates/frf-crdt/`
- `crates/frf-store-redb/`
- `.github/workflows/ci.yml`
- `dagger/codegen.ts`

## Risk
MEDIUM — touches many sites; each fix is mechanical (Result propagation / expect-with-justify).
