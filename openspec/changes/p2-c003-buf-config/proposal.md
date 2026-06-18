# p2-c003 — buf config + codegen plugins + generate Go/TS SDKs

## Phase
phase-2-generated-sdks

## Depends on
p2-c002 (csharp_namespace must be in proto files before buf lint)

## Affected directories
- `proto/` (new buf.yaml, buf.gen.yaml)
- `sdks/go/gen/` (generated — created by buf generate)
- `sdks/ts/src/gen/` (generated — created by buf generate)

## What this change does

1. **Update buf CLI**: `brew upgrade buf` (1.65.0 → 1.71.0)
2. **Install `protoc-gen-connect-go`**: `go install connectrpc.com/connect/cmd/protoc-gen-connect-go@v1.20.0`
3. **Create `proto/buf.yaml`**: Workspace config, lint rules, breaking detection against `proto-v1` tag
4. **Create `proto/buf.gen.yaml`**: Plugin config for Go (protoc-gen-go + protoc-gen-connect-go) and TypeScript (buf remote plugins for `@bufbuild/protoc-gen-es` + `@connectrpc/protoc-gen-connect-es`)
5. **Run `buf generate`** from `proto/` — produces generated files in `sdks/go/gen/` and `sdks/ts/src/gen/`
6. **Run `buf lint`** — confirms proto contract is lint-clean
7. **Run `buf breaking --against '.git#tag=proto-v1'`** — confirms no breaking changes

## buf.yaml content

```yaml
version: v2
modules:
  - path: .
    name: buf.build/prometheusags/frf
lint:
  use:
    - DEFAULT
breaking:
  use:
    - FILE
```

## buf.gen.yaml content

```yaml
version: v2
plugins:
  - local: protoc-gen-go
    out: ../sdks/go/gen
    opt:
      - paths=source_relative
  - local: protoc-gen-connect-go
    out: ../sdks/go/gen
    opt:
      - paths=source_relative
  - remote: buf.build/bufbuild/es:v2.12.0
    out: ../sdks/ts/src/gen
    opt:
      - target=ts
  - remote: buf.build/connectrpc/es:v1.7.0
    out: ../sdks/ts/src/gen
    opt:
      - target=ts
```

## Note on C# generation

buf does not have a first-party C# Connect plugin. C# SDK generation
(p2-c006) uses `Grpc.Tools` MSBuild integration with local proto copies —
not `buf generate`. The `buf.gen.yaml` covers Go and TS only.

## Exit criteria

- `buf lint` from `proto/` exits 0
- `buf breaking --against '.git#tag=proto-v1'` exits 0 (or tag does not exist yet — flag clearly)
- Generated `.pb.go`, `_connect.pb.go` files exist under `sdks/go/gen/`
- Generated `.ts` files exist under `sdks/ts/src/gen/`
