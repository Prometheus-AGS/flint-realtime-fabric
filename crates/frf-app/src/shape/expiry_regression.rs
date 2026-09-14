use super::*;

#[tokio::test(start_paused = true)]
async fn subsecond_request_time_does_not_extend_the_grant_past_expiry() {
    let (use_case, _) = use_case(response("short-grant-handle"));
    let mut short_grant = claims("subject-1", 10);
    short_grant.expires_at = 1_001;
    let mut response = use_case
        .execute(
            &short_grant,
            initial_request(),
            ShapeRequestTime::new(Duration::from_millis(1_000_900), Instant::now()),
        )
        .await
        .expect("grant is still current for one tenth of a second");
    tokio::task::yield_now().await;

    tokio::time::advance(Duration::from_millis(100)).await;

    assert!(matches!(
        response.body.next().await,
        Some(Err(PortError::PermissionDenied(message)))
            if message == "shape response authority lease ended"
    ));
}

#[tokio::test(start_paused = true)]
async fn delay_after_request_clock_sample_does_not_extend_grant_expiry() {
    let (use_case, _) = use_case(response("delayed-short-grant-handle"));
    let mut short_grant = claims("subject-1", 10);
    short_grant.expires_at = 1_001;
    let request_epoch = Duration::from_millis(1_000_900);
    let request_time = ShapeRequestTime::new(request_epoch, Instant::now());

    tokio::time::advance(Duration::from_millis(200)).await;

    let result = use_case
        .execute(&short_grant, initial_request(), request_time)
        .await;
    assert!(matches!(result, Err(ShapeUseCaseError::GrantExpired)));
}
