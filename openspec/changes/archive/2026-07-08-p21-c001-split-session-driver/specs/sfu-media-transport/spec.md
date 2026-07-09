# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The str0m media engine MUST keep its modules under the file-size limit

The str0m media engine's source files MUST each stay under the 500-line limit; the per-session
async driver loop MUST live in its own module (`driver.rs`) separate from the `StrOmTransport`
/ `MediaTransport` surface (`session.rs`). The split MUST be behavior-preserving — the existing
tests pass unchanged.

#### Scenario: the driver loop is a separate module under the size limit

- **WHEN** the str0m crate is built
- **THEN** `session.rs` and `driver.rs` are each under 500 lines and the driver loop lives in
  `driver.rs`

#### Scenario: the split changes no behavior

- **WHEN** the existing `frf-media-str0m` tests run after the split
- **THEN** they pass unchanged (no behavior or API change)
