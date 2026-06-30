/// Errors produced by the `LibreFang` actor bus.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LibreFangError {
    #[error("actor spawn failed: {0}")]
    SpawnFailed(String),

    #[error("actor send failed: {0}")]
    SendFailed(String),

    #[error("subscription failed: {0}")]
    SubscriptionFailed(String),
}
