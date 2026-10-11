# Baseline — pri-c005-authority-lifetime

Captured: 2026-09-15

## Accepted dependencies

| Change | Receipt SHA-256 | Accepted boundary |
|---|---|---|
| `pri-c002-local-fixtures` | `4409fe7fa6ca1db3b01091ea8160fb4317a8102b8fd1e5074912177185716622` | owned local Iggy delivery fixture; no CDC claim |
| `pri-c003-scope-research-contract` | `84583fc30b6b5ef13bd41ace5d02b2447f9133da40bb4ee3cd2ed9c09778c366` | Q2 full/shape-only topology, Q5 tenant/object rules, Q7 five-second authority lifetime |
| `pri-c004-deployment-profiles` | `e4e022ab6db66ad6052aa70737c26ce9e798aea0b5b1f67d45e63f638ba2152f` | truthful full and shape-only profiles, TLS edge, asymmetric Gate bootstrap |

Each dependency is verified and archived. C005 does not expand their release
claims: the generic event path still needs stream-lifetime enforcement, while
the restricted shape path must preserve the previously measured lease boundary.

## Repository identity and isolation

| Repository | Entry revision | Branch / status |
|---|---|---|
| Fabric | `315f8494fafcad9c7f8f42b58b791607f79cd9d9` | `codex/production-readiness-integration`; status SHA-256 `e05435a3492c6006b30216d5098b40ac1f5a97847b10cf5d10323c8dc35dbba0` |
| Gate | `dfe685bf25cc07db2d0c289a92fbaa5c20eb931a` | external `feat/db-lazy-reconnect` checkout, clean at final baseline observation |
| Forge | `dc313be3a044c65b05d845a9c34350bf5ca3ca3e` | `main`, clean |
| ASO | `d95242542060d1e19a6efe9143f57c015d6162f4` | `main`, 382 pre-existing paths; status SHA-256 `60becd615840e5dc7d817ef31b0f6ea2f447f176e3b6746cec89869ceb31ff8d` |

The Gate checkout changed from clean `main` at `de88f40…` during c004 to an
externally owned recovery branch while c005 baseline inspection was running.
C005 will not edit that checkout. Any Gate source change must use an isolated
worktree. The ASO checkout is also read-only for this change because its large
pre-existing diff is outside c005 ownership.

Fabric contains 62 dirty/untracked paths from KBD planning plus three
pre-existing shape edits. Their entry hashes remain:

- `crates/frf-app/src/shape/mod.rs`: `ee3d47c8…acf1ad9`
- `crates/frf-app/src/shape/tests.rs`: `4bcd1c59…35a4d04`
- `crates/frf-gateway/src/routes/shape.rs`: `60610541…ebf4d6`

Those three files remain excluded from c005 commits.

## Affected-source fingerprints

| Path | SHA-256 |
|---|---|
| `crates/frf-app/src/subscribe.rs` | `93ec80871908408c8fa6bcf137eac3e6f122928b69b3a69828131e2c14ce0c84` |
| `crates/frf-app/src/shape/lease.rs` | `62dc5644ef424a15bf9af57b64d33be287fd65a570f0e9e7031681eddde4879f` |
| `crates/frf-authz-keto/src/provider.rs` | `1146d5201a9687b0e1fa84f5c6f46bec4a892db17be958581681b41a61837e36` |
| `crates/frf-authz-keto/src/cache.rs` | `d6ba3a6ab6bc407b68edb8235cd2a6f7373219d2358b1deede3e3fe4c9a6829b` |
| `crates/frf-identity-ory/src/verifier.rs` | `735a74c78929257170d7d1b6571e106bdff4633f51873469a61440dcedde3018` |
| `crates/frf-identity-ory/src/jwks.rs` | `555b95e82d5e3f61537d21076ba1bb362e2033df38d3ae4a63cf99dccfd639c9` |
| `crates/frf-app/tests/subscribe_pipeline.rs` | `3561d4659291bc89013b8639ce86c546cefe54234a85244457a86ff1865b3144` |
| `crates/frf-app/tests/shape_revocation.rs` | `e9d1a33d41b257965bcf336c68bae3cc9e40983d1981342ea365ee0c107c1342` |
| `crates/frf-app/tests/shape_lease_timing.rs` | `7bc61bbceedd8d5e58150cd24694c83091f70ba066fd35179493d1741a1aef57` |

## Gap observed at entry

- Generic subscription verifies the token once, so an idle stream can survive
  token expiry until another layer closes it.
- Each event receives a tenant check and an object `view` decision, but Keto
  caches that decision for 60 seconds, beyond the accepted five-second bound.
- A relation deletion invalidates existing cache entries, but an older in-flight
  allow response can refill the cache after invalidation.
- JWKS refresh occurs for an unknown key or bad signature. A still-valid token
  signed by a removed cached key remains accepted because the cache has no age.
- Restricted shape responses already use a 1,750 ms maximum lease, 750 ms
  revalidation, authority timeout, buffered-frame rejection, and new-request
  denial tests. C005 must keep those guarantees intact and prove them without
  adopting the pre-existing shape edits.

Locked identities: Fabric `Cargo.lock` `605fe98e…30736`; Gate `Cargo.lock`
`f7a12663…cf11`; ASO `web/pnpm-lock.yaml` `3dcdb70c…c789c`.
