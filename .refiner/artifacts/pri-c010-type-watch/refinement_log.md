# Refinement log — pri-c010-type-watch

## Iteration 1 — deterministic implementation

**Specify:** Implement ADR-010 as a versioned authorized type watch without
changing v1, using c007 broker replay, c008 committed canonical mutations, and
c009 durable projection state.

**Plan:** Add canonical domain types and a dedicated broker watch-source port,
store lossless typed projection rows transactionally, join an atomic projection
snapshot to replay at the next broker offset, sign scope-bound checkpoints, and
serve the generated v2 tonic API with bounded per-subscriber delivery.

**Execute:** Implemented domain/port/adapter/application/transport composition,
deployment configuration, operator documentation, generated-client compile
proof, focused race/recovery/security regressions, and a source-bound local
PostgreSQL/Iggy/SurrealDB/tonic runner.

**Reflect:** The first live run exposed a mismatch between the stored canonical
entity type and a table-only SurrealDB snapshot filter. The filter now compares
the complete `schema.table@projection` identity; the rerun passed all six live
scenarios and removed its owned resources. Focused tests also caught and cover
checkpoint tampering, snapshot/subscription timing, cancellation during awaited
authorization, lag, revocation, malformed canonical values, and key-only delete.

**Persist:** Passing run `20260916152131-47074` and its initial candidate hash
`fab1f2876236be67d1628ddedd3d48a8aa9347302bf5a3249c5b3c5e8c6ea2fa`
are recorded in c010 evidence. Final source binding, quality gates, and
independent review remain pending.

## Iteration 2 — adversarial convergence

**Specify:** Close every security, concurrency, recovery, packaging, and
configuration defect found by independent diff review while preserving the
frozen v1 contract and source-bound acceptance proof.

**Plan:** Resolve findings one boundary at a time, add a regression for every
real defect, regenerate the real-stack receipt after each source change, and
resubmit a complete manifest-bound packet until the judge reports no findings.

**Execute:** Replaced signed plaintext checkpoints with AES-256-GCM; hardened
snapshot and live authorization, token expiry, terminal ordering, bounded
delivery, cancellation, source-epoch recovery, filtered-row resume, adapter
errors, retention configuration, disabled-service composition, and review
packet coverage. Added regressions for each corrected boundary.

**Reflect:** The final independent round passed with zero critical, warning, or
suggestion findings. All ten stream security/ordering tests, five application
watch tests, 59 gateway tests, deployment-profile checks, strict Clippy,
OpenSpec, file-size, v1 hash, and real adapter acceptance passed locally.

**Persist:** Final source-bound run `20260916223916-49993`, candidate hash
`e76a442f83ab1d1a000e9d7125826e1e263bf7baedaf758d365b131d4d674962`,
and the clean round-23 verdict are recorded in c010 evidence.
