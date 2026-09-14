# p38-c006 — Remove dead code and classify every unproven guard

## Summary

One orphaned source file compiles into nothing, and four `#[ignore]`d tests have never
executed in the default suite. Three vacuous guards were already found and removed in this
codebase on 2026-09-14; the remaining ignored tests must be classified rather than assumed
to work.

## Evidence

**`crates/frf-gateway/src/ws.rs` is orphaned.** 20 lines exporting `ws_echo`, whose only
occurrence in the workspace is its own definition. `lib.rs` declares twelve modules and `ws`
is not among them; no `mod ws;` exists anywhere. The file never compiles into the binary.

**`fetch_and_cache` is NOT dead** — an earlier sweep flagged it wrongly by excluding the
defining file. It has two in-file callers (`crates/frf-identity-ory/src/jwks.rs:58`, `:72`).
Recorded here so the mistake is not repeated.

**Four `#[ignore]`d tests, none ever executed:**

| Test | Blocker |
|---|---|
| `frf-broker-iggy/tests/publish_subscribe.rs:29`, `:85` | c001 |
| `frf-postgres-cdc/tests/cdc_integration.rs:16` | local Postgres 17 + logical replication |
| `frf-gateway/tests/subscribe_mux.rs:10` | Iggy + Keto + flint-gate |

## Why this is independent

The deletions are independent. The *classification* depends on c001, because whether the two
broker tests are "runnable" is exactly what c001 determines.

## Scope

Delete `ws.rs` unless it is a deliberate stub — in which case say so in the file. Classify
each ignored test as (a) runnable now, and then actually run it, or (b) permanently
unrunnable locally, with the reason written into the `#[ignore]` string rather than left
implicit.

## Non-goals

- Deleting `fetch_and_cache` or anything else flagged by a heuristic sweep without
  confirming zero callers including the defining file.

## Files

`crates/frf-gateway/src/ws.rs`, and the `#[ignore]` attributes in the four test files.
