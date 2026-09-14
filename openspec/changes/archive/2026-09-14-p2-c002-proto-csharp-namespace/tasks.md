# Tasks — p2-c002 proto-csharp-namespace

- [ ] **T1** Add `csharp_namespace` to `proto/flint/v1/envelope.proto`
  - Insert `option csharp_namespace = "PrometheusAgs.Frf.Flint.V1";` after `option go_package = ...;`
  - Verification: file contains the option line

- [ ] **T2** Add `csharp_namespace` to `proto/flint/v1/entity.proto`
  - Same as T1
  - Verification: file contains the option line

- [ ] **T3** Add `csharp_namespace` to `proto/flint/v1/agent.proto`
  - Same as T1
  - Verification: file contains the option line

- [ ] **T4** Add `csharp_namespace` to `proto/flint/v1/signal.proto`
  - Same as T1
  - Verification: file contains the option line

- [ ] **T5** Add `csharp_namespace` to `proto/flint/v1/sync.proto`
  - Same as T1
  - Verification: file contains the option line

- [ ] **T6** Add `csharp_namespace` to `proto/flint/v1/authz.proto`
  - Same as T1
  - Verification: file contains the option line

- [ ] **T7** Verify all 6 files
  - `grep -c csharp_namespace proto/flint/v1/*.proto` → each file shows 1
  - `cargo build -p frf-proto` still exits 0 (proto codegen still works for Rust)
