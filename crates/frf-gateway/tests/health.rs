use axum_test::TestServer;

#[tokio::test]
async fn healthz_returns_200() {
    let app = frf_gateway::build_router();
    let server = TestServer::new(app).expect("test server");
    let response = server.get("/healthz").await;
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ok");
}
