# Tasks — p21-c001-split-session-driver

- [x] 1. driver.rs (new): move RECV_BUF + SessionCommand + SessionMeta + state_for_event + run_session out of session.rs (pub(crate)); session.rs imports them
- [x] 2. lib.rs: `pub mod driver;`; fix session.rs imports; both files ≤500 lines
- [x] 3. Verify: cargo fmt + clippy (pedantic, unwrap_used) + all existing tests green (unchanged)
