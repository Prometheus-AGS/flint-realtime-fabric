use std::sync::Arc;
use std::{future::Future, pin::Pin};

use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, Offset};
use frf_ports::{EventStream, LogBroker, PortError};
use futures_util::StreamExt;
use iggy::client::{Client, ConsumerOffsetClient, StreamClient, TopicClient};
use iggy::clients::client::IggyClient;
use iggy::clients::consumer::AutoCommit;
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::consumer::Consumer;
use iggy::error::IggyError;
use iggy::messages::poll_messages::PollingStrategy;
use iggy::messages::send_messages::Message as IggyMessage;
use iggy::utils::duration::IggyDuration;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;

use crate::channel::partition_id;
use crate::error::IggyBrokerError;
use crate::position::{
    locate_message, new_message_range, partition_snapshot, validate_first_delivery,
};

const CHANNEL_BUF: usize = 256;
const REPLAY_RETENTION_SECONDS: u64 = 86_400;

fn replay_retention() -> IggyExpiry {
    IggyExpiry::ExpireDuration(IggyDuration::new_from_secs(REPLAY_RETENTION_SECONDS))
}

const fn consumer_commit_policy() -> AutoCommit {
    AutoCommit::Disabled
}

fn polling_strategy(from: Offset) -> PollingStrategy {
    if from == Offset::BEGINNING {
        PollingStrategy::first()
    } else {
        // Iggy's explicit offset strategy is inclusive. Callers that persist the
        // last processed position resume with `last.next()`.
        PollingStrategy::offset(from.0)
    }
}

fn encode_message(envelope: &EventEnvelope) -> Result<IggyMessage, PortError> {
    let payload =
        serde_json::to_vec(envelope).map_err(|e| PortError::Serialization(e.to_string()))?;
    // The domain event ID survives producer retry and restart. Supplying it to
    // Iggy activates the broker's persisted message-ID deduplicator.
    let message_id = envelope.id.as_uuid().as_u128();
    Ok(IggyMessage::new(
        Some(message_id),
        Bytes::from(payload),
        None,
    ))
}

fn decode_message(payload: &[u8], broker_offset: u64) -> Result<EventEnvelope, PortError> {
    let mut envelope: EventEnvelope =
        serde_json::from_slice(payload).map_err(|e| PortError::Serialization(e.to_string()))?;
    // The serialized value is a producer-local source hint. Replay and client
    // checkpoints must use the position assigned by the broker partition.
    envelope.offset = Offset(broker_offset);
    Ok(envelope)
}

async fn next_while_open<T>(
    tx: &mpsc::Sender<Result<EventEnvelope, PortError>>,
    next: Pin<&mut (dyn Future<Output = T> + Send)>,
) -> Option<T> {
    tokio::select! {
        biased;
        () = tx.closed() => None,
        result = next => Some(result),
    }
}

pub struct IggyBroker {
    client: Arc<IggyClient>,
    connection_string: Arc<str>,
}

impl IggyBroker {
    /// Connect to an Iggy server via a connection string.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection string is invalid or the connection
    /// cannot be established.
    pub async fn new(connection_string: &str) -> anyhow::Result<Self> {
        let client = IggyClient::from_connection_string(connection_string)?;
        client.connect().await?;
        Ok(Self {
            client: Arc::new(client),
            connection_string: Arc::from(connection_string),
        })
    }

    /// Read the stored consumer offset for a channel/consumer, if one has been recorded.
    ///
    /// This is the read counterpart to [`LogBroker::seek`]. It is an inherent method
    /// (not part of the `LogBroker` port) so the port stays minimal; operational tools
    /// like `frf-cli` use the adapter directly. Returns `Ok(None)` when Iggy has no
    /// stored offset for the pair yet.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the Iggy offset lookup fails for a reason
    /// other than "no offset stored".
    #[instrument(name = "IggyBroker::get_consumer_offset", skip(self))]
    pub async fn get_consumer_offset(
        &self,
        channel_id: ChannelId,
        consumer_id: &str,
    ) -> Result<Option<Offset>, PortError> {
        let stream = format!("channel-{channel_id}");
        let topic = "events";
        let partition = partition_id(consumer_id);

        let stream_id = stream
            .as_str()
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;
        let topic_id = topic
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;
        let iggy_consumer = Consumer::new(
            consumer_id
                .try_into()
                .map_err(|e: IggyError| IggyBrokerError::Transport(e))?,
        );

        let stored = self
            .client
            .get_consumer_offset(&iggy_consumer, &stream_id, &topic_id, Some(partition))
            .await
            .map_err(IggyBrokerError::Transport)?;

        Ok(stored.map(|info| Offset(info.stored_offset)))
    }

    /// Create `stream` and `topic` if they do not already exist.
    ///
    /// Shared by [`LogBroker::publish`] and [`LogBroker::ensure_channel`] so both use
    /// one definition of "exists". Idempotent: an already-existing stream or topic is
    /// treated as success, not as an error.
    async fn create_stream_and_topic(&self, stream: &str, topic: &str) -> Result<(), PortError> {
        match self.client.create_stream(stream, None).await {
            Ok(_) | Err(IggyError::StreamNameAlreadyExists(_)) => {}
            Err(e) => return Err(IggyBrokerError::Transport(e).into()),
        }

        let stream_id = stream
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;

        match self
            .client
            .create_topic(
                &stream_id,
                topic,
                1,
                CompressionAlgorithm::None,
                None,
                None,
                replay_retention(),
                MaxTopicSize::ServerDefault,
            )
            .await
        {
            Ok(_) => {}
            Err(IggyError::TopicNameAlreadyExists(_, _)) => {
                // Existing deployments may have the former unbounded topic.
                // Preserve all other settings while converging retention to the
                // production replay contract.
                let topic_id = topic
                    .try_into()
                    .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;
                let current = self
                    .client
                    .get_topic(&stream_id, &topic_id)
                    .await
                    .map_err(IggyBrokerError::Transport)?
                    .ok_or_else(|| {
                        IggyBrokerError::NotFound(format!(
                            "topic {stream}/{topic} disappeared while applying replay retention"
                        ))
                    })?;

                if current.message_expiry != replay_retention() {
                    self.client
                        .update_topic(
                            &stream_id,
                            &topic_id,
                            &current.name,
                            current.compression_algorithm,
                            Some(current.replication_factor),
                            replay_retention(),
                            current.max_topic_size,
                        )
                        .await
                        .map_err(IggyBrokerError::Transport)?;
                }
            }
            Err(e) => return Err(IggyBrokerError::Transport(e).into()),
        }

        Ok(())
    }

    async fn reject_expired_cursor(
        &self,
        stream: &str,
        topic: &str,
        partition: u32,
        from: Offset,
    ) -> Result<(), PortError> {
        if from == Offset::BEGINNING {
            return Ok(());
        }

        let details = partition_snapshot(&self.client, stream, topic, partition).await?;
        let floor = details.retained_floor();
        if from < floor {
            return Err(PortError::NotFound(format!(
                "resnapshot_required: requested broker offset {} precedes retained floor {}",
                from.0, floor.0
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl LogBroker for IggyBroker {
    /// Publish an event to a channel. Returns its authoritative broker `Offset`.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Serialization`] if the envelope cannot be JSON-encoded.
    /// Returns [`PortError::Transport`] if the Iggy producer fails.
    #[instrument(name = "port::LogBroker::publish", skip(self, envelope))]
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
        let stream = format!("channel-{}", envelope.channel.id);
        let topic = "events";

        // Create the stream+topic if absent. Without this, publishing to a channel that
        // was never `ensure_channel`ed fails at `producer.init()`, making delivery depend
        // on boot ordering. The stream name derives from the channel id alone, so no
        // tenant or path is needed here — the same reason `subscribe` can address it.
        self.create_stream_and_topic(&stream, topic).await?;
        let partition = partition_id("publish-confirmation");
        let before = partition_snapshot(&self.client, &stream, topic, partition).await?;

        let msg = encode_message(&envelope)?;
        let message_id = msg.id;

        let mut producer = self
            .client
            .producer(&stream, topic)
            .map_err(IggyBrokerError::Transport)?
            .build();

        producer.init().await.map_err(IggyBrokerError::Transport)?;

        producer
            .send(vec![msg])
            .await
            .map_err(IggyBrokerError::Transport)?;

        let after = partition_snapshot(&self.client, &stream, topic, partition).await?;
        if let Some(range) = new_message_range(before, after)
            && let Some(offset) =
                locate_message(&self.client, &stream, topic, partition, message_id, range).await?
        {
            return Ok(offset);
        }

        let retained = (after.retained_floor(), Offset(after.current_offset));
        locate_message(
            &self.client,
            &stream,
            topic,
            partition,
            message_id,
            retained,
        )
        .await?
        .ok_or_else(|| {
            PortError::Transport(format!(
                "broker accepted message {message_id} but its retained position was not found"
            ))
        })
    }

    /// Open a streaming subscription starting from `from`.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the Iggy consumer cannot be built or initialized.
    #[instrument(name = "port::LogBroker::subscribe", skip(self))]
    async fn subscribe(
        &self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError> {
        let stream = format!("channel-{channel_id}");
        let topic = "events".to_owned();
        let partition = partition_id(&consumer_id);

        self.reject_expired_cursor(&stream, &topic, partition, from)
            .await?;
        let strategy = polling_strategy(from);

        // Long polling owns a dedicated transport connection. Cancelling an
        // in-flight poll on the shared command connection leaves its eventual
        // response queued for the next command in the pinned TCP client, which
        // can corrupt that response. Closing this dedicated client on receiver
        // drop cancels the server request without poisoning publish/admin calls.
        let subscription_client = IggyClient::from_connection_string(&self.connection_string)
            .map_err(IggyBrokerError::Transport)?;
        subscription_client
            .connect()
            .await
            .map_err(IggyBrokerError::Transport)?;
        let mut consumer = subscription_client
            .consumer(&consumer_id, &stream, &topic, partition)
            .map_err(IggyBrokerError::Transport)?
            .polling_strategy(strategy)
            // Delivery is acknowledged only through `LogBroker::ack`; polling
            // must never advance the durable consumer checkpoint.
            .auto_commit(consumer_commit_policy())
            .build();

        consumer.init().await.map_err(IggyBrokerError::Transport)?;

        let (tx, rx) = mpsc::channel(CHANNEL_BUF);

        tokio::spawn(async move {
            let mut first_delivery = true;
            loop {
                let mut next = Box::pin(consumer.next());
                let Some(result) = next_while_open(&tx, next.as_mut()).await else {
                    break;
                };
                match result {
                    Some(Ok(msg)) => {
                        if first_delivery {
                            first_delivery = false;
                            if let Err(error) =
                                validate_first_delivery(from, Offset(msg.message.offset))
                            {
                                let _ = tx.send(Err(error)).await;
                                break;
                            }
                        }
                        let item = decode_message(&msg.message.payload, msg.message.offset);
                        if tx.send(item).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        let _ = tx.send(Err(PortError::Transport(e.to_string()))).await;
                        break;
                    }
                    None => break,
                }
            }
            if let Err(error) = subscription_client.disconnect().await {
                tracing::warn!(%error, "failed to close Iggy subscription connection");
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }

    /// Seek a named cursor to an explicit offset.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the Iggy offset store fails.
    #[instrument(name = "port::LogBroker::seek", skip(self))]
    async fn seek(&self, cursor: Cursor) -> Result<(), PortError> {
        let stream = format!("channel-{}", cursor.channel_id);
        let topic = "events";
        let partition = partition_id(&cursor.consumer_id);

        let stream_id = stream
            .as_str()
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;
        let topic_id = topic
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;

        let iggy_consumer = Consumer::new(
            cursor
                .consumer_id
                .as_str()
                .try_into()
                .map_err(|e: IggyError| IggyBrokerError::Transport(e))?,
        );

        self.client
            .store_consumer_offset(
                &iggy_consumer,
                &stream_id,
                &topic_id,
                Some(partition),
                cursor.offset.0,
            )
            .await
            .map_err(IggyBrokerError::Transport)?;

        Ok(())
    }

    /// Acknowledge delivery up to and including `offset` for a consumer.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the Iggy offset store fails.
    #[instrument(name = "port::LogBroker::ack", skip(self))]
    async fn ack(
        &self,
        channel_id: ChannelId,
        consumer_id: &str,
        offset: Offset,
    ) -> Result<(), PortError> {
        let stream = format!("channel-{channel_id}");
        let topic = "events";
        let partition = partition_id(consumer_id);

        let stream_id = stream
            .as_str()
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;
        let topic_id = topic
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;

        let iggy_consumer = Consumer::new(
            consumer_id
                .try_into()
                .map_err(|e: IggyError| IggyBrokerError::Transport(e))?,
        );

        self.client
            .store_consumer_offset(
                &iggy_consumer,
                &stream_id,
                &topic_id,
                Some(partition),
                offset.0,
            )
            .await
            .map_err(IggyBrokerError::Transport)?;

        Ok(())
    }

    /// Ensure the channel exists; create stream and topic if absent.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the Iggy create calls fail with
    /// an error other than "already exists".
    #[instrument(name = "port::LogBroker::ensure_channel", skip(self))]
    async fn ensure_channel(&self, channel: Channel) -> Result<(), PortError> {
        let stream = format!("channel-{}", channel.id);
        self.create_stream_and_topic(&stream, "events").await
    }
}

#[cfg(test)]
#[path = "broker_tests.rs"]
mod tests;
