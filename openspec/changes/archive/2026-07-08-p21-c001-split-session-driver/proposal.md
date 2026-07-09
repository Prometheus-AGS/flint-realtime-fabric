# p21-c001 — split the session driver out of session.rs

## Why

`crates/frf-media-str0m/src/session.rs` is at **498 / 500 lines** (the CLAUDE.md limit).
Phase-21's RTP forwarding (c004) adds a room registry + forwarding channels to this file, so
it must gain headroom **first**. This is a behavior-preserving refactor: extract the async
driver loop into a sibling `driver.rs`, leaving `session.rs` as the `StrOmTransport` /
`MediaTransport` surface.

## What Changes

Pure refactor — no behavior change, no test change.

1. **`driver.rs` (new):** move the per-session driver internals out of `session.rs` —
   `RECV_BUF`, `SessionCommand`, `SessionMeta`, `state_for_event`, and `run_session` (the
   async loop). Marked `pub(crate)` so `session.rs` uses them.
2. **`session.rs`:** keep the crypto init, `SessionHandle`, `StrOmTransport`, its `Default`,
   the `impl StrOmTransport` (`new`/`negotiate`/`wait_for_connected`), and the
   `impl MediaTransport`; import the moved items from `driver`.
3. **`lib.rs`:** `pub mod driver;` (its items are `pub(crate)`; no new public surface).

## Non-goals

- Any behavior change, new feature, or test change (the 18 tests / 1 `#[ignore]` are unchanged).
- RTP forwarding (c004).

## Impact

- Affected: `crates/frf-media-str0m/src/{session.rs,driver.rs,lib.rs}`.
- `session.rs` back under 500 lines with room for the c004 registry; all existing tests green.
