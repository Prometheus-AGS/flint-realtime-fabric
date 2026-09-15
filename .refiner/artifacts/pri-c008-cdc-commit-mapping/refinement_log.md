# Refinement log — pri-c008-cdc-commit-mapping

## Iteration 1 — deterministic implementation

**Specify:** Replace row-at-a-time string CDC with the accepted c006 typed
source contract and the c007 durable replay boundary.

**Plan:** Validate explicit enrollment from PostgreSQL catalogs, canonicalize
typed row/key data, buffer transactions through COMMIT, publish in order, then
advance applied LSN. Prove it with an owned PostgreSQL 17 plus Iggy fixture.

**Execute:** Added enrollment/catalog validation, canonical models, transaction
assembly, stable IDs, commit-controlled publication, gateway/deployment config,
operator documentation and the local live runner.

**Reflect:** Formatting, 11 unit tests, pedantic clippy and the full live fixture
pass. The fixture observes PostgreSQL slot positions around a deliberately
injected failure after durable broker publication; restart produces two unique
mutations and no loss. Unsupported mappings, poison values and schema drift
fail before checkpoint advancement. Partitioned relations were removed from the
accepted enrollment surface because their publication routing is not qualified
by this slice.

**Persist:** Candidate artifact hash
`9bd9c095a56ea4b3a93ddbf0f0d599c1f487ed165452bc96af36c3930527e02c`
and run `20260915202300-19992` are recorded under the c008 evidence directory.

## Iteration 2 — first-review corrections

The isolated judge found three blockers: the adapter-local integration test
created an adapter-to-adapter development dependency, timestamp without zone
was mislabeled as time, and task 5 was still open. The fixture now belongs to
the gateway composition crate, the CDC manifest no longer references Iggy,
timestamp without zone has a distinct canonical variant, and a new source-bound
live run passes. Task 5 remains open until the resolution review passes.

## Iteration 3 — second-review corrections

The second judge found the CDC crate's unused `anyhow` dependency and a
misleading producer-local envelope offset shared by mutations from one commit.
The dependency is removed. CDC source position remains solely in the payload;
the envelope starts without a producer position, and the broker assigns and
returns a unique durable offset. The live test now asserts contiguous unique
broker offsets across all six core mutations.

## Iteration 4 — catalog nullability and convergence state

The third judge required the all-columns publication form to tolerate nullable
`pg_publication_tables.attnames`. Catalog loading now treats null as all physical
columns and continues to filter an explicit list. The receipt and state now
describe the completed task rather than the state before its archive review.

## Iteration 5 — generated lockfile policy

The fourth judge applied the 500-line source-file limit to `Cargo.lock`. The
pre-existing generated lock already exceeds that size and Cargo requires one
native lockfile. Project and KBD wording now exempts machine-generated package
locks while retaining the limit for every hand-authored source and config file.
The fifth review also required an explicit identity proof when PostgreSQL omits
an UPDATE old tuple. The decoder now checks matching replica identity and the
presence of every key and tenant-routing column in parser metadata, failing
closed when that proof is incomplete.
The sixth review required the lockfile exception in c008's own acceptance note,
removed raw invalid values from error Display output, and replaced local
absolute manifest paths with portable paths relative to the artifact directory.
The seventh review found an envelope/payload tenant mismatch for column tenant
mode. Publication now assigns the validated mutation tenant to each envelope,
and the live fixture asserts equality for both accepted tenant modes.
The eighth review found that the reference profile enabled CDC against an empty
explicit publication. Compose now defaults CDC off; operators enable it only
after reviewed migrations add all enrolled tables.
The ninth review's epoch finding contradicted its packet, which already includes
hyphen in the safe character set and passed a live hyphenated epoch. Its Compose
finding exposed ambiguous evidence wording; the receipt now distinguishes empty
CDC enrollment inputs from the credentials required by the always-running stack.
The tenth review found that an empty optional CDC tenant still entered UUID
parsing. Config loading now normalizes empty optional UUIDs to unset and retains
fail-fast parsing for malformed non-empty values.
The eleventh review found split source-epoch validation and unnecessary stable
event identifiers in logs. Gateway and adapter now share one epoch validator,
and publication logs retain only the assigned broker offset.
The twelfth review found that fixed and column tenant fixtures were
indistinguishable and that generic brokers were not asked to ensure derived
channels. The live fixture now proves two tenant UUIDs, and the publisher
ensures each tenant channel before publishing.

The thirteenth fresh-context review returned `PASS` with no findings after the
source-bound live run passed all ten scenarios.

Result: `PASS`; zero blocking implementation violations remain and all prior
independent findings are resolved.
