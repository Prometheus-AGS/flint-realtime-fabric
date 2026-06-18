use frf_identity_ory::OryIdentityVerifier;
use frf_ports::IdentityVerifier;
use httpmock::prelude::*;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;
use uuid::Uuid;

// RSA modulus (base64url) for the test key in tests/fixtures/test_private_rsa.pem
const TEST_KID: &str = "test-key-1";
const TEST_RSA_N: &str = "yRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5_CYYi_cvI-SXVT9kPWSKXxJXBXd_4LkvcPuUakBoAkfh-eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG_AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi-yUod-j8MtvIj812dkS4QMiRVN_by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQ";
const TEST_RSA_E: &str = "AQAB";

fn test_jwks_json() -> serde_json::Value {
    json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": TEST_KID,
            "n": TEST_RSA_N,
            "e": TEST_RSA_E
        }]
    })
}

fn private_key_pem() -> Vec<u8> {
    include_bytes!("fixtures/test_private_rsa.pem").to_vec()
}

fn make_jwt(_tenant_id: &str, exp_offset_secs: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let exp = (now as i64 + exp_offset_secs) as u64;

    let claims = serde_json::json!({
        "sub": Uuid::new_v4().to_string(),
        "email": "user@example.com",
        "tenant_id": Uuid::nil().to_string(),
        "roles": ["viewer"],
        "jti": Uuid::new_v4().to_string(),
        "aud": ["frf-gateway"],
        "exp": exp,
    });

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_owned());

    let key = EncodingKey::from_rsa_pem(&private_key_pem()).expect("test RSA private key");
    encode(&header, &claims, &key).expect("encode JWT")
}

#[tokio::test]
async fn verify_valid_jwt_returns_claims() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/.well-known/jwks.json");
        then.status(200).json_body(test_jwks_json());
    });

    let jwks_url = format!("{}/.well-known/jwks.json", server.base_url());
    let verifier = OryIdentityVerifier::new(jwks_url, "frf-gateway");
    let token = make_jwt(Uuid::nil().to_string().as_str(), 300);

    let claims = verifier.verify(&token).await.expect("verify failed");
    assert_eq!(claims.email.as_deref(), Some("user@example.com"));
    assert!(!claims.roles.is_empty());
}

#[tokio::test]
async fn verify_expired_jwt_returns_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/.well-known/jwks.json");
        then.status(200).json_body(test_jwks_json());
    });

    let jwks_url = format!("{}/.well-known/jwks.json", server.base_url());
    let verifier = OryIdentityVerifier::new(jwks_url, "frf-gateway");
    // exp in the past
    let token = make_jwt(Uuid::nil().to_string().as_str(), -300);

    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "expected error for expired token");
}

#[tokio::test]
async fn verify_unknown_kid_refreshes_jwks_and_retries() {
    let server = MockServer::start();
    // JWKS is served twice — once for the initial warm-up (miss) and once after refresh
    let mock = server.mock(|when, then| {
        when.method(GET).path("/.well-known/jwks.json");
        then.status(200).json_body(test_jwks_json());
    });

    let jwks_url = format!("{}/.well-known/jwks.json", server.base_url());
    let verifier = OryIdentityVerifier::new(jwks_url, "frf-gateway");

    // Mint a JWT with a kid that is NOT in the initial JWKS; after refresh the JWKS will match.
    // We model this by first populating the cache with a JWKS that has a different kid,
    // then presenting a token with TEST_KID. Since the cache starts empty, get_or_fetch
    // runs on the first call — the kid IS present, so no retry is triggered.
    // To test the rotation path we mint a JWT with an unknown kid.
    let claims_json = serde_json::json!({
        "sub": Uuid::new_v4().to_string(),
        "email": "rotate@example.com",
        "tenant_id": Uuid::nil().to_string(),
        "roles": [],
        "jti": Uuid::new_v4().to_string(),
        "aud": ["frf-gateway"],
        "exp": 9_999_999_999_u64,
    });
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("unknown-kid".to_owned());
    let key = EncodingKey::from_rsa_pem(&private_key_pem()).unwrap();
    let token = encode(&header, &claims_json, &key).unwrap();

    // First verify call: unknown kid → refresh JWKS → still not found → error
    // (The test validates that the refresh path is exercised; the refreshed JWKS also
    //  doesn't contain "unknown-kid", so the call returns an error.)
    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "unknown kid should fail after refresh");

    // JWKS was fetched twice (initial + refresh)
    mock.assert_hits(2);
}

#[tokio::test]
async fn verify_bad_audience_returns_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/.well-known/jwks.json");
        then.status(200).json_body(test_jwks_json());
    });

    let jwks_url = format!("{}/.well-known/jwks.json", server.base_url());
    // Verifier expects "frf-gateway" but token has "wrong-audience"
    let verifier = OryIdentityVerifier::new(jwks_url, "frf-gateway");

    let claims_json = serde_json::json!({
        "sub": Uuid::new_v4().to_string(),
        "tenant_id": Uuid::nil().to_string(),
        "aud": ["wrong-audience"],
        "exp": 9_999_999_999_u64,
    });
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_owned());
    let key = EncodingKey::from_rsa_pem(&private_key_pem()).unwrap();
    let token = encode(&header, &claims_json, &key).unwrap();

    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "wrong audience should fail");
}
