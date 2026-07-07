# Tasks — p17-c004

- [x] Implement EntityService tonic server (GetEntity unary, WatchEntity server-stream) backed by a store port
- [x] Wire it through a use-case in frf-app (ports-first; no adapter import in domain/app)
- [x] Register in frf-gateway spawn_grpc_server .add_service(entity_svc)
- [x] Add unit/integration tests; update the proto-only disclaimer comment
- [x] clippy pedantic + unwrap_used clean; no file >500 lines
