use frf_ports::error::PortError;

/// Errors produced by the Matrix federation bridge.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MatrixBridgeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Matrix event projection failed: {0}")]
    Projection(String),
}

impl From<MatrixBridgeError> for PortError {
    fn from(e: MatrixBridgeError) -> Self {
        PortError::Transport(e.to_string())
    }
}
