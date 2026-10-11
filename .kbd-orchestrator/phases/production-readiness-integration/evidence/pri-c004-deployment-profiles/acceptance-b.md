# Acceptance B — artifact, TLS and authority boundaries

Captured: 2026-09-15

## Executable profile gate

`scripts/test-deployment-profiles.sh` ran locally and passed. It generated an
ephemeral CA/certificate/key set, rendered both profiles, exercised negative
inputs, and removed its owned containers and network.

| Check | Result |
|---|---|
| Every rendered image ends in a nonzero `@sha256:<64 hex>` digest | PASS |
| Only service `edge` has host-published ports | PASS for both profiles |
| `full` has no shape endpoint and disables media, agent, admin, federation | PASS |
| `shape-only` has no Iggy DSN, disables gRPC/CDC/media/agent/admin/federation | PASS |
| Missing issuer | rejected before render |
| Mutable image tag | rejected before render |
| Missing TLS key | rejected before render |
| Certificate/private-key mismatch | rejected before render |
| Gate config containing an anonymous provider | rejected before render |
| Missing or mismatched asymmetric Gate signing key | rejected before render |
| Gate config using a symmetric signing algorithm | rejected before render |
| Full data router mounts agent/media/admin endpoints | PASS negative; all return 404 when disabled |

The source contains no hard-coded developer checkout path. Gate configuration,
shape catalog, certificate, key, CA and Gate key pair are explicit file inputs;
passwords are required environment/secret-manager inputs. The committed
`.env.example` contains no usable secret.

## Live TLS boundary

The test pulled and ran the immutable multi-platform Nginx OCI index
`nginx@sha256:42a516af16b852e33b7682d5ef8acbd5d13fe08fecadc7ed98605ba5e3b26ab8`.
It started an internal backend with no published port, then started each edge
configuration with the generated certificate/key/CA mounts.

| Scenario | Result |
|---|---|
| `curl --cacert <ca> https://localhost:<dynamic>/healthz` through full edge | PASS; backend response received |
| plaintext HTTP sent to the same TLS port | rejected |
| lookup of a host-published backend port | none present |
| trusted HTTPS through shape edge | PASS |
| `/ws/v1/subscribe` through shape edge | `404`; route outside profile authority |

The full edge also exposes TLS gRPC separately and admits only `SpineService`,
`SyncService`, and `EntityService`. The shape edge admits only `/healthz`,
`/readyz`, and `/v1/shape`. Internal Gate/Electric/Keto/Iggy/Postgres traffic is
isolated on non-published Compose networks.

## Kubernetes SSR render

The SSR template uses a zero image digest and reserved host so direct application
cannot masquerade as a release. `render-profile.sh` requires a nonzero immutable
shape-image digest, real DNS hostname, Secret name and valid trusted TLS keypair.
It emits the bound `kubernetes.io/tls` Secret, including `ca.crt`. A rendered candidate passed
`kubectl apply --dry-run=client --validate=false` for its ServiceAccount,
Service, Deployment, PodDisruptionBudget, Ingress and NetworkPolicies. Negative
tests rejected a mutable image, `.invalid` host, missing Secret name and
mismatched TLS key.

The Ingress forces TLS using the rendered Secret and forwards to the actual Gate
proxy Service. The
Fabric Service is reachable only from Gate by NetworkPolicy; its outbound access
is limited to Gate JWKS, Electric and DNS. Iggy and its workload were removed
from this restricted profile.
