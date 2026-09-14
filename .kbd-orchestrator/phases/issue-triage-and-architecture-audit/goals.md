# Goals — issue-triage-and-architecture-audit

> Seeded manually after phase-36 closed (3/3, archived). **Three issues are open and one
> of them blocks the other two.** Issue #6 means no Iggy server can complete a handshake
> with the pinned client, so *no* local integration run of `frf-broker-iggy` is possible —
> and under this repo's local-integration-only policy, that makes #2's fix unprovable.
> This phase clears that blocker, proves or disproves the #2 fix, fixes #7, and audits the
> architecture for the drift and vacuous-guard defects that let #2 be misdiagnosed for two
> months.

## G1 — Unblock local integration testing (issue #6)

**Exit:** `cargo test -p frf-broker-iggy -- --ignored` completes (pass or fail) against a
local Iggy server instead of hanging, and the server logs a non-zero client count.

No available server speaks to the pinned client: client **0.6.203** (GQAdonis fork
`d34b9c96`, `Cargo.lock:4501`), `iggyrs/iggy:latest` is **0.4.214** (`compose.yml:58`),
and the digest pinned in `k8s/overlays/ssr/iggy.yaml:41` is **0.8.13**. Both servers were
tried on 2026-09-14; each logged `Clients: 0, Messages processed: 0` for its whole uptime
while the test hung until killed. Ruled out: TCP reachability, credentials (`iggy:iggy`
authenticates — CLI `ping` returns in 2.44 ms over the same host→container path), and port
forwarding. The mechanism was **not** determined — do not assume a protocol-version story
without evidence.

Also fix the compose drift this exposed: `compose.yml` and `compose.ci.yml` use the
floating tag `latest` while `k8s/overlays/ssr` pins a digest, so this breakage can change
without any repo change. Commit `26e4dfc` is titled "align FRF broker with **pinned** Iggy
server" yet no pin exists in either compose file.

## G2 — Prove (or disprove) the issue #2 fix

**Exit:** the guard `a_subscriber_knowing_only_the_well_known_id_receives_published_events`
is observed to **pass** against a live server **and** observed to **fail** when
`ChannelId::WELL_KNOWN_ENTITIES` is reverted to `ChannelId::new()`. Both outcomes recorded.
Only then close #2.

Commit `788637a` fixed the real cause — channel ids minted randomly at publish time — but
its end-to-end verification was blocked by G1. The sabotage step never ran. By this
project's own standard the new guard is a hypothesis, not a guard: **a guard that has
never failed proves nothing.** Depends on G1.

## G3 — Fix issue #7: E2E smoke sends a non-UUID tenantId

**Exit:** `tests/e2e/smoke_test.sh` publishes successfully instead of being rejected at the
gateway boundary.

`smoke_test.sh:48` sends `"tenantId": "e2e-tenant"`, which `parse_tenant_id`
(`grpc_service.rs:64-68`) rejects with `invalid_argument` because it is not a UUID. The
script cannot pass today regardless of any other fix. The sibling TS/Go/C# clients already
use a valid UUID; it is specifically the shell script's hard-coded string.

## G4 — Reconcile CLAUDE.md against the ADRs that already settled it

**Exit:** every row of CLAUDE.md's "Open Decisions" table is either genuinely open or
removed with a pointer to the ADR that closed it; the workspace tree matches disk.

Confirmed drift as of 2026-09-14:

| CLAUDE.md claim | Reality |
|---|---|
| `:102` "CRDT: Loro **or** automerge-rs — **OPEN**" | ADR-001 accepted Loro 2026-06-19; `Cargo.toml:146` ships `loro 1.13.1` |
| `:274` four decisions "must be resolved" | ADR-001 and ADR-003 settled CRDT and the FFI/codegen toolchain |
| workspace tree | `frf-did`, `frf-p2p`, `frf-shape-electric`, `frf-wallet`, `uniffi-bindgen` exist on disk but are absent from the tree |
| `:256` Dart = "flutter_rust_bridge over Rust core" | ADR-003 overrode this; FRB's parser panics on `#[uniffi::export]` |

This is not cosmetic. Stale docs are what produced issue #2's incorrect table — a reader
followed `channel.rs`'s `tenant-` naming, which no code had used since `26e4dfc`.

## G5 — Hunt the vacuous-guard and dead-code class workspace-wide

**Exit:** every `#[ignore]`d test is classified as (a) runnable once G1 lands and then
actually run, or (b) documented as permanently unrunnable with the reason. Every remaining
`pub fn` with zero production callers is deleted or marked with an explicit reason.

Three vacuous guards were found in one session, all the same shape: a test asserting on
code nothing calls. `channel_mapping.rs` was deleted in `788637a`; two more were found in
the TS and Rust shape clients. The workspace has **4** `#[ignore]`d tests — by definition
none has ever run in the default suite:

- `frf-broker-iggy/tests/publish_subscribe.rs:29`, `:85` (blocked by G1)
- `frf-postgres-cdc/tests/cdc_integration.rs:16` (needs local Postgres 17 + logical replication)
- `frf-gateway/tests/subscribe_mux.rs:10` (needs Iggy + Keto + flint-gate)

Assume none of them bites until observed failing.

## G6 — Repair the file-size violation introduced by `788637a`

**Exit:** `crates/frf-gateway/src/main.rs` is under CLAUDE.md's hard 500-line cap, and a
check exists that would catch the next overage.

`main.rs` went **487 → 504 lines** in `788637a` (this session's own commit), crossing the
cap. Four more files sit within 25 lines of it: `frf-gateway/src/config/mod.rs` (498),
`frf-sdk-rust/src/shape.rs` (495), `frf-app/src/shape/tests.rs` (484),
`frf-app/src/shape/lease.rs` (477). The cap is declared but nothing enforces it — that is
why it was crossed silently.
