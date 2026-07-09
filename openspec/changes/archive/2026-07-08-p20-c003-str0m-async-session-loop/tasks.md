# Tasks — p20-c003-str0m-async-session-loop

- [x] 1. Cargo.toml: add tokio `net`, `time`, `macros` features to frf-media-str0m
- [x] 2. session.rs (new): `StrOmTransport` (`MediaTransport` impl) + `SessionHandle` + `SessionCommand` + `run_session` async driver (poll_output→send_to/state; select! timer vs recv_from → handle_input); watch<ConnectionState>; no lib unwrap/expect
- [x] 3. session.rs: test — `create_session` on a real tokio socket turns the loop (transmit produced or state advances off New); `remove_session` tears down. lib.rs re-export `StrOmTransport`
- [x] 4. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green; file ≤500 lines
