# c010 completion receipt

Status: **PASS; independently reviewed with no remaining findings**

## Source-bound runtime

- Command: `./scripts/run-entity-type-watch-integration.sh`
- Run: `20260916223916-49993`
- Candidate artifact-set SHA-256:
  `e76a442f83ab1d1a000e9d7125826e1e263bf7baedaf758d365b131d4d674962`
- Result: six of six scenarios passed; exit `0`
- Cleanup: owned Compose containers, network, and volumes removed
- Tracked receipt: `runtime-receipt.json`

The c009 projection regression also passed all eight scenarios after verifying
upgrade behavior for a v1 snapshot without typed metadata. Its run was
`20260916175923-42953`, source binding
`10287f2e6b165342ca4e87a4f992c36adda7a8b56cb34e56509792b736f0254b`.

## Local quality matrix

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| `./scripts/test-deployment-profiles.sh` | PASS |
| `./scripts/check-file-size.sh` | PASS; 311 files checked, none over 500 lines |
| `openspec validate pri-c010-type-watch --strict --no-interactive` | PASS |
| `cargo clippy --offline -p frf-domain -p frf-ports -p frf-postgres-cdc -p frf-broker-iggy -p frf-watch-broker -p frf-app -p frf-projection-surreal -p frf-proto -p frf-gateway --all-targets --locked -- -D warnings -W clippy::pedantic` | PASS |
| `cargo test --offline -p frf-broker-iggy -p frf-app -p frf-watch-broker -p frf-projection-surreal -p frf-proto --locked` | PASS |
| `cargo test --offline -p frf-gateway --lib --locked` | PASS; 59 tests |
| `cargo test --offline -p frf-app --test entity_type_watch --locked` | PASS; 3 tests |
| `cargo test --offline -p frf-app --test entity_type_watch_race --locked` | PASS |
| `cargo test --offline -p frf-app --lib entity_type_watch::stream --locked` | PASS; 10 ordering/backpressure/security/recovery tests |
| `cargo check --offline -p frf-gateway --bin frf-gateway --locked` | PASS |
| v1 protobuf SHA-256 baseline | PASS; all six hashes unchanged |

All tests ran locally. No CI test result is used as evidence.

## Review corrections

Round 1 reported three critical findings and one warning. The hardcoded fallback
checkpoint key was removed, terminal ordering was corrected with revocation-safe
regressions, and completion evidence was persisted. The alleged duplicate entity
service registration was a namespace misread: the runtime registers separate
`flint.v1.EntityService` and `flint.v2.EntityService` names; explicit aliases now
make this visible in the composition source. The detailed disposition is in
`review/pri-c010-type-watch/resolution-round-1.md`.

Later review found that authenticated plaintext did not meet the opaque-token
contract, identified a narrower revocation/frame interleaving, and caught a
lossy timestamp-without-zone conversion. Checkpoints now use AES-256-GCM with
random nonces and authenticated metadata, delivery rechecks preempting terminals
after frame receipt, and the unsupported canonical type fails with `DATA_LOSS`.
The detailed disposition is in `resolution-round-3.md`.

Round 4 identified unbounded initial-snapshot backpressure and requested a
subscription-authority recheck immediately before mutation enqueueing. Both
were added with a stalled-snapshot regression. Its compile-error finding was a
misread of a nested async block whose inferred return type is Boolean. The
detailed disposition is in `resolution-round-4.md`.

Round 5 identified that snapshot delivery trusted the projection's tenant/type
filter and flagged exhaustive public enums. Snapshot rows now receive an
independent app-layer scope check exercised with an intentionally overbroad
projection fixture, and all three public watch enums are non-exhaustive. The
detailed disposition is in `resolution-round-5.md`.

Round 6 alleged that `EntityRow::entity_type` did not exist. The field is
declared and populated in the submitted model; the focused adapter compile/test,
strict Clippy matrix, and runtime path all pass. The detailed source citation is
in `resolution-round-6.md`.

Round 7 found that live delivery did not compare the mutation source epoch with
the configured watch epoch. The producer now emits a terminal
`SourceEpochChanged` resnapshot frame before checkpoint or payload delivery, with
a focused regression. The detailed disposition is in `resolution-round-7.md`.

Round 8 found that snapshot `Accepted` exposed its barrier checkpoint before
the covered rows and that acceptance A retained an older test-log hash. Snapshot
admission now uses the pre-snapshot checkpoint, a disconnect/resume regression
proves offset zero is replayed, and the evidence hash matches the tracked
receipt. The detailed disposition is in `resolution-round-8.md`.

Round 9 repeated the earlier nested-Boolean compile misread. The nested result
now uses a named `ProjectionPreparation` enum, making the duplicate-to-
`ProjectionApply::Duplicate` mapping explicit. Strict adapter Clippy and tests
pass. The detailed disposition is in `resolution-round-9.md`.

Round 10 found a mismatch between configurable advertised retention and Iggy's
fixed topic expiry. Gateway composition now passes the same validated retention
to both, with default/custom/invalid adapter regressions. The detailed
disposition is in `resolution-round-10.md`.

Round 11 found that initial frames were not reauthorized and that non-lag
terminal frames could repeat. Initial delivery now rechecks identity and
subscription authority before every enqueue, and all control frames terminate
after one emission. The detailed disposition is in `resolution-round-11.md`.

Round 12 found that filtered deliveries could advance checkpoints without a
per-event subscription recheck. The recheck now precedes checkpoint issuance
and scope filtering, with an out-of-scope revocation regression. The detailed
disposition is in `resolution-round-12.md`.

Round 13 alleged a missing `IggyBroker` re-export that is present at the crate
root and correctly identified silent fallback for malformed watch numbers. All
three numeric variables now fail startup with their names when malformed. The
detailed disposition is in `resolution-round-13.md`.

Round 14 found `anyhow` in the adapter constructor and visible snapshot-barrier
progress from filtered rows. The adapter now returns `IggyBrokerError`, and the
public barrier derives only from authorized in-scope rows while global progress
remains encrypted. The detailed disposition is in `resolution-round-14.md`.

Round 15 found that the base Compose file required a checkpoint key even when
CDC was disabled and alleged that the broker adapter rejected valid events from
other tenants. The base file now permits an absent key and has a regression for
that mode; enabled production profiles and gateway startup still require it.
The adapter check is envelope integrity, not subscriber filtering, and focused
regressions prove valid events from different tenants pass. The detailed
disposition is in `resolution-round-15.md`.

Round 16 alleged that the two-argument `ConfiguredLogBroker::for_full` caller
had no matching implementation. The implementation was already present and
compiled, but its file was missing from the change manifest and therefore from
the review packet. The manifest now includes it, together with the related Iggy
adapter error files found by a complete manifest audit. The detailed disposition
is in `resolution-round-16.md`.

Round 17 found that a fully filtered snapshot could complete at the global
projection cursor and later skip a newly granted row on resume. Zero-row
snapshots now retain the pre-snapshot completion position, and a grant/resume
regression proves the historical row is delivered. The detailed disposition is
in `resolution-round-17.md`.

Round 18 refined that recovery boundary: it must be immediately before the
earliest withheld retained row, rather than unconditionally broker beginning.
The producer now records that precise encrypted position, and the regression
expires all earlier history to prove resume requests offset 5. The detailed
disposition is in `resolution-round-18.md`.

Round 19 found that token expiry did not preempt an already awaited
authorization or initial enqueue. Deadline races now cover every live and
initial authorization path and initial send, with two blocked-authorization
regressions proving no protected frame escapes. The detailed disposition is in
`resolution-round-19.md`.

Round 20 passed with one warning: a stray checkpoint key could register an
inert v2 watch service while CDC was disabled. Service registration now depends
on validated CDC enablement and no longer invents empty enrollments or a dummy
epoch. The detailed disposition is in `resolution-round-20.md`.

Round 21 passed with one warning: replay retention reached full-profile broker
construction even when CDC was disabled, while its minimum was conditional.
The retention minimum is now unconditional, with a disabled-CDC configuration
regression. The detailed disposition is in `resolution-round-21.md`.

Round 22 found a revocation race between awaited object authorization and live
enqueue, plus stale retention documentation. A final identity/subscription
check now preempts every constructed live frame, its regression passes, and the
documentation matches unconditional validation. The detailed disposition is in
`resolution-round-22.md`.

Round 23 returned `PASS` with no critical, warning, or suggestion findings.
