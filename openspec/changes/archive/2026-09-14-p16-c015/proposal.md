# p16-c015 — Create frf-cli operator tooling

## Status: DONE

## Goal
G3.5 (H13) — phase-16-production-hardening

## Problem
No first-party operator CLI exists. There is no supported way to seed Keto tuples, manage CDC replication slots, inspect broker offsets, or force checkpoints.

## Solution
1. Create crates/frf-cli (clap)
2. Commands: keto seed, cdc slot manage, broker offsets, checkpoint
3. Surface AuthzProvider::write()

## Files Changed
- `crates/frf-cli/`
- `Cargo.toml`

## Risk
MEDIUM — new binary crate.
