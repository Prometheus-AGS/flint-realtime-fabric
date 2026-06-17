use frf_ports::PortError;
use iggy::prelude::IggyError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum IggyBrokerError {
    #[error("iggy transport: {0}")]
    Transport(#[from] IggyError),

    #[error("serialization: {0}")]
    Serialization(String),

    #[error("not found: {0}")]
    NotFound(String),
}

impl From<IggyBrokerError> for PortError {
    fn from(err: IggyBrokerError) -> Self {
        match err {
            IggyBrokerError::NotFound(msg) => PortError::NotFound(msg),
            IggyBrokerError::Serialization(msg) => PortError::Serialization(msg),
            IggyBrokerError::Transport(e) => PortError::Transport(e.to_string()),
        }
    }
}
