use thiserror::Error;

#[derive(Debug, Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum CrdtFfiError {
    #[error("apply_delta failed: {0}")]
    ApplyDelta(String),

    #[error("encode failed: {0}")]
    Encode(String),

    #[error("decode failed: {0}")]
    Decode(String),
}
