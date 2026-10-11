# c008 source-bound acceptance receipt

Status: **PASS**

Captured: `2026-09-15T20:23:22Z`

## Source identity

- Fabric base revision: `b60e15a31cfb54058f56e96b76fa4b48c5080172`
- Candidate artifact-set SHA-256:
  `9bd9c095a56ea4b3a93ddbf0f0d599c1f487ed165452bc96af36c3930527e02c`
- Exact candidate hashes: `source-binding.sha256`
- Runtime result: `runtime-receipt.json`
- PostgreSQL 17 image:
  `postgres@sha256:18cfe3ef5e6815560c98237d6216d1e5119702fb0f3894c8785dd58b8bbe5d73`
- Iggy image:
  `iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5`

The binding includes every c008 implementation, composition, fixture and runner
file, including untracked candidates. Unrelated workspace modifications are
outside this receipt.

## Runtime acceptance

`./scripts/run-cdc-integration.sh` exited 0. It created an owned PostgreSQL/Iggy
Compose project on dynamic loopback ports, ran the named ignored test exactly
once under the gateway composition crate, required the success marker and
removed all owned containers and volumes.

All ten machine-readable scenarios passed: committed CRUD, rollback/multi-row
atomicity, composite non-first non-UUID keys, column/fixed tenants, typed values
and unchanged TOAST, unsupported enrollment, crash after durable publication,
stable-ID deduplication, poison checkpoint blocking and schema-drift checkpoint
blocking.

## Local quality checks

All commands ran locally and exited 0:

| Gate | Command | Result |
|---|---|---|
| CDC unit tests | `cargo test --offline -p frf-postgres-cdc --lib --locked` | PASS, 14 tests |
| CDC strict lint | `cargo clippy --offline -p frf-postgres-cdc --all-targets --locked -- -D warnings -W clippy::pedantic` | PASS |
| Composition runtime | `./scripts/run-cdc-integration.sh` | PASS, ten scenarios |
| Composition strict lint | `cargo clippy --offline -p frf-gateway --test cdc_integration --locked -- -D warnings -W clippy::pedantic` | PASS |
| Gateway config | focused config tests | PASS, 17 tests |
| Rust format | `cargo fmt --all -- --check` | PASS |
| Shell syntax | `bash -n scripts/run-cdc-integration.sh` | PASS |
| Compose render | pinned integration overlay with disposable required variables | PASS |
| File size/diff hygiene | touched Rust/test line counts and `git diff --check` | PASS; largest file 424 lines |
| OpenSpec | `openspec validate pri-c008-cdc-commit-mapping --strict --no-interactive` | PASS |

No CI test invocation or result is evidence.

## Review disposition

The first fresh-context gpt-5.5 review against the gpt-6-astra producer blocked
with three critical findings and passed strict sycophancy screening. Each
finding changed the candidate:

- The live test moved from the CDC adapter to the gateway composition crate;
  the CDC manifest no longer depends on the Iggy adapter.
- PostgreSQL timestamp without time zone now has its own canonical value kind
  and a live assertion.
- Task 5 now has current source-bound runtime, quality, semver and finding
  resolution evidence. The task closes before the required resolution packet
  is assembled so the reviewer sees its final state.

The second review blocked with two critical findings. It found an unused
`anyhow` dependency in the CDC library and correctly challenged using the
shared commit LSN as a producer-local envelope offset. The dependency is
removed. The source LSN remains in the typed payload while the c007 broker
assigns and returns a unique durable position for every mutation; the live
composition now asserts that all six observed broker offsets are contiguous
and unique.

The third review blocked with two critical findings. Catalog publication
column metadata now accepts a null `attnames` value as PostgreSQL's all-columns
form while retaining explicit column-list filtering. The other finding was
procedural: this receipt and refinement state still described review as pending
after the task was complete. All substantive findings have current code and
runtime evidence, so this final candidate is recorded as converged for the
archive review.

The fourth review applied the 500-line source-file rule to the generated
`Cargo.lock`. That lockfile already exceeded 500 lines before c008 and Cargo
requires it as one generated file. The repository and KBD rule now state the
operable boundary explicitly: the limit covers every hand-authored source and
configuration file, while package-manager lockfiles retain their native
generated format. Both policy files are included in the candidate binding.

The fifth review found that the normal PostgreSQL no-old-tuple UPDATE path did
not make its identity-safety proof explicit. The decoder now requires matching
replica-identity metadata and verifies that every primary-key and tenant-routing
column is in the parser's key-column set before accepting that form. Missing or
inconsistent proof poisons the transaction, with a focused regression test.

The sixth review required the generated-lockfile exception in the c008
acceptance text itself, so the task and design now carry the same explicit
boundary as project policy. It also found raw invalid source values in error
Display output; those errors now report only the column and expected type. The
Artifact Refiner manifest uses paths relative to its persisted artifact folder.

The seventh review found that column-derived payload tenants were still placed
in an envelope carrying the fixed configured tenant. Publication now creates
the event channel with each validated mutation tenant, and the live test asserts
envelope/payload tenant equality for all column and fixed tenant events.

The eighth review found that the reference Compose profile enabled CDC before
any product-table publication migration exists. The profile now defaults CDC
off, accepts empty CDC enrollment and source-epoch inputs while disabled, and
fails gateway validation if an operator enables it without those values. The
profile's normal PostgreSQL, Iggy, identity and public-origin credentials remain
required because their services still run when CDC is disabled. The environment
template and runbook make the migration-before-enable sequence explicit.

The ninth review reported that hyphens were absent from the source-epoch
validator, but its own packet contains `"._:-"` and the source-bound live run
used the hyphenated `c008-core-v1` epoch successfully. It also read “disabled
inputs” as all Compose inputs; the evidence above now names the CDC-specific
inputs and the profile credentials separately.

The tenth review found that Compose's deliberately empty `CDC_TENANT_ID` still
reached UUID parsing before disabled-mode validation. Optional UUID loading now
normalizes empty and whitespace-only values to unset, while malformed non-empty
values still fail with the variable name.

The eleventh review found that gateway startup checked source-epoch presence but
left syntax validation until adapter enrollment. Both layers now call one public
CDC validator, with gateway and adapter regressions. Per-event stable IDs and
source positions were also removed from publication debug logs.

The twelfth review found that the live column and fixed tenant modes used the
same UUID and that a generic broker was not asked to ensure derived channels.
The fixture now uses distinct tenant UUIDs and asserts both independently; the
publisher ensures each validated tenant channel before durable publication.

The thirteenth fresh-context review returned **PASS** with zero critical,
warning or suggestion findings. Its anti-sycophancy screen also passed.
