# p2-c002 — Add `csharp_namespace` option to all proto files

## Phase
phase-2-generated-sdks

## Affected files
- `proto/flint/v1/envelope.proto`
- `proto/flint/v1/entity.proto`
- `proto/flint/v1/agent.proto`
- `proto/flint/v1/signal.proto`
- `proto/flint/v1/sync.proto`
- `proto/flint/v1/authz.proto`

## What this change does

Adds `option csharp_namespace = "PrometheusAgs.Frf.Flint.V1";` to each of
the 6 proto files, immediately after the existing `go_package` option line.

Without this option, the C# `Grpc.Tools` generator uses the proto `package`
name (`flint.v1`) as the C# namespace, which produces `flint.v1` — a valid
proto identifier but an invalid C# namespace (`.v1` has no namespace
component separator meaning in C#; the dot notation conflicts with `using`
directives in some toolchains). The correct .NET namespace convention is
PascalCase components separated by dots: `PrometheusAgs.Frf.Flint.V1`.

## Why now (before C# codegen)

`proto-v1` is frozen — any proto edit is a potential breaking change. This
option is metadata-only (it does not change message encoding or wire format)
and is safe to add post-freeze. However, it must be added before the C# SDK
is generated (p2-c006) since the namespace appears in all generated file
headers.

## Exit criteria

- `grep -c csharp_namespace proto/flint/v1/*.proto` == 6
- `buf lint` passes (proto/buf.yaml from p2-c003 not required for this check;
  can verify with `protoc --descriptor_set_out=/dev/null` after this change)
- `buf breaking --against '.git#tag=proto-v1'` confirms no wire-format changes
  (metadata options are not breaking)
