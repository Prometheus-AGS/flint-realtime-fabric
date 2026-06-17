# Tasks — p1-c005 frf-identity-ory

- [ ] **T1** Create `crates/frf-identity-ory/src/claims.rs`
  - `FrfClaims`: serde struct for JWT payload — `sub: String`, `email: Option<String>`, `tenant_id: Option<String>`, `roles: Option<Vec<String>>`, `jti: Option<String>`, `aud: Vec<String>` (standard), `exp: u64`
  - `fn to_verified_claims(claims: FrfClaims) -> Result<VerifiedClaims, IdentityError>`
  - Verification: `cargo check -p frf-identity-ory` exits 0

- [ ] **T2** Create `crates/frf-identity-ory/src/jwks.rs`
  - `JwksCache` wrapping `Arc<tokio::sync::RwLock<Option<jsonwebtoken::jwk::JwkSet>>>`
  - `async fn fetch_and_cache(http: &reqwest::Client, url: &str, cache: &JwksCache) -> Result<jsonwebtoken::jwk::JwkSet, IdentityError>`
  - `async fn get_or_fetch(http: &reqwest::Client, url: &str, cache: &JwksCache) -> Result<jsonwebtoken::jwk::JwkSet, IdentityError>` — read cache; fetch if None
  - `async fn refresh(http: &reqwest::Client, url: &str, cache: &JwksCache) -> Result<jsonwebtoken::jwk::JwkSet, IdentityError>` — force re-fetch (key rotation)
  - Verification: `cargo check -p frf-identity-ory` exits 0

- [ ] **T3** Create `crates/frf-identity-ory/src/verifier.rs`
  - `pub struct OryIdentityVerifier { http: reqwest::Client, jwks_url: String, jwks_cache: JwksCache, audience: String }`
  - `impl OryIdentityVerifier { pub fn new(jwks_url: impl Into<String>, audience: impl Into<String>) -> Self }`
  - `#[async_trait] impl IdentityVerifier for OryIdentityVerifier`
    - `verify(token)`: decode header to get `kid` → get JWKS from cache → find matching JWK → verify RS256 signature → decode claims → if InvalidSignature/UnknownKid: refresh JWKS and retry once → extract `VerifiedClaims`
  - `#[tracing::instrument(name = "port::IdentityVerifier::verify", skip(token))]`
  - Verification: `cargo check -p frf-identity-ory` exits 0

- [ ] **T4** Update `crates/frf-identity-ory/src/lib.rs`
  - `pub mod claims; pub mod jwks; pub mod verifier;`
  - `pub use verifier::OryIdentityVerifier;`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-identity-ory` exits 0

- [ ] **T5** Write tests with mock JWKS server
  - File: `crates/frf-identity-ory/tests/verifier.rs`
  - Use `httpmock::MockServer` to serve a JWKS JSON document
  - Generate a test RSA keypair using `jsonwebtoken`'s test helpers (or hard-code a test key pair)
  - Tests:
    - `verify_valid_jwt_returns_claims`
    - `verify_expired_jwt_returns_error`
    - `verify_unknown_kid_refreshes_jwks_and_retries`
    - `verify_bad_audience_returns_error`
  - Verification: `cargo test -p frf-identity-ory` — all 4 tests pass
