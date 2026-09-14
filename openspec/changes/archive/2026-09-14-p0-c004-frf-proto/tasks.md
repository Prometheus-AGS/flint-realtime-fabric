# Tasks — p0-c004 frf-proto (Contract Freeze)

**Semver note:** The git tag `proto-v1` is the contract freeze boundary. No edits to `proto/flint/v1/` after tagging. frf-proto crate version locks at 1.0.0 at that point.

- [x] **T1** Create `proto/flint/v1/envelope.proto`
  - File: `proto/flint/v1/envelope.proto`
  - Messages: `EventEnvelope`, `Channel`, `Offset`, `Cursor`
  - `syntax = "proto3"`, `package flint.v1`, `option go_package = "github.com/prometheusags/frf/proto/flint/v1"`
  - Verification: `protoc --descriptor_set_out=/dev/null proto/flint/v1/envelope.proto` exits 0

- [x] **T2** Create remaining proto files
  - Files: `proto/flint/v1/entity.proto`, `proto/flint/v1/agent.proto`, `proto/flint/v1/signal.proto`, `proto/flint/v1/sync.proto`, `proto/flint/v1/authz.proto`
  - Each mirrors the domain type structure from RFC-FRF-002 §03
  - No circular imports; common types (Timestamp, UUID) use `google/protobuf/timestamp.proto`
  - Verification: `protoc` parses each file without error

- [x] **T3** Create `crates/frf-proto/Cargo.toml`
  - File: `crates/frf-proto/Cargo.toml`
  - `[build-dependencies]`: `prost-build`
  - `[dependencies]`: `prost = { workspace = true }`, `tonic = { workspace = true }`, `frf-domain = { path = "../frf-domain" }`
  - Add to `[workspace.members]` in root `Cargo.toml`
  - Verification: `cargo check -p frf-proto` exits 0

- [x] **T4** Create `crates/frf-proto/build.rs`
  - File: `crates/frf-proto/build.rs`
  - Uses `prost_build::compile_protos` on all six `.proto` files with absolute path resolution via `CARGO_MANIFEST_DIR`
  - Verification: `cargo build -p frf-proto` exits 0; generated Rust in `$OUT_DIR`

- [x] **T5** Create `crates/frf-proto/src/lib.rs`
  - File: `crates/frf-proto/src/lib.rs`
  - `include!(concat!(env!("OUT_DIR"), "/flint.v1.rs"))` inside nested module
  - `#![allow(warnings)]` / `#![allow(clippy::all)]` for generated code only
  - Verification: `cargo build -p frf-proto` exits 0; types accessible as `frf_proto::fv1::EventEnvelope`

- [x] **T6** Tag `proto-v1`
  - Command: `git tag proto-v1`
  - Verification: `git tag | grep proto-v1` returns `proto-v1` ✓
