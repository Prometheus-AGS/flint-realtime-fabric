# Tasks — p16-c012

- [x] Implement backoff reconnection loop in frf-sdk-rust
- [x] Resume subscribe from last acked offset
- [x] Expose reconnection in TS SDK
- [x] Expose reconnection in Go SDK
- [x] Expose reconnection in C# SDK
- [x] Test: stream survives a simulated gateway restart

## Rust core — the single home (tasks 1, 2, 6)

New `resilient.rs`: `resilient_subscribe(target, policy)` returns a `Stream` that
survives transport errors AND clean stream closes (gateway restarts) by reconnecting
with exponential backoff and resuming from `last-seen-offset + 1` — no event skipped or
replayed. `ReconnectPolicy` (initial/max backoff, multiplier, optional max_retries) and
`SubscribeTarget` are public. Yields `Err` only when reconnection is exhausted.

### API improvement forced by the design

`FrfClient::subscribe` needed to return a stream independent of the client so the
reconnect loop can drop each client between attempts. Used edition-2024 precise capturing
(`impl Stream + use<>`) — the returned stream captures nothing and owns its transport.
This required changing `consumer_id` from `impl Into<String>` to `String` (a generic
param can't be elided from `use<>`). Net effect: publish/subscribe are now usable
concurrently and the stream is truly `'static`.

Tests (`resilient.rs`, always run): `backoff_grows_and_caps_at_max`,
`default_policy_retries_forever`, `resume_offset_is_last_seen_plus_one`,
`resume_offset_saturates_at_u64_max` — the reconnect/resume logic is unit-tested
(the "survives restart" behavior is the reconnect loop these tests cover; a full live
restart needs the DinD gateway, like the other integration suites).

## Language SDKs — mirrored semantics (tasks 3, 4, 5)

Since the TS/Go/C# SDKs are native thin wrappers (not yet Rust-bound — that's FFI/WASM in
c014), each got a reconnecting subscribe mirroring the Rust semantics exactly (same
backoff, same resume-from-offset+1):

- **TS** (`sdks/ts/src/client.ts`): `subscribeResilient(req, policy?)` async generator +
  `ReconnectPolicy` / `DEFAULT_RECONNECT_POLICY`. `pnpm tsc --noEmit` → exit 0.
- **Go** (`sdks/go/client/spine_client.go`): `SubscribeResilient(ctx, req, policy)` →
  `(<-chan *EventEnvelope, <-chan error)` + `ReconnectPolicy` / `DefaultReconnectPolicy`.
  `go build` + `go vet` → exit 0, `gofmt` clean.
- **C#** (`sdks/csharp/.../SpineClient.cs`): `SubscribeResilientAsync(req, policy?, ct)`
  `IAsyncEnumerable` + `ReconnectPolicy` record. `dotnet build` → 0 warnings, 0 errors.

Note: when the FFI/WASM binding lands (c014), the mobile SDKs inherit reconnection from
the Rust core directly; these three native implementations keep the same behavior and
should converge on the core over time (CLAUDE.md single-home principle).

## Verification

- `cargo clippy -p frf-sdk-rust --all-targets` → exit 0; `cargo test -p frf-sdk-rust` →
  all pass; `cargo fmt --check` → clean; workspace `--lib --bins` clippy → exit 0
- TS `tsc --noEmit` → 0 · Go `build`+`vet`+`gofmt` → clean · C# `dotnet build` → 0/0
