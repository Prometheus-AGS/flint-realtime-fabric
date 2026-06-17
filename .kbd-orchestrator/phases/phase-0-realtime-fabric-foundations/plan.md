# Plan — phase-0-realtime-fabric-foundations

## Summary

6 changes in strict build-order dependency sequence. Each change is a prerequisite for the next: workspace must exist before crates can be added; domain must compile before ports can depend on it; ports must exist before the proto crate needs them; gateway depends on proto; CI gates all of the above.

Change backend: **OpenSpec** (`openspec/changes/p0-c00*-*/`)

---

## Ordered Change List

| # | Change ID | Title | Blocks | Recommended agent |
|---|-----------|-------|--------|-------------------|
| 1 | `p0-c001-workspace-restructure` | Replace root Cargo.toml with virtual workspace manifest; delete src/main.rs | p0-c002..006 | `rust-build-resolver` + manual verification |
| 2 | `p0-c002-frf-domain` | Create crates/frf-domain with pure domain types (newtypes, EventEnvelope, Channel, Offset, etc.) | p0-c003..006 | `tdd-guide` → `rust-reviewer` |
| 3 | `p0-c003-frf-ports` | Create crates/frf-ports with six async trait seams (LogBroker, AuthzProvider, etc.) | p0-c004..006 | `tdd-guide` → `rust-reviewer` |
| 4 | `p0-c004-frf-proto` | Author proto/flint/v1/*.proto + frf-proto crate with tonic-build codegen; tag proto-v1 | p0-c005..006 | `rust-reviewer` (manual proto authoring) |
| 5 | `p0-c005-frf-gateway-stub` | Create crates/frf-gateway: /healthz, tonic stub, WS echo | p0-c006 | `tdd-guide` → `rust-reviewer` |
| 6 | `p0-c006-dagger-ci` | Create dagger/ CI pipelines: fmt, clippy::pedantic, test, MSRV | none | `devops-engineer` |

---

## Phase 0 Exit Criteria (from RFC-FRF-002)

- [ ] `cargo check --workspace` exits 0 on all member crates
- [ ] `proto-v1` git tag exists
- [ ] `GET /healthz` returns 200 from frf-gateway
- [ ] Dagger CI is green (fmt, clippy::pedantic, test, MSRV)

**Halt for operator approval after all four criteria are met.**

---

## Ordering Rationale

The dependency rule is compiler-enforced: frf-domain cannot import frf-ports, and frf-ports cannot import adapters. This means the build order is topologically determined by the dependency graph. Attempting to build frf-ports before frf-domain fails to compile; attempting to build frf-gateway before frf-proto produces missing-type errors. The sequence above is the only valid order.

CI (p0-c006) is last because it needs all crates present to run its gates meaningfully; however, the `rust-toolchain.toml` created in p0-c006/T1 can be committed early (no blocking dependency).

---

## Risk Notes (from Assessment)

- **Iggy fork availability:** `iggy = { git = "https://github.com/GQAdonis/iggy", branch = "master" }` — network availability required at `cargo fetch` time. Pin to a commit SHA in workspace deps if the branch HEAD becomes unstable.
- **tonic-build + prost versions:** Confirm compatible versions before p0-c004. `tonic` and `prost` must be version-aligned; mismatch produces codegen errors.
- **async-in-trait MSRV:** If Rust stable does not yet stabilize RPITIT for the pinned MSRV, `async-trait` (proc-macro crate) is required in frf-ports. Confirm before p0-c003.
- **CRDT engine (OPEN DECISION):** Loro vs automerge-rs. This decision is NOT required for Phase 0. `CrdtStore` port trait is sufficient. Decide before Phase 3 when the adapter crate is authored.

---

## OpenSpec Change Locations

```
openspec/changes/
├── p0-c001-workspace-restructure/
│   ├── proposal.md
│   └── tasks.md
├── p0-c002-frf-domain/
│   ├── proposal.md
│   └── tasks.md
├── p0-c003-frf-ports/
│   ├── proposal.md
│   └── tasks.md
├── p0-c004-frf-proto/
│   ├── proposal.md
│   └── tasks.md
├── p0-c005-frf-gateway-stub/
│   ├── proposal.md
│   └── tasks.md
└── p0-c006-dagger-ci/
    ├── proposal.md
    └── tasks.md
```
