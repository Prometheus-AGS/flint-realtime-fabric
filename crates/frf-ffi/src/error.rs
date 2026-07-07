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

/// Errors surfaced by the client transport FFI (connect/publish/subscribe).
#[derive(Debug, Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum ClientFfiError {
    #[error("connect failed: {0}")]
    Connect(String),

    #[error("request failed: {0}")]
    Request(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

impl From<frf_sdk_rust::SdkError> for ClientFfiError {
    fn from(e: frf_sdk_rust::SdkError) -> Self {
        use frf_sdk_rust::SdkError;
        match e {
            SdkError::Connect(m) | SdkError::InvalidEndpoint(m) => ClientFfiError::Connect(m),
            SdkError::Request(m) | SdkError::InvalidResponse(m) => ClientFfiError::Request(m),
            _ => ClientFfiError::Request(e.to_string()),
        }
    }
}
