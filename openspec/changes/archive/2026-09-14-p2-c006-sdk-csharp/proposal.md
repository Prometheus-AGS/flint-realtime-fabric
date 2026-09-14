# p2-c006 — C# SDK

## Phase
phase-2-generated-sdks

## Depends on
p2-c002 (csharp_namespace must be in all proto files before Grpc.Tools generation)

## Directory
`sdks/csharp/`

## What this change does

Creates the C# SDK project using `Grpc.Tools` MSBuild integration to generate
protobuf/gRPC stubs from the frozen proto files.

> **Why not buf?** buf has no first-party C# Connect plugin. The C# ecosystem
> uses gRPC (not Connect protocol). Clients connect via standard gRPC-Core or
> `Grpc.Net.Client` over HTTP/2.

### Project structure

```
sdks/csharp/
├── FlintSdk.sln
└── FlintSdk/
    ├── FlintSdk.csproj        # targets net9.0; Grpc.Tools; proto <Protobuf Include="...">
    ├── proto/                 # LOCAL COPY of proto/flint/v1/*.proto
    │   └── flint/v1/*.proto
    └── Client/
        └── SpineClient.cs     # hand-written wrapper over generated GrpcChannel stubs
```

### `FlintSdk.csproj` (key fragments)

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Google.Protobuf" Version="3.29.3" />
    <PackageReference Include="Grpc.Net.Client" Version="2.67.0" />
    <PackageReference Include="Grpc.Tools" Version="2.67.0" PrivateAssets="All" />
  </ItemGroup>
  <ItemGroup>
    <Protobuf Include="proto/flint/v1/*.proto" GrpcServices="Client" />
  </ItemGroup>
</Project>
```

### `SpineClient.cs` surface

```csharp
namespace PrometheusAgs.Frf.Flint.V1.Client;

public sealed class SpineClient : IDisposable
{
    public static SpineClient Create(string address);
    public Task<PublishResponse> PublishAsync(EventEnvelope envelope, CancellationToken ct = default);
    public IAsyncEnumerable<EventEnvelope> SubscribeAsync(SubscribeRequest request, CancellationToken ct = default);
    public void Dispose();
}
```

## Exit criteria

- `dotnet build` from `sdks/csharp/` exits 0
- Generated `*.cs` stubs exist under `obj/` (standard Grpc.Tools output)
- `dotnet build -p:TreatWarningsAsErrors=true` exits 0 (nullable clean)
