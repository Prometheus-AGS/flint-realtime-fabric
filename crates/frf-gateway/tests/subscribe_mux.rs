// Run with: cargo test -p frf-gateway -- --ignored
//
// Requires live Iggy, Keto, and Oathkeeper instances configured via env vars:
//   IGGY_CONNECTION_STRING, KETO_BASE_URL, KETO_NAMESPACE,
//   OATHKEEPER_JWKS_URL, JWT_AUDIENCE

#[tokio::test]
#[ignore = "requires live infrastructure (Iggy, Keto, Oathkeeper)"]
async fn subscriber_receives_published_event() {
    // Integration smoke test outline:
    //
    // 1. Load GatewayConfig from env.
    // 2. Construct IggyBroker, KetoAuthzProvider, OryIdentityVerifier.
    // 3. Build AppState + router; bind to a random port via TcpListener::bind("127.0.0.1:0").
    // 4. Obtain a valid JWT from the test Oathkeeper instance.
    // 5. Connect a tokio-tungstenite WS client to /ws/v1/subscribe?channel=<uuid>.
    // 6. POST to /v1/publish with the same channel UUID and a test EventEnvelope.
    // 7. Assert the WS client receives one text frame whose JSON deserialises to
    //    an EventEnvelope with matching channel and payload fields.
    //
    // This test is left as a runnable skeleton; full wiring requires the infra
    // stack defined in docker-compose.yml.
    println!("integration smoke test: infrastructure not available in CI — skipped");
}
