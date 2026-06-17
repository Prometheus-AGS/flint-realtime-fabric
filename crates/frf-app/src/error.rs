use frf_ports::PortError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("broker error: {0}")]
    Broker(#[from] PortError),

    #[error("identity error: {0}")]
    Identity(PortError),
}
