# p1-c005 — frf-identity-ory: IdentityVerifier → Ory Oathkeeper

## Affected crates
- `crates/frf-identity-ory` (new — stub created by p1-c001)

## Dependency-rule impact
Layer 2 (infrastructure adapter). Imports `frf-domain` and `frf-ports`. Implements `IdentityVerifier`. Must NOT import `frf-app` or any other adapter crate.

## What this change does

Implements the `IdentityVerifier` port using JWKS-based JWT verification.

### Verification strategy (Oathkeeper-proxied)
Oathkeeper is the authentication proxy. By the time a request reaches `frf-gateway`, Oathkeeper has already validated the upstream bearer token and re-minted a new signed JWT (via the `id_token` mutator). The gateway receives this re-minted token.

`frf-identity-ory` verifies this re-minted JWT:
1. Fetch JWKS from Oathkeeper's `/.well-known/jwks.json` (cached in memory)
2. Decode + verify the JWT signature against the cached JWKS
3. Extract claims: `sub`, `email`, `tenant_id` (custom claim), `roles` (custom claim)
4. Return `VerifiedClaims`

On signature verification failure with `InvalidKeyFormat` or unknown `kid`: refresh JWKS cache and retry once (handles key rotation).

### `OryIdentityVerifier` struct
```rust
pub struct OryIdentityVerifier {
    http: reqwest::Client,
    jwks_url: String,
    jwks_cache: Arc<RwLock<Option<JwkSet>>>,
    audience: String,
}
```

### `VerifiedClaims` extraction
Custom JWT claims expected (set by Oathkeeper id_token mutator):
```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "tenant_id": "tenant-uuid",
  "roles": ["admin", "viewer"],
  "aud": ["frf-gateway"]
}
```
`session_id` is derived as a `SessionId` from the JWT `jti` claim (or generated if absent).

### Module layout
```
crates/frf-identity-ory/src/
├── lib.rs
├── verifier.rs    OryIdentityVerifier + IdentityVerifier impl
├── jwks.rs        JWKS fetch + cache (RwLock<Option<JwkSet>>)
└── claims.rs      JWT claims structs + VerifiedClaims extraction
```

## Phase 1 exit criterion satisfied
`verify()` tested with a known RSA-signed test JWT and a mock JWKS server (httpmock). Key rotation retry path tested.

## Non-goals
- Does not implement the full Kratos SDK (`GET /sessions/whoami`) — that is the fallback for Phase 1 dev; implement only the Oathkeeper-proxied path.
- Does not implement token refresh or session management.
- Does not handle PASETO or non-JWT token formats.
