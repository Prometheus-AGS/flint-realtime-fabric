# Tasks — p2-c003 buf-config

- [ ] **T1** Update buf CLI
  - `brew upgrade buf` (target: 1.71.0)
  - Verification: `buf --version` shows 1.71.0

- [ ] **T2** Install `protoc-gen-connect-go`
  - `go install connectrpc.com/connect/cmd/protoc-gen-connect-go@v1.20.0`
  - Verification: `which protoc-gen-connect-go` exits 0

- [ ] **T3** Create `proto/buf.yaml`
  - Content: version v2, module name `buf.build/prometheusags/frf`, DEFAULT lint rules, FILE breaking rules
  - Verification: `buf config ls-lint-rules` exits 0 from `proto/`

- [ ] **T4** Create `proto/buf.gen.yaml`
  - Content: local plugins for Go (protoc-gen-go, protoc-gen-connect-go), remote plugins for TS (buf.build/bufbuild/es@v2.12.0, buf.build/connectrpc/es@v1.7.0)
  - Output Go → `../sdks/go/gen`, TS → `../sdks/ts/src/gen`
  - Verification: file exists; yaml is valid

- [ ] **T5** Create output directories
  - `mkdir -p sdks/go/gen sdks/ts/src/gen`

- [ ] **T6** Run `buf generate`
  - From `proto/`: `buf generate`
  - Verification: `sdks/go/gen/flint/v1/*.pb.go` files exist; `sdks/ts/src/gen/flint/v1/*.ts` files exist

- [ ] **T7** Run `buf lint`
  - `buf lint` from `proto/`
  - Verification: exits 0 (0 lint errors)

- [ ] **T8** Check breaking changes (best-effort)
  - `buf breaking --against '.git#tag=proto-v1'` from `proto/`
  - If tag does not exist: document that `proto-v1` tag should be created; `git tag proto-v1` and re-run
  - Verification: exits 0 or tag is created and exits 0
