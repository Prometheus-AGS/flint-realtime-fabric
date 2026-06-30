use frf_ports::PortError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum LiveKitError {
    #[error("LiveKit API error: {0}")]
    Api(String),
    #[error("signal serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("session not found: {0}")]
    SessionNotFound(String),
}

impl From<LiveKitError> for PortError {
    fn from(e: LiveKitError) -> Self {
        Self::Transport(e.to_string())
    }
}
