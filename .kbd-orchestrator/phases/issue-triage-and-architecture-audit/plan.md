# Plan — issue-triage-and-architecture-audit

> Backend: OpenSpec. Changes are in `openspec/changes/p38-c00[1-6]-*/`.
> Phase goal: clear the three open GitHub issues, correct the documentation drift that
> caused one of them to be misdiagnosed for two months, and repair two BLOCKING constraint
> violations — one of which this session introduced.
>
> **The ordering constraint that matters:** `p38-c001` is diagnostic, not corrective. No
> Iggy server available to this project completes a handshake with the pinned client, and
> **the mechanism is undetermined**. Planning a fix before the cause is known would be
> guessing; `c002` cannot even be verified until `c001` lands.

PLAN: issue-triage-and-architecture-audit
Project: flint-realtime-fabric
Date: 2026-09-14
OpenSpec available: YES
Changes to implement: 6

---

## Ordered Change List

### p38-c001 — Diagnose the Iggy client/server handshake failure (issue #6)

**Priority: FIRST — blocks c002 and every future integration proof in this repo.**

- **Scope:** tooling | compose | diagnosis
- **Depends on:** NONE
- **Recommended agent:** Claude Code (protocol-level debugging, uncertain cause)
- **Est. complexity:** M
- **Complexity score:** High
- **Model class:** frontier
- **Customer value:** HIGH (unblocks all local integration testing)

**Details.** Client is **0.6.203** (`Cargo.lock:4501`, GQAdonis fork `d34b9c96`);
`iggyrs/iggy:latest` is **0.4.214**; the digest pinned in `k8s/overlays/ssr/iggy.yaml:41`
is **0.8.13**. Both servers were run on 2026-09-14 and each logged
`Clients: 0, Messages processed: 0` for its entire uptime while
`cargo test -p frf-broker-iggy -- --ignored` hung until killed.

**Already ruled out — do not re-test these:** TCP reachability (host connect succeeds),
credentials (`iggy:iggy` authenticates; CLI `ping` returns in 2.44 ms over the *same*
host→container path the test uses), and port forwarding.

**This change must first observe, then fix.** Capture the client side of the handshake
(`RUST_LOG=iggy=trace` or equivalent) and determine where it stalls. Only then decide
between (a) pinning a server digest that matches the fork, or (b) repinning the client.
**Option (b) widens scope considerably** — if the diagnosis points there, stop and report
rather than proceeding.

Secondary, once the cause is known: replace the floating `iggyrs/iggy:latest` tag in
`compose.yml:58` and `compose.ci.yml:51` with a digest, so this breakage cannot change
without a repo change. Note `26e4dfc` is titled "align FRF broker with **pinned** Iggy
server" while no pin exists in either compose file.

**Exit:** `cargo test -p frf-broker-iggy -- --ignored` completes (pass *or* fail) instead
of hanging, and the server reports a non-zero client count.

---

### p38-c002 — Prove or disprove the issue #2 fix by sabotage

**Priority: SECOND — hard dependency on c001.**

- **Scope:** test | verification
- **Depends on:** p38-c001
- **Recommended agent:** Claude Code
- **Est. complexity:** S
- **Complexity score:** Low
- **Model class:** small
- **Customer value:** HIGH (converts an unproven claim into a proven one)

**Details.** `788637a` fixed the real cause of #2 — channel ids minted randomly at publish
time — but its end-to-end verification was blocked, and **the sabotage step never ran**.
The guard `a_subscriber_knowing_only_the_well_known_id_receives_published_events` has
never been observed to fail.

Run it against the working server from c001. Then revert
`ChannelId::WELL_KNOWN_ENTITIES` to `ChannelId::new()` and confirm the test **fails**.
Record both outcomes. A guard that has never failed is a hypothesis, not a guard.

**Close issue #2 only on demonstrated receipt** — not on a green compile.

**Exit:** both outcomes recorded; #2 closed with the run output linked, or reopened with
what the run actually showed.

---

### p38-c003 — Fix the non-UUID tenantId in the E2E smoke script (issue #7)

**Priority: THIRD — independent, small, unblocks the E2E path.**

- **Scope:** test tooling
- **Depends on:** NONE
- **Recommended agent:** OpenCode or Cline (single-line shell edit)
- **Est. complexity:** S
- **Complexity score:** Low
- **Model class:** small
- **Customer value:** MEDIUM

**Details.** `tests/e2e/smoke_test.sh:48` sends `"tenantId": "e2e-tenant"`;
`parse_tenant_id` (`grpc_service.rs:64-68`) rejects any non-UUID with `invalid_argument`,
so the script cannot pass regardless of any other fix. The sibling TS/Go/C# clients already
default to a valid UUID. Use the fixture tenant `00000000-0000-0000-0000-000000000001`
(matching `CDC_TENANT_ID` in `compose.yml:29`) or read it from an env var with that default.

**Exit:** the script's publish is accepted at the gateway boundary rather than rejected.
Note that a *full* green E2E run may still depend on c001.

---

### p38-c004 — Repair the file-size violation introduced by `788637a`

**Priority: FOURTH — cheap, self-inflicted, BLOCKING constraint R5.**

- **Scope:** frf-gateway
- **Depends on:** NONE
- **Recommended agent:** Claude Code
- **Est. complexity:** S
- **Complexity score:** Low
- **Model class:** small
- **Customer value:** LOW (internal), but it fails the project's own QA gate

**Details.** `crates/frf-gateway/src/main.rs` is **504** lines against a hard 500-line cap
(`constraints.md` R5, BLOCKING). It was 487 before `788637a` — **this session caused it**,
by extracting `ensure_entities_channel` into the same file rather than a new module.

Move `ensure_entities_channel` (and plausibly the telemetry init) into a `bootstrap`
module. Four files sit within 25 lines of the cap and will cross it next:
`frf-gateway/src/config/mod.rs` (498), `frf-sdk-rust/src/shape.rs` (495),
`frf-app/src/shape/tests.rs` (484), `frf-app/src/shape/lease.rs` (477).

**Add an enforcement check** — the cap is declared in two documents and enforced by
nothing, which is why it was crossed silently. A `just`/script target or a CI *lint* step
(permitted: CI may lint, only never test) closes the loop.

**Exit:** no file over 500 lines; a check exists that fails when one appears.

---

### p38-c005 — Reconcile CLAUDE.md with the ADRs that already settled it

**Priority: FIFTH — no code risk, high downstream value.**

- **Scope:** docs
- **Depends on:** NONE
- **Recommended agent:** Claude Code
- **Est. complexity:** S
- **Complexity score:** Low
- **Model class:** small
- **Customer value:** MEDIUM

**Details.** Confirmed drift:

| CLAUDE.md | Reality |
|---|---|
| `:102` CRDT "Loro **or** automerge-rs — **OPEN**" | ADR-001 accepted Loro 2026-06-19; `Cargo.toml:146` ships `loro 1.13.1` |
| `:274` four decisions "must be resolved" | ADR-001 and ADR-003 settled CRDT and the FFI/codegen toolchain |
| workspace tree | `frf-did`, `frf-p2p`, `frf-shape-electric`, `frf-wallet`, `uniffi-bindgen` exist on disk, absent from the tree |
| `:256` Dart = flutter_rust_bridge | ADR-003 overrode this; FRB panics on `#[uniffi::export]`. Dart uses `uniffi-bindgen-dart` |

**This is not cosmetic.** Stale naming docs are precisely what produced issue #2's
incorrect table: a reader followed `channel.rs`'s `tenant-` helpers, which no code had
called since `26e4dfc`. Each corrected row cites the ADR that closed it.

**Exit:** every "Open Decisions" row is genuinely open or removed with an ADR pointer; the
workspace tree matches disk.

---

### p38-c006 — Sweep dead code and classify every unproven guard

**Priority: LAST — depends on c001 to classify the Iggy-blocked tests honestly.**

- **Scope:** workspace-wide
- **Depends on:** p38-c001 (for classification, not for the deletions)
- **Recommended agent:** Claude Code
- **Est. complexity:** M
- **Complexity score:** Medium
- **Model class:** medium
- **Customer value:** MEDIUM

**Details.** Confirmed findings:

- **`crates/frf-gateway/src/ws.rs` is orphaned** — 20 lines exporting `ws_echo`, whose only
  occurrence in the workspace is its own definition. `lib.rs` declares twelve modules; `ws`
  is not among them and no `mod ws;` exists. The file never compiles into the binary.
  Delete unless it is a deliberate stub, in which case say so in the file.
- **`fetch_and_cache` is NOT dead** — it has two in-file callers
  (`frf-identity-ory/src/jwks.rs:58`, `:72`). An earlier sweep flagged it wrongly by
  excluding the defining file. **Do not delete it.** Recorded so the mistake is not repeated.
- **Four `#[ignore]`d tests, none ever executed:**
  `frf-broker-iggy/tests/publish_subscribe.rs:29`, `:85`;
  `frf-postgres-cdc/tests/cdc_integration.rs:16` (needs local Postgres 17 + logical
  replication); `frf-gateway/tests/subscribe_mux.rs:10` (needs Iggy + Keto + flint-gate).

Classify each as (a) runnable now — then run it, or (b) permanently unrunnable locally —
then document the reason in the `#[ignore]` string. Three vacuous guards were found and
removed earlier in this session; assume none of these bites until observed failing.

**Exit:** every ignored test classified and, where runnable, actually run; every remaining
zero-caller `pub fn` deleted or annotated with a reason.

---

## EXECUTION ROUND ORDER

```
Round 1 (parallel): p38-c001, p38-c003, p38-c004, p38-c005
Round 2 (serial):   p38-c002   (hard dependency on c001)
Round 3:            p38-c006   (classification needs c001's outcome)
```

`c003`, `c004` and `c005` touch disjoint trees (test script, gateway crate, root docs) and
share no files with `c001`, so they parallelise safely.

---

## Ordering Rationale

1. **c001 first because it is the only true blocker.** Under this repo's
   local-integration-only policy, an unreachable broker means no integration claim in this
   workspace can be proven. It gates c002 outright and c006's classification.
2. **c001 is scoped to diagnose before it fixes.** The assessment establishes what the
   failure is *not* (TCP, credentials, forwarding) and explicitly does not establish what
   it *is*. A change that presumed "version mismatch" would be building on a guess.
3. **c003/c004/c005 are deliberately independent** so the phase produces value even if
   c001 proves hard. None of them requires a running broker.
4. **c004 is small but non-negotiable** — R5 is BLOCKING in this project's own QA gate, and
   the violation is this session's fault. It carries an enforcement check so the next
   overage is caught rather than discovered.
5. **c006 is last** because classifying the Iggy-blocked tests honestly requires knowing
   whether c001 made them runnable.

---

## Sycophancy Self-Check

- **S-02 (agreement without grounding).** The plan does not assume G1 is achievable. It
  states the mechanism is unknown, forbids assuming a version story, and names the
  scope-widening outcome (repinning the client) as a stop-and-report condition rather than
  quietly absorbing it.
- **S-07 (scope creep).** Six changes map 1:1 onto the six phase goals. Nothing was added.
  Two adjacent problems found during assessment were **deliberately excluded**: the 126
  unarchived OpenSpec changes (P1) and the phase-37 child's 85 unexecuted tests. Both are
  real, neither is in this phase's goals, and folding them in would double the phase.
- The plan records one failure caused by this session (`c004`) rather than presenting the
  work so far as clean.

---

## Deferred — recorded, deliberately not in this phase

- ~~**P1 spec-ledger drift.** 126 change directories outside `archive/` vs 89 archived; at
  least 42 are complete but unarchived, and `openspec list` cannot parse them
  (`Rules for 'proposal' must be an array of strings`). A BLOCKING constraint failure that
  warrants its own phase — bulk-archiving would bury the ~84 whose state is unverified.~~

  **SUPERSEDED — 2026-09-14. Addressed directly at operator direction, not deferred.** The
  deferral rationale above rested on a false premise: `openspec list` parses the ledger
  fine (127 clean rows on stdout); the repeated stderr warning meant the CLI was *ignoring
  this project's custom rules*, not failing to read changes. The two real defects were
  fixed rather than deferred:

  1. **`openspec/config.yaml:57`** — an unquoted `: ` inside a list entry made it parse as
     a mapping, so Zod rejected the entire `proposal` rules array and dropped all five
     rules silently. Quoting it took **120 warnings → 0**; `proposal` and `tasks` now load
     5 strings each, so this project's ten custom rules are enforced for the first time.
  2. **No spec deltas in the active pile** — 0 of 126 active changes had `specs/`, against
     87 of 89 archived. Every active change therefore failed `openspec validate` with one
     identical error. Remedy applied per the CLI's own guidance: `.openspec.yaml` carrying
     `skip_specs: true` (120 files created, 6 appended without clobbering existing
     `schema`/`created`/`goal` keys). Verified by two independent measurements — a
     per-change loop (126 valid / 0 invalid) and the CLI's bulk validator
     (`items=126 passed=126 failed=0`).

  **Checkboxes were deliberately left untouched** at operator direction. They are stale —
  12 of 12 sampled reference files that exist — but a file existing is weak evidence a task
  was completed as written, and mass-ticking 126 `tasks.md` files on that heuristic would
  bake the inference into the record.
- **The phase-37 child's 85 authored-but-unexecuted tests**, carried forward from
  phase-36's reflection as the largest unproven surface in the repo.
- **Adversarial review of this plan** — see below.

---

## Adversarial review — NOT PERFORMED

Step 9 calls for `/adversarial-review --mode artifact plan` before emitting change
structures. **It did not run**, for the same two reasons recorded in `assessment.md`: the
model gateway at `http://localhost:4000/v1` returns HTTP `401` — reachable but unauthenticated; CORRECTED
from an earlier false `000`/unreachable claim (the `model-preflight.json`
`status: "ok"` is a stale 24 h cache), and `distinct_models: 2` cannot fill three roles, so
the judge would collide with the producer regardless.

Per the skill's "never block the stage on preflight status", planning proceeded. The
consequence: **no CRITICAL findings were generated for this plan** — in particular nothing
independently checked the ordering or the dependency claims above. Downstream stages must
not read that silence as endorsement. Re-vet with
`/adversarial-review --mode artifact plan` against this file if the gateway is restored
before `/kbd-execute`.

---

## COMMANDS TO RUN

```
/opsx:new p38-c001-iggy-handshake-diagnosis
/opsx:new p38-c002-prove-channel-id-fix
/opsx:new p38-c003-e2e-tenant-uuid
/opsx:new p38-c004-gateway-main-size-and-enforcement
/opsx:new p38-c005-claude-md-adr-reconciliation
/opsx:new p38-c006-dead-code-and-guard-classification
```

PLAN COMPLETE
