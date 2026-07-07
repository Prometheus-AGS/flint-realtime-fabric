#![allow(clippy::unwrap_used, clippy::expect_used)] // test/bench crate — see clippy.toml + rules/rust/testing.md

//! Integration tests for the gateway security middleware (p16-c005):
//! request body-size limit and CORS policy applied by `apply_security_layers`.

use axum_test::TestServer;
use frf_gateway::GatewayConfig;

fn config_with(max_body_bytes: usize, origins: &[&str]) -> GatewayConfig {
    let mut cfg = GatewayConfig::test_default();
    cfg.max_body_bytes = max_body_bytes;
    cfg.cors_allowed_origins = origins.iter().map(|s| (*s).to_owned()).collect();
    cfg
}

#[tokio::test]
async fn body_within_limit_is_accepted() {
    let cfg = config_with(64, &[]);
    let app = frf_gateway::build_security_test_router(&cfg);
    let server = TestServer::new(app).expect("test server");

    let response = server.post("/echo").text("small").await;
    response.assert_status_ok();
    response.assert_text("small");
}

#[tokio::test]
async fn body_over_limit_is_rejected() {
    // 8-byte cap; send 64 bytes.
    let cfg = config_with(8, &[]);
    let app = frf_gateway::build_security_test_router(&cfg);
    let server = TestServer::new(app).expect("test server");

    let oversized = "x".repeat(64);
    let response = server.post("/echo").text(oversized).await;

    // RequestBodyLimitLayer rejects with 413 Payload Too Large.
    assert_eq!(
        response.status_code(),
        413,
        "expected 413 for over-limit body, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn cors_preflight_from_allowed_origin_is_permitted() {
    let cfg = config_with(1024, &["https://admin.example.com"]);
    let app = frf_gateway::build_security_test_router(&cfg);
    let server = TestServer::new(app).expect("test server");

    let response = server
        .method(axum::http::Method::OPTIONS, "/echo")
        .add_header("origin", "https://admin.example.com")
        .add_header("access-control-request-method", "POST")
        .await;

    // The CORS layer answers the preflight and echoes the allowed origin.
    let allow_origin = response
        .maybe_header("access-control-allow-origin")
        .expect("CORS layer should set access-control-allow-origin for an allowed origin");
    assert_eq!(allow_origin, "https://admin.example.com");
}

#[tokio::test]
async fn cors_origin_not_in_allowlist_is_not_reflected() {
    let cfg = config_with(1024, &["https://admin.example.com"]);
    let app = frf_gateway::build_security_test_router(&cfg);
    let server = TestServer::new(app).expect("test server");

    let response = server
        .method(axum::http::Method::OPTIONS, "/echo")
        .add_header("origin", "https://evil.example.com")
        .add_header("access-control-request-method", "POST")
        .await;

    // A disallowed origin must NOT be reflected in access-control-allow-origin.
    assert!(
        response
            .maybe_header("access-control-allow-origin")
            .is_none(),
        "disallowed origin must not be reflected"
    );
}
