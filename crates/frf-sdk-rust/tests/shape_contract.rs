#![cfg(feature = "shape-facade")] // the module under test is gated; so is this file
#![allow(clippy::unwrap_used, clippy::expect_used)] // test crate — see clippy.toml + rules/rust/testing.md

//! Contract tests for [`ShapeClient`] against the facade's documented wire
//! contract (`crates/frf-gateway/src/routes/shape.rs`).
//!
//! These pin *server behaviour*, not implementation detail. Each case names the
//! rule it enforces, and each was verified to fail when the mechanism it covers
//! is removed — a test that stays green when its branch is deleted proves
//! nothing. The TypeScript equivalent shipped exactly that defect: its 204 case
//! asserted an empty message list, which a broken implementation also produces
//! because a failed body parse yields the same empty list.

use frf_sdk_rust::{SdkError, ShapeClient, ShapeCursor, ShapeRequestOptions};
use httpmock::prelude::*;

/// A client pointed at a mock server, with a bearer token.
fn client(server: &MockServer) -> ShapeClient {
    ShapeClient::new(server.base_url(), Some("t-1".to_owned())).expect("client builds")
}

/// A client with no credential — the browser/cookie-propagation case.
fn anonymous_client(server: &MockServer) -> ShapeClient {
    ShapeClient::new(server.base_url(), None).expect("client builds")
}

fn options() -> ShapeRequestOptions {
    ShapeRequestOptions::default()
}

// ── Request contract ────────────────────────────────────────────────────────

#[tokio::test]
async fn a_cold_request_sends_only_the_shape_id() {
    let server = MockServer::start();
    // A shape declaring `allowed_params: []` rejects any extra query parameter
    // with 400. Cold means: the shape id, and nothing else.
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/shape")
            .query_param("shape", "cases");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });

    client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("cold fetch succeeds");

    mock.assert();
}

#[tokio::test]
async fn a_cursor_sends_both_halves_together() {
    let server = MockServer::start();
    // `parse_cursor` returns Err for a half cursor, which becomes a 400. The
    // `ShapeCursor` struct makes the half form unrepresentable; this proves the
    // client actually sends both.
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/shape")
            .query_param("handle", "h-1")
            .query_param("offset", "17");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });

    let cursor = ShapeCursor {
        handle: "h-1".to_owned(),
        offset: "17".to_owned(),
    };
    client(&server)
        .fetch_shape("cases", Some(&cursor), &options())
        .await
        .expect("resume succeeds");

    mock.assert();
}

#[tokio::test]
async fn a_bearer_token_is_presented_when_configured() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/shape")
            .header("authorization", "Bearer t-1");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });

    client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");

    mock.assert();
}

#[tokio::test]
async fn no_authorization_header_is_sent_without_a_token() {
    let server = MockServer::start();
    // A browser lets Gate mint the downstream JWT from a session cookie. A
    // client that invented an Authorization header here would defeat that.
    //
    // httpmock has no negative header matcher, and asserting only that *a*
    // request arrived would pass whether or not the header was present — the
    // vacuity trap. So: two mocks, and the discriminator is which one matches.
    // The authorization-bearing mock is registered FIRST, so httpmock prefers
    // it if the header is present; it recording zero hits is the proof.
    let with_auth = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/shape")
            .header_exists("authorization");
        then.status(500).body("should never match");
    });
    let without_auth = server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });

    anonymous_client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");

    with_auth.assert_hits(0);
    without_auth.assert_hits(1);
}

#[tokio::test]
async fn protocol_echoes_are_forwarded_but_nothing_else() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/shape")
            .query_param("live", "true")
            .query_param("cursor", "c-9")
            .header("if-none-match", "etag-1");
        then.status(200)
            .header("content-type", "application/json")
            .body("[]");
    });

    let opts = ShapeRequestOptions {
        live: true,
        cursor: Some("c-9".to_owned()),
        if_none_match: Some("etag-1".to_owned()),
    };
    client(&server)
        .fetch_shape("cases", None, &opts)
        .await
        .expect("fetch succeeds");

    mock.assert();
}

// ── Refusals are distinguishable ────────────────────────────────────────────

async fn refusal_for(status: u16, body: &str) -> SdkError {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(status).body(body);
    });
    client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect_err("must refuse")
}

#[tokio::test]
async fn grant_expiry_is_recoverable_and_distinct() {
    // 401 means the credential must change. A backoff loop that treats this as
    // transient spins forever.
    assert!(matches!(
        refusal_for(401, "").await,
        SdkError::ShapeGrantExpired
    ));
}

#[tokio::test]
async fn forbidden_is_distinct_from_expiry() {
    assert!(matches!(
        refusal_for(403, "").await,
        SdkError::ShapeForbidden
    ));
}

#[tokio::test]
async fn unknown_shape_names_the_shape_because_it_is_a_catalog_mismatch() {
    let error = refusal_for(404, "").await;
    match error {
        SdkError::UnknownShape(shape) => assert_eq!(shape, "cases"),
        other => panic!("expected UnknownShape, got {other:?}"),
    }
}

#[tokio::test]
async fn an_invalid_request_carries_the_facades_detail() {
    let error = refusal_for(400, "invalid shape cursor").await;
    match error {
        // The detail must survive: it names the offending key class, which is
        // the difference between a fixable bug and a guess.
        SdkError::InvalidShapeRequest(detail) => assert!(detail.contains("invalid shape cursor")),
        other => panic!("expected InvalidShapeRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn an_upstream_failure_is_retryable_unlike_every_refusal() {
    assert!(matches!(
        refusal_for(502, "").await,
        SdkError::ShapeUpstream(_)
    ));
    // 503 (clock before epoch) and 500 (unrecognized) fold into the same
    // retryable class — they say nothing is wrong with the grant.
    assert!(matches!(
        refusal_for(503, "").await,
        SdkError::ShapeUpstream(_)
    ));
    assert!(matches!(
        refusal_for(500, "").await,
        SdkError::ShapeUpstream(_)
    ));
}

// ── Protocol handling ───────────────────────────────────────────────────────

#[tokio::test]
async fn a_cursor_is_returned_only_when_both_header_halves_are_present() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            .header("electric-handle", "h-1")
            .header("electric-offset", "0_0")
            .body("[]");
    });
    let frame = client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");
    assert_eq!(
        frame.cursor,
        Some(ShapeCursor {
            handle: "h-1".to_owned(),
            offset: "0_0".to_owned(),
        })
    );

    // Half a cursor handed back would be echoed and rejected with 400.
    let half = MockServer::start();
    half.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            .header("electric-handle", "h-1")
            .body("[]");
    });
    let frame = client(&half)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");
    assert!(frame.cursor.is_none(), "a half cursor must not escape");
}

#[tokio::test]
async fn a_409_is_must_refetch_and_withholds_the_cursor() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(409)
            .header("electric-handle", "h-1")
            .header("electric-offset", "17")
            .body("");
    });

    let cursor = ShapeCursor {
        handle: "h-1".to_owned(),
        offset: "17".to_owned(),
    };
    let frame = client(&server)
        .fetch_shape("cases", Some(&cursor), &options())
        .await
        .expect("409 is a protocol outcome, not an error");

    assert!(frame.must_refetch);
    // Returning a cursor would invite a retry that cannot succeed.
    assert!(frame.cursor.is_none());
}

#[tokio::test]
async fn the_must_refetch_header_is_equivalent_to_a_409() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            .header("electric-must-refetch", "true")
            .body("[]");
    });
    let frame = client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");
    assert!(frame.must_refetch);
}

#[tokio::test]
async fn up_to_date_is_read_from_the_headers_presence_not_its_value() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            // Electric sends this with an empty value.
            .header("electric-up-to-date", "")
            .body("[]");
    });
    let frame = client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");
    assert!(frame.up_to_date);
}

#[tokio::test]
async fn a_204_is_an_empty_frame_carrying_a_usable_cursor() {
    let server = MockServer::start();
    // The facade returns 204 when it has metadata but no rows. The Electric
    // protocol treats that as an empty frame, not an error, and the cursor
    // must survive so the caller can continue the stream.
    //
    // This asserts an OUTCOME, not a branch. An earlier version tried to prove
    // a dedicated 204 branch existed by serving an unparseable body — but HTTP
    // forbids a body on 204, so both paths saw `""` and the test passed with
    // the branch deleted. The branch was dead code and has been removed; what
    // remains is the property a caller actually depends on.
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(204)
            .header("electric-handle", "h-1")
            .header("electric-offset", "9");
    });

    let frame = client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("204 is a protocol outcome, not an error");

    assert!(frame.messages.is_empty());
    assert!(!frame.not_modified, "204 is not 304");
    assert_eq!(
        frame.cursor,
        Some(ShapeCursor {
            handle: "h-1".to_owned(),
            offset: "9".to_owned(),
        })
    );
}

#[tokio::test]
async fn a_304_is_not_modified_and_the_body_is_never_read() {
    let server = MockServer::start();
    // Same discriminator as the 204 case.
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(304).body("{not json at all");
    });

    let opts = ShapeRequestOptions {
        if_none_match: Some("etag-1".to_owned()),
        ..ShapeRequestOptions::default()
    };
    let frame = client(&server)
        .fetch_shape("cases", None, &opts)
        .await
        .expect("304 must not attempt to parse a body");

    assert!(frame.not_modified);
    assert!(frame.messages.is_empty());
}

#[tokio::test]
async fn control_frames_are_dropped_and_deletes_are_preserved() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"[
                    {"headers":{"control":"up-to-date"}},
                    {"headers":{"operation":"insert"},"key":"a","value":{"id":"1"}},
                    {"headers":{"operation":"delete"},"key":"b","value":{"id":"2"}}
                ]"#,
            );
    });

    let frame = client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect("fetch succeeds");

    // A replica that rebuilds may ignore deletes; a general client must not
    // make that decision for the caller.
    assert_eq!(frame.messages.len(), 2);
    assert!(frame.messages[1].is_delete());
}

#[tokio::test]
async fn a_malformed_body_is_upstream_not_a_refusal() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/v1/shape");
        then.status(200)
            .header("content-type", "application/json")
            .body("{not json");
    });

    let error = client(&server)
        .fetch_shape("cases", None, &options())
        .await
        .expect_err("must reject");

    // Nothing is wrong with the grant, so this is retryable — not a refusal.
    assert!(matches!(error, SdkError::ShapeUpstream(_)));
}
