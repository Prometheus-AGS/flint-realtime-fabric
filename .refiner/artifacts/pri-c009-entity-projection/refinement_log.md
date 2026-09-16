# Refinement log — pri-c009-entity-projection

## Iteration 1 — deterministic implementation

**Specify:** Replace the gateway's production in-memory entity read store with a
durable projection fed by c008's committed canonical CDC stream, while retaining
the existing v1 entity RPCs and per-event authority lifetime.

**Plan:** Add projection/cursor operations to the entity-store port, implement a
dedicated SurrealDB adapter, consume the well-known Iggy entity channel, combine
CDC/projector readiness, and prove state, overlap, restart, delete, and revocation
with owned PostgreSQL, Iggy, and SurrealDB fixtures.

**Execute:** Implemented the adapter, projector, gateway composition, deployment
profile, operator documentation, source-bound runner, and focused regressions.

**Reflect:** Formatting, diff hygiene, the production profile renderer, entity
authorization tests, projector/config tests, all-target compilation, and strict
Clippy pass. Clippy led to a boxed projection result, watcher-map type aliases,
and an extracted entity-pipeline startup helper. The finalized live run passes
all six acceptance scenarios and removes its owned resources.

**Persist:** Candidate artifact hash
`bf36b30267b69ac6f60f433ce7e69ff157bb0047161dc5ff12b44627993a38de`
and run `20260916132836-97391` are recorded under the c009 evidence directory.
Independent review is pending.

## Iteration 2 — review corrections

The first complete-diff review required the deployment compatibility statement
to distinguish gateway validation from the reference full Compose profile. The
reference profile intentionally keeps SurrealDB in its topology while CDC is
disabled and therefore requires its renderer inputs; custom CDC-disabled gateway
deployments still use the in-memory entity store. The runbook now uses the exact
Compose service names. An earlier incomplete-diff review also prompted an
explicit owned `String` conversion at the gRPC permission boundary.

## Iteration 3 — catch-up readiness and failure cleanup

The resolution review found that projector readiness preceded retained-backlog
catch-up, pre-commit SurrealDB errors did not all cancel their transactions, and
non-finite floats silently became JSON null. The projector now subscribes first,
captures Iggy's high-water offset, and stays unready until that event is applied
and acknowledged. Transaction preparation has one cancellation path for every
error, and non-finite floats fail as invalid payloads. The live fixture creates
backlog before projector start and proves immediate v1 readability at readiness;
it also proves a failed update transaction does not poison the following insert.

## Iteration 4 — one-port broker capability

The next review correctly rejected a projection-specific adapter trait because
Iggy already implements the `LogBroker` port. High-water lookup now belongs to
that existing port with a fail-closed default. Iggy and the gateway's configured
broker override it; unrelated brokers remain source-compatible and cannot claim
projection readiness support. The final runbook reference now uses the exact
`iggy` Compose service name.

## Iteration 5 — credential metadata

The passing review left two warnings because SurrealDB credentials appeared in
container argv. The pinned image documents `SURREAL_USER`, `SURREAL_PASS`, and
`SURREAL_PATH`; both production and fixture Compose files now use those
environment inputs and keep the password out of the command array. The profile
renderer and all eight live scenarios pass with that composition.

The next packet omitted `configured_broker.rs` from the declared c009 file set
and therefore missed its existing Iggy high-water delegation. The file is now
source-bound and the gateway binary test exercises fail-closed high-water access
through the configured wrapper; the live receipt binds that complete composition.

The following review found that the scan-based watch guard emitted the
revocation error but waited for another upstream item before ending. The guard
now owns a pinned checked stream and, after per-event authorization detects
revocation, returns `None` from its next poll before polling the idle source. A
timeout regression keeps the source pending forever. Snapshot installation now
also cancels its transaction on every preparation error, matching incremental
projection cleanup discipline.

A later review incorrectly treated `EntityStore` as a native async trait and
claimed it was not dyn-compatible. The port is already annotated with
`#[async_trait]`; a fresh offline all-target gateway check compiled the
`Arc<dyn EntityStore>` composition successfully and records direct
disconfirming evidence without changing correct code.

The subsequent passing review reported one serializer asymmetry: `delete` was
encoded but not decoded. Delete projection normally removes rows, but snapshot
input could still expose that mismatch. The parser now accepts `delete`, and a
focused test covers round trips for every persisted operation.

The final packet review correctly found stale proposal status and an obsolete
adapter path; both now describe the completed `frf-projection-surreal` work.
Its two dependency findings were false: the app already directly declares
`futures-util`, the gateway already directly declares test-only
`tokio-postgres`, and the all-target gateway check compiles both paths.
