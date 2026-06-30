#![deny(warnings)]
#![warn(clippy::pedantic)]

use frf_ports::PortError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum StrOmError {
    #[error("ICE negotiation failed: {0}")]
    Ice(String),
    #[error("DTLS handshake failed: {0}")]
    Dtls(String),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("signal serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("port error: {0}")]
    Port(#[from] PortError),
}

impl From<StrOmError> for PortError {
    fn from(e: StrOmError) -> Self {
        Self::Transport(e.to_string())
    }
}
