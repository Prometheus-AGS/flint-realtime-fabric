# Verification receipt — pri-c004-deployment-profiles

Captured: 2026-09-15T12:26:13Z

## Result

Implementation: COMPLETE for the deployment-profile boundary.
Production readiness: BLOCKED by the later CDC, replay, projection, persistence,
consumer, recovery and release-image qualification changes. This receipt does
not certify an image or production deployment.

## Source binding

| Repository | Revision | status SHA-256 |
|---|---|---|
| Fabric | `4c2a8326c932ebe0f80799454b0cbbf378e53811` | `07e257303ff41eb1ce0cd377c9635ab6acfaa903b7de24259e33bc8bafe5bc86` |
| Gate | `de88f40dbfe77bc03203bafc2b64de9815fe27b2` | clean, `e3b0c442…b855` |

Fabric's source commit is the c003 boundary; the candidate is the scoped
uncommitted c004 diff plus the hashes below. The three pre-existing shape files
still match `entry-state.json` exactly and are excluded from c004. Gate remained
clean and required no source change.

The candidate worktree status after the final local gates has SHA-256
`56f27a07ef58cbccf2ef4475d435dd8293ce851a8a18f9149b12c43d2cbc676a`.

## Candidate fingerprints

| Artifact | SHA-256 |
|---|---|
| `.env.example` | `bd32155fc9380664890bdc5578909440f5b33cc98f7f9fc20ff3c3da792b7daa` |
| `compose.yml` | `3eb89593d04d4b6b9a0e64fe67579bf8e41bd8aa3a228165a3f3fa9577002812` |
| `compose.shape-only.yml` | `7031710a07145d05bf277e1ca6ae293a11dd3cebf651b453914dfa16e4b12eaf` |
| `config/mod.rs` | `9dd0a6bbe0b43841b0d82eda65dcd8dbc791602ff28a46a208002ab0c8dd7f9c` |
| `config/validation.rs` | `b45dc7086354ab63929f3d86df21162e1647ee7ba8a6fa91eba9f612620a4539` |
| `configured_broker.rs` | `b859074a5852f2481c99d149716228bcc683f5f24866f4d5968eedf10d7a1b4c` |
| `routes/health.rs` | `e99966bb7efc4df5b4ba82bbad86466ddb3a6db6dc9f73404dc37a8f4e0d3dd3` |
| `gateway lib.rs` | `65c4aaec970fb28ad3958a1c4ca06a219fb7587ba4a8d7a132c825787859f302` |
| `gateway main.rs` | `9455354440d2f8bf59497a86860c4d4cdc3faa54e42a4299409d08677947b4ce` |
| `tests/deployment_profiles.rs` | `697ac705083d82519254c6aa6ff33e73a0fcaf69bbd23d91214ff1608570671e` |
| `postgres-cdc consumer.rs` | `25e3306ebfb1c551605d7f08a942d88964cc7f3f839fbde25740f0c2a7e471fa` |
| `nginx-full.conf` | `c28a1335fb303ebdb0919b0ee059356a29fa7eb33a43d884dfc31d72b4f2454b` |
| `nginx-shape-only.conf` | `43cd73956b4126b1f483d139f36407c6cf88b28ecfe44bb5c622c7a732338c6a` |
| `render-deployment-profile.sh` | `67ef9f3240ec9c7c9b38a8b12a3b55f64fbe4ecdc38e5f2f3e732af3026811b8` |
| `test-deployment-profiles.sh` | `de7d756a90bb0c653cfb48f107abc5f1f185b79f08dccbc3312e7503c1262fe4` |
| SSR `gateway.yaml` | `00dd8d3814d221b139fa1642be860adbd1317eaec09afa0b5c56b99276e62699` |
| SSR `ingress.yaml` | `88c3f5e5b592f12d350ecfa78c33b0eca2d7389a75a3c3e1a8087d69107b541d` |
| SSR `network-policy.yaml` | `f893bc0aa46d75100ddc9fc7a854cb6fa3710c25123493fa875816bcc0393686` |
| SSR `render-profile.sh` | `0cf4694b424c32024babe7f837cd95db1006ef139234038772d634a17b8faba0` |

The removed `k8s/overlays/ssr/iggy.yaml` is represented by its deletion from the
base revision. Secondary touched-file hashes are available from the local
command log and final Git tree.

## Local verification

| Gate | Result |
|---|---|
| `cargo check --workspace --locked` | PASS |
| `cargo check -p frf-gateway --features shape-facade --locked` | PASS |
| `cargo clippy -p frf-postgres-cdc -p frf-gateway --all-targets --features shape-facade --locked -- -D warnings -W clippy::pedantic` | PASS |
| `cargo test -p frf-postgres-cdc --locked` | PASS; 7 unit + 3 smoke |
| `cargo test -p frf-gateway --features shape-facade --locked` | PASS; 76 executed, 1 owned-integration test explicitly ignored |
| owned `scripts/run-local-integration.sh` | PASS; 2/2, real pinned Iggy, cleanup complete |
| runtime receipt | `c004-profile-boundary.json`, exit 0 |
| `scripts/test-deployment-profiles.sh` | PASS; render/negative/live TLS matrix |
| Shellcheck for three profile scripts | PASS |
| `kubectl apply --dry-run=client --validate=false` over rendered SSR | PASS; TLS Secret and profile resources accepted |
| mutable-image, reserved-host, missing-Secret and mismatched-key SSR negatives | PASS; rejected |
| `cargo fmt --all --check` | PASS |
| `bash scripts/check-file-size.sh` | PASS; 273 files, none over 500 lines |
| `git diff --check` | PASS |
| frozen `proto/flint/v1` and dependency manifests | PASS; no diff |
| `openspec validate pri-c004-deployment-profiles --strict --no-interactive` | PASS |
| fresh post-QA resolution review | PASS; 0 critical, 0 warning, 0 suggestion |

The live TLS test used immutable Nginx index
`sha256:42a516af16b852e33b7682d5ef8acbd5d13fe08fecadc7ed98605ba5e3b26ab8`.
Both profile edges accepted a CA-verified HTTPS request. Plaintext on the TLS
port, direct backend access, mutable images, missing/mismatched TLS inputs,
symmetric Gate signing and a development Gate provider were rejected.

## Compatibility and remaining boundary

`proto/flint/v1` is byte-unchanged. `GatewayConfig` gains profile inputs and the
public `GatewayLanes` value; exhaustive downstream Rust struct literals are
source-incompatible. The workspace remains pre-1.0 at `0.1.0`; a published
release must treat this as a breaking pre-1.0 API change and select an
appropriate version before c023 signoff. Environment defaults keep optional
agent, media and admin lanes off; deployments that need them must opt in.

The full profile now has truthful Iggy/Keto/Gate readiness and scoped routes,
but persistent projection/CRDT adapters and release images remain later gates.
The shape profile has truthful Gate/Electric readiness and no Iggy composition;
ASO catalog/topology and revocation proofs remain c014/c015 responsibilities.

## Review resolution

The preserved initial review (`packet.json`, `findings.json`) reported one
critical and two warnings. The distinct fresh review (`resolution-packet.json`,
`resolution-findings.json`) verified the implemented corrections and returned
PASS with zero findings. The initial and resolution review artifact hashes are
recorded in the review directory and both pairs remain valid JSON under the
80-line review limit.
