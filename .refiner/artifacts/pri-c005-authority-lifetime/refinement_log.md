# Refinement log — pri-c005-authority-lifetime

## Iteration 1

Result: **DETERMINISTIC PASS; REVIEW PENDING**.

| Constraint | Result | Evidence |
|---|---|---|
| A | PASS | Seven real Gate/Keto/Fabric identity, tenant, object and revocation scenarios completed with an exit-0 source-bound receipt. |
| B | PASS | Eight generic subscription cases and ten protected shape cases pass; the largest measured server cancellation is 1,500 ms. |
| C | PASS | Removed signing keys expire from trust; stale tuple refills are rejected after invalidation. |
| P1 | PASS | Cargo resolves the exact Cedar 4.11.2 family and its four policy parity/negative tests pass. |
| P2 | PASS | Strict all-target Clippy, 150 package tests, format, Shellcheck, syntax, file-size, diff and strict OpenSpec validation pass locally. |

The owned runtime first exposed two real Keto v0.12 mismatches that HTTP mocks
had hidden: relation mutations use `/admin/relation-tuples`, and a denied check
returns HTTP 403. The adapter and regression tests now reflect the selected
runtime. Clippy then found three structural issues in the new code, which were
resolved before this checkpoint.

The next step is the isolated post-QA adversarial review. Convergence remains
open until its critical and warning findings are resolved.

## Iteration 2

Result: **REVIEW FINDINGS RESOLVED; RESOLUTION REVIEW PENDING**.

The independent judge reported two critical findings and one warning. Task 5
remained open while review was in progress; it is now checked after the final
receipt and strict validations passed. The runtime now generates a random
Postgres password per run. Keto tuple deletion uses reqwest query serialization,
with a regression test covering spaces, plus signs and `@` in tuple fields.

The first post-fix runtime attempt then exposed that credentials minted before a
cold nine-minute build can expire before the test starts. The runner now builds
the integration test before starting services or minting any token. The warm
rerun passed all seven scenarios and replaced the failed receipt. Targeted Keto
tests, strict Clippy, format, shell checks, file size, diff and strict OpenSpec
validation pass. A fresh independent resolution review remains before
convergence.

## Iteration 3

Result: **SECOND REVIEW FINDINGS RESOLVED; FINAL REVIEW PENDING**.

The resolution judge found that Keto cache identity lacked the tenant and that
the compatibility receipt incorrectly called the public JWKS type replacement
additive. The provider now binds effective namespace, tenant and subject into an
unambiguous authority scope while preserving the existing three-string
`CacheKey` API. A regression proves identical tuples in two tenants perform two
remote checks. Successful writes also invalidate prior cached decisions.

The receipt now records the JWKS alias-to-struct replacement as a pre-1.0
source-breaking change requiring at least a minor release and migration notes in
c023. The stale proposal status is corrected. The final Keto suite passed 10
tests, strict Clippy and OpenSpec validation passed, and the refreshed owned
runtime passed seven scenarios. A duplicate aggregate rerun stalled in sccache
on unchanged SurrealDB code and was stopped without a result; it is explicitly
excluded from evidence.

## Iteration 4

Result: **THIRD REVIEW WARNING RESOLVED; FINAL REVIEW PENDING**.

The third judge passed the candidate with one warning: any HTTP 403 was treated
as a normal Keto denial. The adapter now parses the expected denial response and
accepts only `allowed=false`. A malformed body returns a serialization failure;
a contradictory `allowed=true` returns a transport failure. Both cases have
regression tests. The full Keto crate suite passed 12 tests, strict Clippy
passed, and the source-bound seven-scenario runtime receipt was refreshed.

Resolution review: **PASS**, 0 critical, 0 warnings, 0 suggestions. The
sycophancy screen passed with score 0.0 and the judge/producer models were
verified distinct. All blocking constraints are satisfied and further changes
fall below the refinement threshold.
