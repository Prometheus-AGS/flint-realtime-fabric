use frf_authz_keto::KetoAuthzProvider;
use frf_domain::{ChannelId, TenantId};
use frf_ports::{AuthzProvider, RelationTuple};
use httpmock::prelude::*;
use uuid::Uuid;

fn test_tuple() -> RelationTuple {
    RelationTuple {
        tenant_id: TenantId::from_uuid(Uuid::nil()),
        subject: "user-1".to_owned(),
        relation: "view".to_owned(),
        object: format!("{}", ChannelId::from_uuid(Uuid::nil())),
    }
}

#[tokio::test]
async fn check_returns_true_on_allowed_response() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/relation-tuples/check");
        then.status(200)
            .json_body(serde_json::json!({"allowed": true}));
    });

    let provider = KetoAuthzProvider::new(server.base_url(), "test-ns");
    let result = provider.check(&test_tuple()).await.expect("check failed");
    assert!(result);
}

#[tokio::test]
async fn check_returns_false_on_denied_response() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/relation-tuples/check");
        then.status(200)
            .json_body(serde_json::json!({"allowed": false}));
    });

    let provider = KetoAuthzProvider::new(server.base_url(), "test-ns");
    let result = provider.check(&test_tuple()).await.expect("check failed");
    assert!(!result);
}

#[tokio::test]
async fn cache_hit_skips_http_call() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/relation-tuples/check");
        then.status(200)
            .json_body(serde_json::json!({"allowed": true}));
    });

    let provider = KetoAuthzProvider::new(server.base_url(), "test-ns");
    let tuple = test_tuple();

    provider.check(&tuple).await.expect("first check failed");
    provider.check(&tuple).await.expect("second check failed");

    mock.assert_hits(1);
}

#[tokio::test]
async fn write_calls_put_endpoint() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(PUT).path("/relation-tuples");
        then.status(201);
    });

    let provider = KetoAuthzProvider::new(server.base_url(), "test-ns");
    provider.write(test_tuple()).await.expect("write failed");
    mock.assert_hits(1);
}

#[tokio::test]
async fn delete_calls_delete_endpoint_and_invalidates_cache() {
    let server = MockServer::start();

    let check_mock = server.mock(|when, then| {
        when.method(POST).path("/relation-tuples/check");
        then.status(200)
            .json_body(serde_json::json!({"allowed": true}));
    });

    let delete_mock = server.mock(|when, then| {
        when.method(DELETE).path_contains("/relation-tuples");
        then.status(204);
    });

    let provider = KetoAuthzProvider::new(server.base_url(), "test-ns");
    let tuple = test_tuple();

    // Warm the cache
    provider.check(&tuple).await.expect("check failed");
    check_mock.assert_hits(1);

    // Delete should invalidate cache
    provider.delete(tuple.clone()).await.expect("delete failed");
    delete_mock.assert_hits(1);

    // Now check again — cache miss so HTTP called again
    provider
        .check(&tuple)
        .await
        .expect("post-delete check failed");
    check_mock.assert_hits(2);
}
