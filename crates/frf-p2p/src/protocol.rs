//! Bounded application frames over an authenticated QUIC session.
use crate::P2pError;
use iroh::endpoint::{RecvStream, SendStream};

pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_CREDENTIAL_BYTES: usize = 16 * 1024;

/// Reject lengths before allocating remote-controlled payload storage.
pub async fn read_frame(stream: &mut RecvStream, limit: usize) -> Result<Vec<u8>, P2pError> {
    let mut length = [0; 4];
    stream
        .read_exact(&mut length)
        .await
        .map_err(|e| P2pError::Stream(e.to_string()))?;
    let length = u32::from_be_bytes(length) as usize;
    if length > limit.min(MAX_FRAME_BYTES) {
        return Err(P2pError::Codec(
            "peer frame exceeds negotiated bound".into(),
        ));
    }
    let mut bytes = vec![0; length];
    stream
        .read_exact(&mut bytes)
        .await
        .map_err(|e| P2pError::Stream(e.to_string()))?;
    Ok(bytes)
}

/// Write one length-delimited frame. The caller finishes the stream.
pub async fn write_frame(
    stream: &mut SendStream,
    bytes: &[u8],
    limit: usize,
) -> Result<(), P2pError> {
    if bytes.len() > limit.min(MAX_FRAME_BYTES) {
        return Err(P2pError::Codec(
            "peer frame exceeds negotiated bound".into(),
        ));
    }
    let length = u32::try_from(bytes.len()).map_err(|e| P2pError::Codec(e.to_string()))?;
    stream
        .write_all(&length.to_be_bytes())
        .await
        .map_err(|e| P2pError::Stream(e.to_string()))?;
    stream
        .write_all(bytes)
        .await
        .map_err(|e| P2pError::Stream(e.to_string()))
}
