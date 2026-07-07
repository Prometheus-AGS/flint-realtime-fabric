use thiserror::Error;

/// Errors returned by the Flint Realtime Fabric Rust client.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SdkError {
    /// Failed to establish the transport connection to the gateway.
    #[error("connect failed: {0}")]
    Connect(String),

    /// The gateway rejected a request (gRPC status).
    #[error("request failed: {0}")]
    Request(String),

    /// A value returned by the gateway could not be converted to a domain type.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// The provided gateway endpoint URL was not valid.
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),
}

impl From<tonic::Status> for SdkError {
    fn from(status: tonic::Status) -> Self {
        SdkError::Request(format!("{}: {}", status.code(), status.message()))
    }
}
