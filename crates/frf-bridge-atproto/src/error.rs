use frf_ports::error::PortError;

/// Errors produced by the ATProto/Bluesky federation bridge.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AtProtoBridgeError {
    #[error("WebSocket connection error: {0}")]
    WebSocket(String),

    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ATProto event projection failed: {0}")]
    Projection(String),
}

impl From<AtProtoBridgeError> for PortError {
    fn from(e: AtProtoBridgeError) -> Self {
        PortError::Transport(e.to_string())
    }
}
