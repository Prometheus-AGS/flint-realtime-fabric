/// Errors produced when converting between proto-generated and domain types.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AgentProtoError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("content serialization failed: {0}")]
    ContentSerde(#[from] serde_json::Error),
}
