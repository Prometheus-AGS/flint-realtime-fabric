use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CrdtError {
    #[error("loro encode error: {0}")]
    Encode(String),

    #[error("loro decode error: {0}")]
    Decode(String),

    #[error("merge failed: {0}")]
    Merge(String),

    #[error("port error: {0}")]
    Port(#[from] frf_ports::PortError),
}
