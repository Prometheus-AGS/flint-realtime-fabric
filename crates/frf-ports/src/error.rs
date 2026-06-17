/// Shared port error variants. Adapter crates wrap these or provide their own
/// `thiserror`-derived error types that convert into these.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("transport error: {0}")]
    Transport(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("operation timed out")]
    Timeout,

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("upstream error: {0}")]
    Upstream(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}
