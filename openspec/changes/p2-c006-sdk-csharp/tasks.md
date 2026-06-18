# Tasks — p2-c006 sdk-csharp

- [ ] **T1** Create `sdks/csharp/` directory structure
  - `mkdir -p sdks/csharp/FlintSdk/Client sdks/csharp/FlintSdk/proto/flint/v1`
  - Verification: directories exist

- [ ] **T2** Copy proto files into `sdks/csharp/FlintSdk/proto/flint/v1/`
  - `cp proto/flint/v1/*.proto sdks/csharp/FlintSdk/proto/flint/v1/`
  - These are a READ-ONLY copy for Grpc.Tools; edits must go to the canonical `proto/` location
  - Verification: 6 `.proto` files exist in the copy

- [ ] **T3** Create `sdks/csharp/FlintSdk/FlintSdk.csproj`
  - TargetFramework: `net9.0`
  - Nullable: enable; ImplicitUsings: enable
  - PackageReferences: `Google.Protobuf 3.29.3`, `Grpc.Net.Client 2.67.0`, `Grpc.Tools 2.67.0 (PrivateAssets=All)`
  - Protobuf item group: `Include="proto/flint/v1/*.proto" GrpcServices="Client"`
  - Verification: valid XML; `dotnet restore` exits 0

- [ ] **T4** Create `sdks/csharp/FlintSdk.sln`
  - `dotnet new sln -n FlintSdk -o sdks/csharp/`
  - `dotnet sln sdks/csharp/FlintSdk.sln add sdks/csharp/FlintSdk/FlintSdk.csproj`
  - Verification: `.sln` file references the project

- [ ] **T5** Create `sdks/csharp/FlintSdk/Client/SpineClient.cs`
  - Namespace: `PrometheusAgs.Frf.Flint.V1.Client`
  - `sealed class SpineClient : IDisposable` wrapping a `GrpcChannel`
  - `PublishAsync(EventEnvelope, CancellationToken) → Task<PublishResponse>`
  - `SubscribeAsync(SubscribeRequest, CancellationToken) → IAsyncEnumerable<EventEnvelope>`
  - `Dispose()` disposes the channel
  - Verification: file compiles as part of `dotnet build`

- [ ] **T6** Run `dotnet build`
  - `dotnet build sdks/csharp/FlintSdk.sln`
  - Verification: exits 0; no errors or nullable warnings

- [ ] **T7** Nullable-clean verification
  - `dotnet build sdks/csharp/FlintSdk.sln -p:TreatWarningsAsErrors=true`
  - Verification: exits 0
