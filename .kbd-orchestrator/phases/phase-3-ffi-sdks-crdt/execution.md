# Execution — Phase 3: CRDT Core + Offline Persistence + FFI SDK Tier

> RFC-FRF-002 · Prometheus AGS
> Started: 2026-06-19
> Backend: openspec

---

## Backend Selection

**OpenSpec** — `openspec/` directory exists at project root; `project.json` has
`"change_backend": "openspec"`. All 13 changes exist as OpenSpec proposals with
task lists.

## Dispatch Contract

| Field | Value |
|---|---|
| Phase | `phase-3-ffi-sdks-crdt` |
| Total changes | 13 |
| First change | `p3-c001-crdt-adr` |
| QA gate | artifact-refiner per change (skip for doc-only changes: p3-c001) |
| Archive target | `openspec/changes/archive/` via `opsx:archive` |

## Execution Order

Changes execute in strict dependency order. After `p3-c002`, three chains
may proceed in parallel when running multi-agent:

```
p3-c001 → p3-c002 → p3-c003 ─┐
                    p3-c004 ─┤→ p3-c006 → p3-c007 → p3-c008 → p3-c009 ─┐
                    p3-c005 ─┘                                 p3-c010 ─┤→ p3-c012 → p3-c013
                                                               p3-c011 ─┘
```

## QA Gate Policy

- `p3-c001` (ADR + config only): **skip QA** — documentation change
- All other changes: run `cargo check --workspace` + `cargo clippy` as minimum gate
- `p3-c003`, `p3-c004`, `p3-c006`: additionally run `cargo test -p <crate>`
- `p3-c007`, `p3-c008`: additionally run `cargo build -p <crate>`
