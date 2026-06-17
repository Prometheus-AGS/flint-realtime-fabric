use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, Offset};
use frf_ports::{EventStream, LogBroker, PortError};
use futures_util::StreamExt;
use iggy::prelude::{
    Client, CompressionAlgorithm, Consumer, ConsumerOffsetClient, IggyClient, IggyError,
    IggyExpiry, IggyMessage, MaxTopicSize, PollingStrategy, StreamClient, TopicClient,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;

use crate::channel::{partition_id, stream_name, topic_name};
use crate::error::IggyBrokerError;

const CHANNEL_BUF: usize = 256;

pub struct IggyBroker {
    client: Arc<IggyClient>,
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
        })
    }
}

#[async_trait]
impl LogBroker for IggyBroker {
    /// Publish an event to a channel. Returns the assigned `Offset`.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Serialization`] if the envelope cannot be JSON-encoded.
    /// Returns [`PortError::Transport`] if the Iggy producer fails.
    #[instrument(name = "port::LogBroker::publish", skip(self, envelope))]
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError> {
        let stream = stream_name(envelope.channel.tenant_id);
        let topic = topic_name(&envelope.channel.path);

        let payload =
            serde_json::to_vec(&envelope).map_err(|e| PortError::Serialization(e.to_string()))?;

        let msg = IggyMessage::from(payload);

        let producer = self
            .client
            .producer(&stream, &topic)
            .map_err(IggyBrokerError::Transport)?
            .build();

        producer.init().await.map_err(IggyBrokerError::Transport)?;

        producer
            .send(vec![msg])
            .await
            .map_err(IggyBrokerError::Transport)?;

        Ok(envelope.offset)
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

        let strategy = if from == Offset::BEGINNING {
            PollingStrategy::first()
        } else {
            PollingStrategy::offset(from.0)
        };

        let mut consumer = self
            .client
            .consumer(&consumer_id, &stream, &topic, partition)
            .map_err(IggyBrokerError::Transport)?
            .polling_strategy(strategy)
            .build();

        consumer.init().await.map_err(IggyBrokerError::Transport)?;

        let (tx, rx) = mpsc::channel(CHANNEL_BUF);

        tokio::spawn(async move {
            while let Some(result) = consumer.next().await {
                match result {
                    Ok(msg) => {
                        let decoded: Result<EventEnvelope, _> =
                            serde_json::from_slice(&msg.message.payload);
                        let item = decoded.map_err(|e| PortError::Serialization(e.to_string()));
                        if tx.send(item).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(PortError::Transport(e.to_string()))).await;
                        break;
                    }
                }
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
        let stream = stream_name(channel.tenant_id);
        let topic = topic_name(&channel.path);

        match self.client.create_stream(&stream).await {
            Ok(_) | Err(IggyError::StreamNameAlreadyExists(_)) => {}
            Err(e) => return Err(IggyBrokerError::Transport(e).into()),
        }

        let stream_id = stream
            .as_str()
            .try_into()
            .map_err(|e: IggyError| IggyBrokerError::Transport(e))?;

        match self
            .client
            .create_topic(
                &stream_id,
                &topic,
                1,
                CompressionAlgorithm::None,
                None,
                IggyExpiry::NeverExpire,
                MaxTopicSize::ServerDefault,
            )
            .await
        {
            Ok(_) | Err(IggyError::TopicNameAlreadyExists(_, _)) => {}
            Err(e) => return Err(IggyBrokerError::Transport(e).into()),
        }

        Ok(())
    }
}
