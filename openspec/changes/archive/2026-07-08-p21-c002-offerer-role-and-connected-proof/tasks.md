# Tasks — p21-c002-offerer-role-and-connected-proof

- [x] 1. session.rs tests: `connect_two_peers()` harness — offerer + answerer Rtc, loopback UdpSockets, SDP+candidate exchange, shuttle loop (Transmit→send_to→recv_from→handle_input)
- [x] 2. session.rs: un-`#[ignore]` `two_peers_reach_dtls_connected` → both reach is_connected() within a bounded time (or re-gate #[ignore] with documented local pass if CI-flaky)
- [x] 3. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green
