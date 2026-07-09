# Tasks — p20-c004-trickle-ice-wiring

- [x] 1. Cargo.toml: promote `chrono` to a normal dependency (outbound envelopes need a timestamp in lib code)
- [x] 2. ice.rs (new): `candidate_string_from_envelope` (inbound) + `candidate_envelope`/`state_envelope` (outbound) helpers; unit tests for the mapping (deterministic, no socket)
- [x] 3. session.rs: add a `local_signals` broadcast to `SessionHandle`; broadcast the host candidate at create + state envelopes on change; real `BroadcastStream` from `local_signals`; inbound path uses `candidate_string_from_envelope`
- [x] 4. session.rs test: `local_signals` yields the host-candidate envelope after create; lib.rs registers `ice`
- [x] 5. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green; files ≤500 lines
