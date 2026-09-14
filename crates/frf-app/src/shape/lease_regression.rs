use super::*;

#[tokio::test(start_paused = true)]
async fn receiver_rejects_a_buffered_frame_at_deadline_before_worker_runs() {
    let deadline = Instant::now() + Duration::from_millis(10);
    let (sender, receiver) = mpsc::channel(1);
    sender
        .try_send(BodyMessage::Frame(Ok(Bytes::from_static(b"late"))))
        .expect("buffer one protected frame");
    drop(sender);
    let cancelled = Arc::new(AtomicBool::new(false));
    let settlement = Arc::new(StreamSettlement::new(|_| {}));
    let task = tokio::spawn(std::future::pending());
    let mut protected = ProtectedShapeStream {
        receiver,
        cancelled,
        cancellation_reported: false,
        deadline,
        settlement,
        task,
    };
    tokio::time::advance(Duration::from_millis(10)).await;
    let result = protected.next().await;
    assert!(matches!(
        result,
        Some(Err(PortError::PermissionDenied(message)))
            if message == "shape response authority lease ended"
    ));
}
