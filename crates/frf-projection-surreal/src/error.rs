use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SurrealProjectionError {
    #[error("surrealdb projection operation failed: {0}")]
    Database(#[from] surrealdb::Error),
    #[error("stored projection data is invalid: {0}")]
    InvalidData(String),
}

impl From<SurrealProjectionError> for frf_ports::PortError {
    fn from(error: SurrealProjectionError) -> Self {
        Self::Transport(error.to_string())
    }
}
