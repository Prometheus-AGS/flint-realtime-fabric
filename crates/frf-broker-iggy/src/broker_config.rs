use std::sync::Arc;

use frf_domain::Offset;
use iggy::client::Client;
use iggy::clients::client::IggyClient;
use iggy::clients::consumer::AutoCommit;
use iggy::messages::poll_messages::PollingStrategy;
use iggy::utils::duration::IggyDuration;
use iggy::utils::expiry::IggyExpiry;

use crate::broker::IggyBroker;
use crate::error::IggyBrokerError;

pub(crate) const CHANNEL_BUF: usize = 256;
pub(crate) const DEFAULT_REPLAY_RETENTION_SECONDS: u64 = 86_400;

impl IggyBroker {
    /// Connect to Iggy with the default 24-hour replay retention.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection string is invalid or connection fails.
    pub async fn new(connection_string: &str) -> Result<Self, IggyBrokerError> {
        Self::with_replay_retention(connection_string, DEFAULT_REPLAY_RETENTION_SECONDS).await
    }

    /// Connect with the replay retention enforced on created and existing topics.
    ///
    /// # Errors
    ///
    /// Returns an error when retention is zero or connection fails.
    pub async fn with_replay_retention(
        connection_string: &str,
        replay_retention_seconds: u64,
    ) -> Result<Self, IggyBrokerError> {
        if replay_retention_seconds == 0 {
            return Err(IggyBrokerError::Configuration(
                "replay retention must be positive".to_owned(),
            ));
        }
        let client = IggyClient::from_connection_string(connection_string)?;
        client.connect().await?;
        Ok(Self {
            client: Arc::new(client),
            connection_string: Arc::from(connection_string),
            replay_retention_seconds,
        })
    }
}

pub(crate) fn replay_retention(seconds: u64) -> IggyExpiry {
    IggyExpiry::ExpireDuration(IggyDuration::new_from_secs(seconds))
}

pub(crate) const fn consumer_commit_policy() -> AutoCommit {
    AutoCommit::Disabled
}

pub(crate) fn polling_strategy(from: Offset) -> PollingStrategy {
    if from == Offset::BEGINNING {
        PollingStrategy::first()
    } else {
        // Iggy's explicit offset strategy is inclusive. Callers that persist the
        // last processed position resume with `last.next()`.
        PollingStrategy::offset(from.0)
    }
}
