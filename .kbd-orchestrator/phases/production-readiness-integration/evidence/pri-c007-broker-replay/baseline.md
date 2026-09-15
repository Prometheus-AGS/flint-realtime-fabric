# c007 broker replay baseline

Captured: `2026-09-15T15:21:39Z`

## Fabric source identity

- Branch: `codex/production-readiness-integration`
- Base revision: `5aaad523ad6803bd07992a6ca6e6d75238d968c9`
- Affected-path status: clean
- Affected-path binary diff SHA-256:
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

The affected scope is limited to `crates/frf-broker-iggy/src/`,
`crates/frf-broker-iggy/tests/` and `crates/frf-ports/src/log_broker.rs`.
Unrelated modified and untracked workspace files were present and are excluded
from c007.

| Current affected file | SHA-256 |
|---|---|
| `crates/frf-broker-iggy/src/broker.rs` | `67ccbbce21262ca2e861f08ea5c5f050fa1b2db677b6b326b32282bf87fcf774` |
| `crates/frf-broker-iggy/src/channel.rs` | `ae5facabec61bcb97708d48c7e90478a953b168d9d28a1864001f29f76ad406b` |
| `crates/frf-broker-iggy/src/error.rs` | `9815e7a418b388d46e695402288077f14cbeecf945ffa9fe0b5b7319aac3adc5` |
| `crates/frf-broker-iggy/src/lib.rs` | `8974e3d05280f9a4ebcb76ff7f205f79567656fac1550a7bb6e9833cd7a98af6` |
| `crates/frf-broker-iggy/tests/publish_subscribe.rs` | `1ef4efc8bc9500f201d34296bdc512ceab2728e7e9beb220b3a3d2ccc7c9afeb` |
| `crates/frf-ports/src/log_broker.rs` | `d98b32ca528dac4555b85ba411bdbbf3581608a414dacf33dad7863b7e0f15da` |

## Dependency receipts

Both required changes are committed, archived and report PASS:

| Dependency | Commit | Receipt SHA-256 | Relevant result |
|---|---|---|---|
| `pri-c002-local-fixtures` | `52ab9b3874abbe35bbca73720d85578f3db70943` | `4409fe7fa6ca1db3b01091ea8160fb4317a8102b8fd1e5074912177185716622` | Owned authenticated Iggy fixture passed and pinned image digest `iggyrs/iggy@sha256:68a314c1380be5a792a134f3bd346ded42bd49d9f7114c86f70b48fc85bc5272`. |
| `pri-c006-watch-contract` | `5aaad523ad6803bd07992a6ca6e6d75238d968c9` | `50fe1da97f0167ecfef66b06a731a5b3c7d73882011dbcf0c708657ddd07e9b9` | Versioned source/broker/checkpoint separation and replay/resnapshot semantics passed independent review. |

## Pinned Iggy source

`Cargo.lock` SHA-256 is
`d23cb3f86cbfe06e53de83865cde9e3a2c2e856b4615113f27c3901f54419b5c`.
It resolves `iggy` 0.6.203 to the exact GQAdonis fork commit
`d34b9c96ad5a15334e06040d68fd7512beeba4c8`. The Cargo checkout has only
Cargo's `.cargo-ok` marker outside Git; the pinned source revision itself is
unchanged.

| Pinned-fork evidence | SHA-256 |
|---|---|
| `sdk/src/clients/consumer.rs` | `e7c5476737b8b33e8b341b0a4e0969e2c4d40c27520642b2862251fdd3e12844` |
| `sdk/src/messages/poll_messages.rs` | `7c74d43711dafd65c740deda0d815fac5e2ec0c1d7b72a801b9c6ac0e1c19378` |
| `server/src/streaming/systems/messages.rs` | `13df827e100103f7e6bd31aab47d65639592d8219aa973da190be18b7875b2f7` |
| `configs/server.toml` | `1e22d04d71d15695561619c6040a3843f7cd7a6474138d7bae079ac2cf32b2ee` |

The current adapter does not set `auto_commit`, so the pinned builder default
is active: one-second interval or commit while polling. The server stores the
last returned offset before application processing when polling auto-commit is
enabled. The SDK then advances an explicit polling strategy to
`message.offset + 1`. This creates the c007 crash-loss window and also shows
that a requested broker offset is inclusive.

The adapter currently serializes an application-supplied
`EventEnvelope.offset`, returns that value from `publish`, and discards the
broker message offset while decoding. It therefore cannot yet supply the c006
broker position or prove producer restart uniqueness.

## Documentation cross-check

Context7 resolved Apache Iggy to `/websites/iggy_apache`. Current upstream docs
confirm that polling carries an explicit `auto_commit` flag, server-side
consumer offsets are stored separately, and polling with `Next` plus
auto-commit is at-most-once. The documentation also describes
`message_saver.enforce_fsync`, partition `enforce_fsync`, explicit partition
flush with `fsync`, and independent size/time retention. Those upstream facts
guide the test plan; only the pinned-fork source and owned runtime fixture count
as c007 compatibility evidence.

The pinned default server file enables the 30-second message saver with fsync,
but leaves partition and state `enforce_fsync` false. The Compose fixtures only
set credentials and system path, so task 4 must record and explicitly override
the actual runtime durability policy rather than assuming the source defaults.
