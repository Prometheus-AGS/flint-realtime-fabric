//! Errors for the Electric shape facade adapter.

use frf_ports::PortError;

/// Failures in the upstream Electric exchange.
///
/// Messages deliberately carry no row data, token or clinical text — replica diagnostics are
/// limited to counts, durations, schema versions, anonymized correlation and error classes
/// (ASO runtime architecture §12). A rejected parameter names only its *key*, never its value.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ShapeError {
    /// The upstream Electric exchange failed.
    #[error("electric upstream error: {0}")]
    Upstream(String),
}

impl From<ShapeError> for PortError {
    fn from(e: ShapeError) -> Self {
        match e {
            ShapeError::Upstream(m) => Self::Transport(m),
        }
    }
}
