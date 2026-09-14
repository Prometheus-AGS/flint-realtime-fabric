# p38-c005 — Reconcile CLAUDE.md with the ADRs that already settled it

## Summary

CLAUDE.md's "Open Decisions" table lists four decisions as unresolved that ADRs closed months
ago, and its workspace tree omits five crates that exist on disk. Stale architecture docs are
not cosmetic here: they are the demonstrated cause of a two-month misdiagnosis.

## Evidence

| CLAUDE.md | Reality |
|---|---|
| `:102` "CRDT: Loro **or** automerge-rs — **OPEN, decide before Phase 3**" | ADR-001 accepted Loro on 2026-06-19; `Cargo.toml:146` ships `loro 1.13.1` |
| `:274` four decisions "must be resolved before advancing phases" | ADR-001 and ADR-003 settled CRDT and the FFI/codegen toolchain |
| `:256` "Dart / Flutter: flutter_rust_bridge over Rust core" | ADR-003 overrode this — FRB's parser panics on `#[uniffi::export]`; Dart uses `uniffi-bindgen-dart` |
| workspace tree | `frf-did`, `frf-p2p`, `frf-shape-electric`, `frf-wallet`, `uniffi-bindgen` exist on disk but are absent from the tree |

**The cost is documented.** GitHub issue #2 was filed against a `tenant-`/`topic_name(path)`
naming scheme that commit `26e4dfc` had already replaced. The reporter followed
`crates/frf-broker-iggy/src/channel.rs`, whose helpers still described the old scheme and
whose tests still asserted a `tenant-` prefix, although no production code had called them
since that commit. The issue stood, wrong, for two months.

## Why this is independent

Documentation only. No code, no dependency on c001.

## Scope

Every row of "Open Decisions" is either genuinely open, or removed with a pointer to the ADR
that closed it. The workspace tree matches disk. The Dart line reflects ADR-003.

## Non-goals

- Fixing `dagger/codegen.ts` stage 4, which diffs against a `frb_generated.dart` that does
  not exist. Flag it; repairing Dagger is a separate change.

## Files

`CLAUDE.md`; possibly `AGENTS.md` if the same claims appear there.
