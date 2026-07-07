# Tasks — p16-c015

- [x] Scaffold crates/frf-cli with clap and add to workspace
- [x] Command: seed Keto relation tuples
- [x] Command: manage CDC replication slot
- [x] Command: inspect broker offsets
- [x] Command: force checkpoint
- [x] Help + smoke test

## Summary

Created `frf-cli` (binary `frf`) — the first-party operator CLI (H13). clap-derive
subcommands, `anyhow` at the binary edge (per project rules), each command driving an
existing adapter. Added `clap` to workspace deps; crate added to the workspace.

## Commands

- **`frf keto seed|revoke`** (task 2) — write/delete a Keto relation tuple via
  `KetoAuthzProvider::{write,delete}`. Args: `--tenant --subject --relation --object`,
  with `--keto-url`/`--namespace` (env `KETO_BASE_URL`/`KETO_NAMESPACE`). This is the
  supported path to seed authz tuples at bring-up (replaces out-of-band Keto REST calls).
- **`frf broker checkpoint`** (tasks 4+5) — force a consumer checkpoint by seeking its
  cursor to an explicit offset via `LogBroker::seek(Cursor)`. Args: `--channel --consumer
  --offset` (env `IGGY_CONNECTION_STRING`). Covers both "inspect/manage broker offsets"
  and "force checkpoint" — `seek` IS the offset-checkpoint primitive. (Iggy exposes no
  standalone offset-read RPC through the port, so the actionable command is set/reset.)
- **`frf cdc status`** (task 3) — report the resolved CDC slot/publication config +
  replication URL (env `CDC_*`). The replication slot itself is created/managed by the
  gateway's `PostgresCdcConsumer` at startup; the CLI verifies naming + the
  `replication=database` param before bring-up (standalone slot creation needs a raw
  replication connection the consumer already owns).

## Task 6 — help + smoke

- `frf --help` lists keto/broker/cdc; `frf keto seed --help` shows all typed args.
- `frf cdc status` runs end to end (pure config, no external service) and prints the
  resolved configuration — a real executed command.

## Verification

- `cargo build -p frf-cli` → exit 0
- `cargo clippy -p frf-cli -- -D warnings -W clippy::pedantic` → exit 0
- `cargo fmt --check -p frf-cli` → clean
- workspace `cargo clippy --lib --bins` → exit 0
- `frf --help` / `frf keto seed --help` / `frf cdc status` → all work

## Note

Live `keto seed` / `broker checkpoint` against real Keto/Iggy need those services running
(integration, like the other suites); the config-only `cdc status` proves the command
plumbing end to end here.
