ASSESSMENT: issue-triage-and-architecture-audit
Project: flint-realtime-fabric
Date: 2026-09-14
Codebase baseline: Workspace compiles clean and every default test passes, but the three
open issues are all unfixed, one of them blocks the other two, and the project's own
BLOCKING constraint set has two failures — one introduced by this session's commit.
Cross-tool progress: NONE recorded in `progress.json` (phase created today, 0/0).

---

## IMPLEMENTATION STATUS

- **G1 — issue #6, Iggy handshake:** MISSING. No fix attempted. No available server
  completes a handshake with the pinned client. Client **0.6.203** (`Cargo.lock:4501`,
  GQAdonis fork `d34b9c96`); `iggyrs/iggy:latest` is **0.4.214**; the digest pinned in
  `k8s/overlays/ssr/iggy.yaml:41` is **0.8.13**. Both servers were run on 2026-09-14 and
  each logged `Clients: 0, Messages processed: 0` for its whole uptime while the test hung
  until killed. Ruled out: TCP reachability, credentials (`iggy:iggy` authenticates — CLI
  `ping` returns in 2.44 ms over the *same* host→container path the test uses), and port
  forwarding. **The mechanism was not determined.** Do not carry a protocol-version story
  into planning as though it were established.

- **G2 — issue #2, stable channel id:** PARTIAL. The code fix landed in `788637a`: CDC and
  the gateway fixture now use `ChannelId::WELL_KNOWN_ENTITIES` instead of a per-run
  `ChannelId::new()`, `publish` self-heals via a shared `create_stream_and_topic`, and the
  fixture failure is fatal rather than a swallowed `warn!`. **Receipt was never
  demonstrated.** The new guard has never been observed to fail and the sabotage step did
  not run. Blocked by G1.

- **G3 — issue #7, non-UUID tenantId:** MISSING. `tests/e2e/smoke_test.sh:48` sends
  `"tenantId": "e2e-tenant"`; `parse_tenant_id` (`grpc_service.rs:64-68`) rejects any
  non-UUID with `invalid_argument`. The sibling TS/Go/C# clients already use a valid UUID.

- **G4 — CLAUDE.md ↔ ADR reconciliation:** MISSING. See SPEC GAP SUMMARY.

- **G5 — vacuous-guard / dead-code sweep:** PARTIAL. Three vacuous guards were found and
  removed earlier in this session. Four `#[ignore]`d tests remain (exact count verified),
  none of which has ever executed in the default suite. One orphaned file confirmed.

- **G6 — file-size regression:** MISSING. `crates/frf-gateway/src/main.rs` is 504 lines,
  over the hard 500-line cap. It was 487 before `788637a`. **This session caused it.**

---

## CROSS-TOOL PROGRESS

NONE — `progress.json` was created today by `kbd-new-phase` with `changes_total: 0`. No
other tool has recorded work against this phase.

Adjacent, and relevant to planning: **PR #5** ("wip: shape facade rework — rescued
uncommitted work", 39 files) merged to `main` at **2026-09-14T09:30:56Z**. Its own body
says *"Not verified to build — review/validate in CI before merging."* It was merged
without that validation. Note the request was **unsatisfiable as written**: CI validation
of behaviour is forbidden by this repo's non-negotiable testing policy (AGENTS.md:14-28,
CLAUDE.md). The only commit after it on `main` is `788637a`.

Encouraging counter-evidence: despite that warning, `cargo test -p frf-app` now runs 55
tests green, and the 44 test attributes in the `frf-app/src/shape/` module landed by that
PR all execute. The shape work is **not** the unproven surface it appeared to be.

---

## SPEC GAP SUMMARY

- **CLAUDE.md "Open Decisions" is stale on all four rows.** `:102` still reads
  "CRDT | Loro **or** automerge-rs — **OPEN, decide before Phase 3**", but ADR-001 accepted
  Loro on 2026-06-19 and `Cargo.toml:146` ships `loro 1.13.1`. ADR-003 pinned the
  FFI/codegen toolchain. The project is at phase-36+; these cannot still be open.

- **Workspace tree omits five crates that exist on disk:** `frf-did`, `frf-p2p`,
  `frf-shape-electric`, `frf-wallet`, `uniffi-bindgen`.

- **`CLAUDE.md:256` says Dart uses flutter_rust_bridge.** ADR-003 overrode this; FRB's
  parser panics on `#[uniffi::export]`. Dart uses `uniffi-bindgen-dart`.

- **OpenSpec ledger has drifted badly.** 126 change directories sit outside `archive/`
  against 89 archived. At least **42 are complete but unarchived** (all tasks checked);
  the split of the remaining 84 between partial and no-`tasks.md` is **approximate** — my
  counting script errored on roughly a third of iterations and I am not presenting that
  breakdown as precise. `openspec list` cannot parse them at all, emitting
  `Rules for 'proposal' must be an array of strings` repeatedly. This violates P1.

> **CORRECTION — 2026-09-14, after investigation.** The claim above that "`openspec list`
> cannot parse them at all" is **wrong**, and the P1 diagnosis was wrong in kind. The CLI
> parses the ledger fine: `openspec list 2>/dev/null` emits 127 clean rows and
> `openspec validate p0-c001-workspace-restructure` exits 0. The repeated warning went to
> **stderr** and was cosmetic — it meant the CLI was *ignoring this project's custom rules*,
> not failing to read the changes.
>
> Two genuinely separate defects were behind it:
>
> 1. **One unquoted YAML line.** `openspec/config.yaml:57` read
>    `- Include the dependency-rule impact: which layers are touched and why it is safe`.
>    The bare `: ` made that list entry parse as a *mapping*, so Zod's
>    `z.array(z.string())` rejected the whole `proposal` array and silently dropped all five
>    rules (`project-config.js:267`). Quoting the line fixed it: **120 warnings → 0**, and
>    both `proposal` and `tasks` now load as 5 strings each.
> 2. **No spec deltas anywhere in the active pile.** 0 of 126 active changes have a
>    `specs/` directory, against 87 of 89 archived ones. *That* is why every active change
>    failed `openspec validate` — one identical error naming the remedy
>    (`skip_specs: true` in the change's `.openspec.yaml`).
>
> Also corrected: **the checkbox counts are stale, not a work backlog.** 12 of 12 sampled
> "in progress" changes reference files that exist — e.g. `p0-c002-frf-domain` reads 0/6
> while `crates/frf-domain/Cargo.toml` and `src/ids.rs` both exist and compile.
> `p14-c002-gateway-dev-no-auth` looked like a real backlog only because
> `crates/frf-gateway/src/config.rs` became the *directory* `config/` in a later refactor;
> `dev_no_auth()` is implemented at `config/mod.rs:92`. The CLI's own status column reads
> the same stale boxes, so "46 in progress" is a checkbox state, not a work state.
>
> One process note: a `--json` flag on `openspec archive` is **not** a dry run. I invoked
> it as a "non-destructive probe" and it archived `p0-c001-workspace-restructure` for real;
> it was reverted via git and `git status --short openspec/` returned to 0 lines before any
> further work.

- **Why this matters, concretely.** Stale naming documentation is what produced issue #2's
  incorrect table: a reader followed `channel.rs`'s `tenant-` helpers, which no code had
  called since `26e4dfc`. Doc drift here has already cost one misdiagnosis that stood for
  two months.

---

## BUILD HEALTH

- **build check: PASS** — `cargo check --workspace`, exit 0.
- **clippy: PASS** — `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic`,
  exit 0, no diagnostics (constraint R2's exact command).
- **format: PASS** — `cargo fmt --check --all`.
- **tests: PASS** — `cargo test --workspace`, exit 0. ~200 tests across 30 binaries, zero
  failures, **4** `#[ignore]`d.
- **known violations:** R5 (file size) and P1 (spec hygiene) — see CONSTRAINT CHECK.
- **test coverage: PARTIAL.** High unit coverage; every *integration* path is unproven.
  All four ignored tests require infrastructure, and G1 establishes that at least three of
  those four cannot currently run at all.

---

## CONSTRAINT CHECK

Against `.kbd-orchestrator/constraints.md` (BLOCKING fails the QA gate).

| ID | Constraint | Result |
|---|---|---|
| A1 | Dependency direction | **PASS** — `frf-domain` has zero `frf-*` deps; `frf-app` imports only `frf-domain` + `frf-ports` |
| A2 | One port per adapter | **PASS** — only `frf-media-str0m` implements two, the deviation already documented in CLAUDE.md/ADR-005 |
| A3 | Proto frozen | PASS — no `proto/flint/v1/*.proto` edits in the working tree |
| R1 | Compiles | **PASS** |
| R2 | Clippy pedantic | **PASS** |
| R3 | No library `unwrap`/`expect` | PASS — denied workspace-wide; test files carry explicit allows |
| R4 | Formatted | **PASS** |
| R5 | No file over 500 lines | **FAIL** — `crates/frf-gateway/src/main.rs` at **504**. Sole offender workspace-wide. Four files sit within 25 lines of the cap: `frf-gateway/src/config/mod.rs` (498), `frf-sdk-rust/src/shape.rs` (495), `frf-app/src/shape/tests.rs` (484), `frf-app/src/shape/lease.rs` (477) |
| S1 | No hardcoded secrets | **PASS** — no credential-shaped literals in committed config or source |
| S2 | No auth bypass in release | **PASS** — `compose.yml` not only omits `DEV_NO_AUTH`/`dev-endpoints` but documents at `:8` and `:31` why it must never be added |
| S3 | No sensitive logging | PASS — the CDC log line added in `788637a` deliberately logs `channel_id` and `path`, never `tenant_id` |
| S4 | Tenant isolation | PASS — `PublishUseCase` asserts claim/envelope tenant equality; `SubscribePipeline` filters cross-tenant envelopes |
| P1 | Valid spec delta / archive hygiene | **FAIL** — 126 unarchived changes, ≥42 complete; `openspec list` cannot parse them |

**AGENTS.md violations: NONE.** Stated from reading the file, not from a pattern match —
my first grep for a "Never Do" section returned empty because **no such section exists**.
AGENTS.md is 28 lines and carries exactly one policy: local-integration-only testing. No
work in this session dispatched CI to test anything.

**constraints.md violations: R5 and P1** (both BLOCKING).

---

## DEAD CODE & UNPROVEN GUARDS (G5 detail)

- **`crates/frf-gateway/src/ws.rs` is orphaned.** 20 lines exporting `ws_echo`, which has
  exactly one occurrence in the workspace — its own definition. `lib.rs` declares twelve
  modules; `ws` is not among them, and no `mod ws;` exists anywhere. The file does not
  compile into the binary.

- **`fetch_and_cache` is NOT dead** — my first sweep flagged it, but it has two in-file
  callers (`frf-identity-ory/src/jwks.rs:58`, `:72`). The sweep excluded the defining file
  and was wrong. Recorded so the next reader does not repeat the deletion.

- **Four `#[ignore]`d tests, none ever executed:**
  `frf-broker-iggy/tests/publish_subscribe.rs:29` and `:85` (blocked by G1),
  `frf-postgres-cdc/tests/cdc_integration.rs:16` (needs local Postgres 17 + logical
  replication), `frf-gateway/tests/subscribe_mux.rs:10` (needs Iggy + Keto + flint-gate).
  Treat each as a hypothesis until observed failing.

---

## PROCESS-STATE DEFECTS FOUND WHILE ASSESSING

- **`.kbd-orchestrator/position-reminder.txt` is stale.** It still names
  `phase-36-sovereign-sfu-ice-linux-fix`, "Step: 2 of 3", `execute_ready`, and
  `Next command: /kbd-apply p36-c002-decode-run-and-flip` — a change that was **withdrawn**
  and a phase archived at 3/3. Every KBD skill instructs agents to read this file **first
  every turn**, so it will misdirect every future session until regenerated. The
  authoritative waypoint (`revision: 1`, current `updatedAt`) was used instead.

- **The phase has no `handoffs/` directory**, so the stage gate takes the legacy
  warn-and-pass path.

---

## GOAL PROGRESS

| Goal | Status | Reason |
|---|---|---|
| G1 — unblock local integration (#6) | **NOT MET** | No fix attempted; mechanism still undetermined |
| G2 — prove the #2 fix | **NOT MET** | Code landed; receipt never demonstrated; sabotage never run. Blocked by G1 |
| G3 — fix #7 | **NOT MET** | No change made |
| G4 — reconcile CLAUDE.md/ADRs | **NOT MET** | Drift catalogued here, not corrected |
| G5 — vacuous-guard sweep | **PARTIAL** | 3 removed earlier; 4 ignored tests + 1 orphaned file remain |
| G6 — file-size regression | **NOT MET** | `main.rs` still 504 lines |

---

## OPEN QUESTIONS FOR PLAN

1. **G1 is the critical path and its cause is unknown.** Planning must not assume a
   version mismatch is the mechanism — that is an untested hypothesis. The first change
   should be diagnostic (capture the client-side handshake), not corrective.
2. **Is any Iggy server known to work with fork `d34b9c96`?** If none exists, G1 may
   require changing the client pin rather than the server, which widens scope considerably.
3. **Should the 126-change ledger be archived in bulk or audited individually?** Bulk
   archiving is fast but would bury the ~84 whose completion state I could not verify.
4. **Is `ws.rs` deletable, or is it a stub for planned work?** Deleting is the default,
   but it predates this session.
5. **G6 is self-inflicted and cheap** — extract the fixture helper from `main.rs`. It
   should not compete for priority with G1.

---

## SYCOPHANCY REVIEW

`detect_sycophancy` (standard, detect_only) returned **0.0** with no classifications.
Record: `sycophancy/assess-<timestamp>.json`. Per the protocol's table (< 0.3) this was
written as-is.

Self-check against the protocol's own criteria: S-02 — the architecture was evaluated
against the constraint file rather than accepted, and A1/A2/S2 pass on cited evidence, not
assertion. S-03 — six goals are NOT MET or PARTIAL, two BLOCKING constraints fail, and one
failure was caused by this session's own commit. S-06 — no "clearly"/"obviously"; the one
place I lacked a mechanism (G1) is labelled undetermined rather than narrated.

---

## ADVERSARIAL REVIEW — NOT PERFORMED

Step 8 of `/kbd-assess` calls for `/adversarial-review --mode artifact assess` on this
file. **It did not run.** Recorded explicitly because an unrun review that leaves no trace
is indistinguishable from one that found nothing.

Two independent reasons:

1. **The model gateway is unreachable.** `.kbd-orchestrator/model-preflight.json` reports
   `status: "ok"` with gateway `http://localhost:4000/v1`, but a live probe of
   `/v1/models` returns HTTP `000` (no response). **The preflight cache is stale** — it is
   valid for 24 h and was written while the gateway was up. Any skill trusting that cached
   `ok` without probing will believe review capability exists when it does not.
2. **Roles are not distinct.** The same file reports `distinct_models: 2` across three
   roles (judge `k3`, critic `MiniMax-M3`, generator `kbd-frontier`). Two distinct models
   cannot fill three roles, so the judge would have collided with the producer even had
   the gateway answered — the `degraded` condition in substance, whatever the status field
   says.

Per the skill ("Never block the stage on preflight status"), the stage proceeded. The
consequence carries forward: **no CRITICAL/WARNING findings were generated for this
assessment**, so downstream stages must not read their absence as a clean bill of health.
If the gateway is restored, re-vet with
`/adversarial-review --mode artifact assess` against this file before `/kbd-execute`.

ASSESSMENT COMPLETE
