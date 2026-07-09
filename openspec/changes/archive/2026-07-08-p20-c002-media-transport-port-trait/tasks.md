# Tasks — p20-c002-media-transport-port-trait

- [x] 1. media_transport.rs (new): `ConnectionState` (#[non_exhaustive]) + `MediaTransport` trait (create_session/add_remote_candidate/local_signals/connection_state/remove_session) + `DynMediaTransport` wrapper
- [x] 2. lib.rs: `pub mod media_transport;` + re-exports; verify frf-ports stays impl-free (cargo fmt + clippy pedantic + unwrap_used + build)
