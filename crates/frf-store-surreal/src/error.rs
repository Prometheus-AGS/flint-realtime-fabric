use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SurrealStoreError {
    #[error("surrealdb error: {0}")]
    Db(#[from] surrealdb::Error),

    #[error("serialization error: {0}")]
    Serde(String),

    #[error("port error: {0}")]
    Port(#[from] frf_ports::PortError),
}

impl From<SurrealStoreError> for frf_ports::PortError {
    fn from(e: SurrealStoreError) -> Self {
        Self::Transport(e.to_string())
    }
}
