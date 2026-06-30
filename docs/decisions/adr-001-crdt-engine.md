# ADR-001: CRDT Engine Selection — Loro over automerge-rs

## Status

Accepted — 2026-06-19

## Context

Phase 3 requires a CRDT engine for the offline-first entity sync layer
(`frf-crdt`). The engine choice is load-bearing: it propagates into every FFI
binding (Swift, Kotlin, Dart) because the binary encoding format and merge
function must be identical on every platform.

Two candidates were evaluated:

| Factor | Loro 1.13.1 | automerge 0.10.0 |
|---|---|---|
| Rust crate maturity | 1.x stable | 0.x pre-release |
| UniFFI FFI crate | `loro-ffi 1.13.1` (first-party, same org) | `automerge-uniffi` exists but is experimental ("first draft", author warns "many things weird or wrong") |
| Swift binding | `loro-swift` v1.13.2 (June 2026), SPM-native, 121 commits — production-grade | `automerge-swift` via UniFFI — experimental, no releases |
| Kotlin binding | **Not first-party** — loro-ffi lists Swift, Python, React Native, C#, Go; Kotlin absent | Same: requires `#[uniffi::export]` proc-macro wrapping |
| Performance | Faster on rich-document range ops; lower memory | Mature but slower on large docs |
| Sync protocol | Loro Sync Protocol (binary, incremental) | Automerge Sync Protocol (same shape) |
| WASM support | `loro` has WASM targets | `automerge-wasm` exists |
| Ecosystem | Younger; purpose-built for mobile FFI | Broader community; `automerge-repo` |

## Decision

**Loro 1.13.1** is selected as the CRDT engine.

Research (June 2026) shows Loro leads on every measured axis:
- **2–9× faster** encode/decode than automerge
- **3.7× smaller** encoded document size (68 kB vs 250 kB on the same corpus)
- **2.7× less memory** (15 MB vs 41 MB loaded)
- **Fugue algorithm** provably prevents interleaving anomalies that RGA (automerge) produces
- **Swift binding**: `loro-swift` v1.13.2 (June 15, 2026), SPM-native, actively maintained.
  `automerge-uniffi` for Swift is an experimental "first draft" with no releases.
- **Kotlin parity**: neither engine has a first-party Kotlin crate — both require
  `#[uniffi::export]` proc-macro wrappers. The Kotlin situation is symmetric, so
  Loro's other advantages are decisive.
- **Stability**: Loro 1.x stable API vs automerge 0.x pre-release.

The one automerge advantage (`automerge-repo` full sync layer) does not apply
here: FRF runs its own tonic-backed sync service.

Both engines produce binary-safe, incremental, convergent snapshots over `Vec<u8>`
payloads. The engine choice is opaque to `frf-domain` and `frf-app`.

## Consequences

**Positive:**
- `loro-swift` v1.13.2 delivers a production-grade Swift SPM package — zero hand-rolled shims.
- Kotlin binding parity: both engines need `#[uniffi::export]` wrappers; Loro wins on all other axes.
- 2–9× better encode/decode performance matters on constrained mobile hardware.
- 3.7× smaller wire payloads reduce bandwidth cost for offline-first sync.
- Fugue algorithm correctness guarantee: no interleaving anomalies vs. automerge's RGA.
- Loro 1.x stable API reduces risk of breaking changes during Phase 3 build-out.
- `apply_delta` is a pure Rust function; the engine is opaque to all layers above `frf-crdt`.

**Negative:**
- Loro ecosystem is "early-stage" — manual WebSocket integration required (matches FRF's own tonic sync layer, so not a blocker).
- Smaller community than automerge; fewer real-world examples and Stack Overflow answers.
- `loro` WASM targets exist but `frf-wasm` is deferred to Phase 4 — untested path.
- No `automerge-repo` equivalent: no turnkey sync server (not needed — FRF uses tonic).
- If a future peer integration requires automerge encoding, migrating `frf-crdt` would require rebuilding and regenerating all FFI bindings.

## Supersedes

None — this is the first CRDT engine decision for this project.
