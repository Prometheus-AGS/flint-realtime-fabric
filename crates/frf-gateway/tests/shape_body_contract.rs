use axum::body::{Body, to_bytes};
use bytes::Bytes;
use frf_ports::{PortError, ShapeBodyStream};
use futures_util::stream;

#[tokio::test]
async fn locked_axum_body_accepts_the_shell_neutral_shape_stream() -> Result<(), axum::Error> {
    let frames: ShapeBodyStream = Box::pin(stream::iter([
        Ok::<Bytes, PortError>(Bytes::from_static(b"frame-1")),
        Ok::<Bytes, PortError>(Bytes::from_static(b"frame-2")),
    ]));

    let body = Body::from_stream(frames);
    let produced = to_bytes(body, 14).await?;

    assert_eq!(produced, Bytes::from_static(b"frame-1frame-2"));
    Ok(())
}
