# Refinement log — pri-c004-deployment-profiles

## Iteration 1

Result: **REVIEW FAIL** after the initial local matrix passed.

| Constraint | Result | Evidence |
|---|---|---|
| A | FAIL | Fresh readiness passed, but the reviewer found that `/readyz` could remain green after the required CDC stream exited. |
| B | PASS | Digest/TLS/Gate-input negative matrix, live CA-verified HTTPS, edge-only ports, router 404 checks and SSR dry-run all pass. |
| P1 | PASS | `openspec validate pri-c004-deployment-profiles --strict --no-interactive`. |
| P2 | PASS | Workspace check, all-target gateway Clippy, 75 executed gateway tests, Shellcheck, format, size and diff gates passed. |
| S1 | PASS | Receipt keeps persistence, replay, recovery, ASO and release-image certification assigned to later changes. |

Clippy first rejected the expanded boolean configuration and the shape-feature
build exposed a missing test-state field. The candidate now groups optional
lanes in `GatewayLanes`, and every AppState fixture compiles with the selected
feature. These corrections were re-run through the complete quality matrix.
The integration audit also corrected Compose and SSR network policy to
use Gate's public JWKS endpoint on port 4456; the deployment-profile matrix and
schema checks passed after that correction. The SSR ingress now targets Gate's
actual `flint-gate-proxy` Service, and the rendered manifest passes local dry-run.

The independent post-QA reviewer then found one critical issue and two warnings:
CDC health was not supervised by readiness, Kubernetes TLS was declared without
a bound verified Secret, and task 5 could not close while those findings remained.

## Iteration 2

Result: **PASS** with zero blocking violations.

| Constraint | Result | Evidence |
|---|---|---|
| A | PASS | CDC readiness starts false, becomes true only after replication starts, and returns false on every consumer exit; full-profile tests prove inactive CDC yields 503. |
| B | PASS | Compose seeds matched asymmetric Gate keys; the SSR renderer verifies and emits its named TLS Secret; every new negative passes. |
| P1 | PASS | Strict OpenSpec validation passes after the fixes. |
| P2 | PASS | Workspace check, strict CDC/gateway Clippy, 10 CDC tests, 76 gateway tests, Shellcheck, format, size and diff gates pass. |
| S1 | PASS | Receipt still assigns data correctness, recovery and release qualification to later changes. |

The owned Iggy integration was rerun after the fixes and passed 2/2 with cleanup.
The distinct fresh resolution review returned PASS with zero critical, warning,
or suggestion findings. It preserved the initial failing review and independently
verified each correction against product code and local evidence.
