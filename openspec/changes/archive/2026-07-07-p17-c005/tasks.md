# Tasks — p17-c005

- [x] Implement AuthzService tonic server (Check, WriteRelation, DeleteRelation) delegating to KetoAuthzProvider
- [x] Route through frf-app use-case against the AuthzProvider port
- [x] Register in frf-gateway spawn_grpc_server
- [x] Tests: check allow/deny, write+delete roundtrip; update disclaimer comment
- [x] Never log relation tuples; clippy/unwrap gates clean
