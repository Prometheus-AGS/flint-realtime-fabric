# Tasks — p16-c016

- [x] Point C# codegen at proto/flint/v1 source of truth
- [x] Remove committed proto copy from sdks/csharp
- [x] Regenerate and build C# SDK

## Problem confirmed

The C# SDK's `.csproj` had `<Protobuf Include="proto/flint/v1/*.proto">` pointing at a
**committed copy** under `sdks/csharp/FlintSdk/proto/`, forked from the frozen source of
truth at the repo root. Verified the fork had drifted: `agent.proto` DIVERGED between the
copy and `/proto/flint/v1/agent.proto` (exactly the audit's #32 finding). The other five
were still identical, but any could drift the same way.

## Fix

1. **Point at source (task 1):** `<Protobuf Include="../../../proto/flint/v1/*.proto"
   ProtoRoot="../../../proto" GrpcServices="Client" />`. The relative path resolves from
   the `.csproj` dir to the repo-root proto; `ProtoRoot` keeps the `flint.v1` package
   paths and cross-file imports resolving. The SDK now generates from the one source every
   build and can never drift from the contract.
2. **Remove the fork (task 2):** deleted `sdks/csharp/FlintSdk/proto/` entirely, plus the
   stale `obj/`/`bin/` (gitignored) that held code generated from the old fork.
3. **Regenerate + build (task 3):** `dotnet build` regenerates C# from the source proto —
   **0 warnings, 0 errors**. `Agent.cs` (and the rest) is now generated from the
   source-of-truth `agent.proto`, resolving the divergence.

## Verification

- No `.proto` files remain in `sdks/csharp` (only generated `.cs` under gitignored `obj/`).
- `dotnet build FlintSdk.csproj` → Build succeeded, 0/0.
- `bin/` and `obj/` are gitignored, so no generated artifacts are committed.
