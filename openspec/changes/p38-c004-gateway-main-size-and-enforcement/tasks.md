# Tasks — p38-c004-gateway-main-size-and-enforcement

> **DONE** = main.rs is under the cap and a check exists.
> **PROVEN** = the check was observed to FAIL on an oversized file and PASS after.

- [ ] T1: Confirm `main.rs` is still over 500 lines (`wc -l`).
- [ ] T2: Extract `ensure_entities_channel` into its own module; keep behaviour identical
      (the fixture failure must stay fatal, not revert to a warn!).
- [ ] T3: Confirm `cargo check -p frf-gateway`, `cargo clippy … -D warnings -W clippy::pedantic`
      and `cargo fmt --check` all still pass.
- [ ] T4: Add a file-size check covering all languages, capped at 500 lines.
- [ ] T5: Sabotage the check — point it at a deliberately oversized file and confirm it
      FAILS, then restore. A check that has never failed is a hypothesis.
- [ ] T6: Record the four near-cap files so the next overage is anticipated.
