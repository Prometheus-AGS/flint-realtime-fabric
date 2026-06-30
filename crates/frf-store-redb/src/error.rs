use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RedbOpStoreError {
    #[error("redb database error: {0}")]
    Database(#[from] redb::DatabaseError),

    #[error("redb table error: {0}")]
    Table(#[from] redb::TableError),

    #[error("redb transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),

    #[error("redb storage error: {0}")]
    Storage(#[from] redb::StorageError),

    #[error("redb commit error: {0}")]
    Commit(#[from] redb::CommitError),

    #[error("port error: {0}")]
    Port(#[from] frf_ports::PortError),
}

impl From<RedbOpStoreError> for frf_ports::PortError {
    fn from(e: RedbOpStoreError) -> Self {
        Self::Transport(e.to_string())
    }
}
