# Tasks — p20-c005-dtls-connected-milestone

- [x] 1. session.rs: install the crypto provider (from_feature_flags().install_process_default(), once) in StrOmTransport::new
- [x] 2. session.rs: `wait_for_connected(session_id, timeout)` awaiting the state watch → Connected
- [x] 3. session.rs test: in-process two-session loopback drives DTLS to Connected (or #[ignore] integration-gated with documented reason); SPIKE-FINDINGS.md records the milestone; SFU_MODE stays off
- [x] 4. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green; file ≤500 lines
