# Tasks — p2-c004 sdk-go

- [ ] **T1** Create `sdks/go/go.mod`
  - Module: `github.com/prometheusags/frf/sdks/go`
  - Go version: 1.23
  - Requires: `connectrpc.com/connect@v1.20.0`, `google.golang.org/protobuf@v1.36.11`
  - Verification: `go mod tidy` exits 0 from `sdks/go/`

- [ ] **T2** Verify generated stubs exist
  - Prerequisite: p2-c003 must have run `buf generate`
  - Verification: `ls sdks/go/gen/flint/v1/*.pb.go` lists files; `ls sdks/go/gen/flint/v1/*connect*.go` lists files

- [ ] **T3** Create `sdks/go/client/spine_client.go`
  - Define `SpineClient` struct with Connect-backed publish and subscribe methods
  - `New(baseURL string, opts ...connect.ClientOption) *SpineClient`
  - `Publish(ctx context.Context, envelope *flintv1.EventEnvelope) (*flintv1.PublishResponse, error)`
  - `Subscribe(ctx context.Context, req *flintv1.SubscribeRequest) (*connect.ServerStreamForClient[flintv1.EventEnvelope], error)`
  - Use `http.DefaultClient`; caller configures transport via opts
  - Verification: file compiles (`go build ./...` from `sdks/go/`)

- [ ] **T4** Create `sdks/go/client/smoke_test.go`
  - Build tag `//go:build integration`
  - Test name: `TestPublishSmoke` — dials `$FRF_GATEWAY_URL`, publishes a minimal envelope, asserts no error
  - Verification: `go build -tags integration ./...` exits 0 (does not require a running gateway)

- [ ] **T5** Run `go vet ./...` from `sdks/go/`
  - Verification: exits 0
