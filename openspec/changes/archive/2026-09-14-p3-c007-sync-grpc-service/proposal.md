# p3-c007 — `SyncGrpcService` in `frf-gateway`

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c006 (SyncUseCase), p3-c005 (SurrealCrdtStore available for wiring)

## Directory
`crates/frf-gateway/src/`

## What this change does

Implements the tonic server-side handler for `SyncService` from `proto/flint/v1/sync.proto`.
Wires `SyncUseCase` into the gateway and registers the `SyncServiceServer` on the Axum
router alongside the existing `SpineServiceServer`.

### `SyncGrpcService`

```rust
pub struct SyncGrpcService<C, O> {
    use_case: Arc<SyncUseCase<C, O>>,
}
```

Implements `sync_service_server::SyncService` (generated from proto):
- `sync()` — bidi streaming RPC: for each incoming `SyncRequest`, call
  `use_case.apply_incoming()`, emit a `SyncResponse` with the merged state
- `get_checkpoint()` — unary: restore current snapshot from `CrdtStore`; return
  `SyncCheckpoint` message

### Gateway wiring (in `main.rs`)

```rust
let surreal_store = SurrealCrdtStore::connect(url, ns, db).await?;
let redb_store = RedbOpStore::open(op_log_path)?;
let apply_delta = Arc::new(LoroDeltaApplier::new());  // thin adapter
let sync_use_case = Arc::new(SyncUseCase::new(surreal_store, redb_store, apply_delta));
let sync_service = SyncGrpcService::new(sync_use_case);

Router::new()
    .add_service(SpineServiceServer::new(spine_service))
    .add_service(SyncServiceServer::new(sync_service))
```

### JWT / authz constraint

The `sync()` handler must extract the JWT subject from tonic request metadata via
the existing `OryIdentityVerifier` middleware pattern used by `SpineGrpcService`.
Do not trust unverified claims downstream (CLAUDE.md security constraint).

## Non-goals

- Does not add WebRTC peer sync (Phase 4).
- Does not add Cedar action-policy checks for CRDT ops (Phase 5).
- Does not change the proto contract — implements what is already in `sync.proto`.
