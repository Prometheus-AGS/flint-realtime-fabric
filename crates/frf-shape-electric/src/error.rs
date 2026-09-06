//! Errors for the Electric shape facade adapter.

use frf_ports::PortError;

/// Failures in policy resolution or the upstream Electric exchange.
///
/// Messages deliberately carry no row data, token or clinical text — replica diagnostics are
/// limited to counts, durations, schema versions, anonymized correlation and error classes
/// (ASO runtime architecture §12). A rejected parameter names only its *key*, never its value.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ShapeError {
    /// The requested shape id is not declared in server policy.
    #[error("unknown shape: {0}")]
    UnknownShape(String),

    /// A client parameter key is not in the shape's allow-list.
    #[error("parameter not allowed for this shape: {0}")]
    ParamNotAllowed(String),

    /// A parameter key or value is not a safe literal.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),

    /// The shape catalog could not be parsed.
    #[error("shape policy error: {0}")]
    Policy(String),

    /// The subject is not authorized for this shape's scope.
    #[error("not authorized for shape scope")]
    Unauthorized,

    /// The upstream Electric exchange failed.
    #[error("electric upstream error: {0}")]
    Upstream(String),
}

impl From<ShapeError> for PortError {
    fn from(e: ShapeError) -> Self {
        match e {
            ShapeError::UnknownShape(s) => Self::NotFound(s),
            ShapeError::Unauthorized => Self::PermissionDenied("shape scope".to_owned()),
            ShapeError::ParamNotAllowed(_)
            | ShapeError::InvalidParam(_)
            | ShapeError::Policy(_) => Self::Serialization(e.to_string()),
            ShapeError::Upstream(m) => Self::Transport(m),
        }
    }
}
