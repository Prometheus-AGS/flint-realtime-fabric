# ADR-003: FFI & Codegen Toolchain Versions

## Status

Accepted — 2026-07-06

## Context

CLAUDE.md flagged the UniFFI / flutter_rust_bridge / Connect / tonic versions as an
open, load-bearing decision to make before committing the FFI/codegen approach. The SDK
strategy is: only Rust is hand-written; every other SDK is generated from the frozen
`proto-v1` (Go, browser-TS, C#) or FFI-bound over the Rust core (Swift, Kotlin, Dart).
This ADR pins the toolchain versions and — importantly — resolves a framework conflict
surfaced during implementation (p16-c014).

## Decision

### gRPC / proto (Rust + generated SDKs)

- **tonic `0.14`** + **tonic-prost `0.14`** — gRPC server/client in `frf-gateway`,
  `frf-proto`, `frf-sdk-rust`.
- **tonic-web `0.14`** — gRPC-web translation so browsers reach the gateway over
  HTTP/1.1 (Connect-Web / gRPC-web). Enabled with `accept_http1(true)` +
  `GrpcWebLayer`.
- **`@connectrpc/connect` `^1.7`** + **`@connectrpc/connect-web` `^1.6`** — the
  browser-TS SDK transport (`sdks/ts`). `connect` is the core package where the
  `Interceptor` type lives (auth header injection).
- **C# / Go** — generated from the source-of-truth proto via `Grpc.Tools` (C#) and
  `connectrpc.com/connect` (Go), never a committed proto copy.

### FFI (Swift, Kotlin, Dart)

- **UniFFI `0.31.2`** is the single FFI framework for the mobile SDKs. Swift and Kotlin
  bindings are generated from `frf-ffi` via the workspace `uniffi-bindgen` crate. The
  async transport surface uses `uniffi`'s `tokio` feature
  (`async_runtime = "tokio"`).
- **Dart uses `uniffi-bindgen-dart` (`0.1.x`), NOT flutter_rust_bridge.**

### Why Dart moves off flutter_rust_bridge (the resolved conflict)

The Dart SDK was originally scaffolded for **flutter_rust_bridge (FRB) 2.11.1**. During
p16-c014 we established that **FRB cannot bridge a UniFFI crate**: FRB's parser panics on
the `#[uniffi::export]` surface (trait objects, edition-2024 syntax), emits only
non-functional stubs, and injects an unwanted `mod frb_generated` into the crate's
`lib.rs` that conflicts with UniFFI. FRB and UniFFI cannot co-own the FFI crate.

Since Swift and Kotlin already use UniFFI, the coherent choice is **one FFI framework
everywhere** — Dart generates from the same UniFFI surface via `uniffi-bindgen-dart`.
This keeps the FFI contract identical across all three mobile platforms and honors the
"business/CRDT/reconnection logic lives in exactly one place" rule (the Rust core).

## Consequences

- **Positive:** a single FFI framework (UniFFI) across Swift/Kotlin/Dart; the mobile
  SDKs share one generated surface over the Rust core; the gRPC-web version aligns with
  tonic so browser + native transports use one contract.
- **Negative / follow-up:** the Dart transport bindings are not generated yet — that
  requires running `sdks/dart/build_dart.sh` in an environment with `uniffi-bindgen-dart`
  installed. Until then the Dart package resolves (`pub get` succeeds) but exposes no
  generated API. Tracked as a follow-up.
- These versions shift over time. Confirm current releases and language coverage before a
  major toolchain bump; a bump to any of them is a deliberate change, not a silent
  `cargo update` / `pnpm up`.

## Related

- ADR-001 (CRDT engine — Loro), whose encoding crosses these same FFI bindings.
- p16-c011 (`frf-sdk-rust`), p16-c012 (reconnection), p16-c014 (FFI transport),
  p16-c013 (service binding).
