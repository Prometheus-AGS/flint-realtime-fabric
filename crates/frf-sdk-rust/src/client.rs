use frf_domain::{ChannelId, EventEnvelope, Offset};
use frf_proto::fv1;
use frf_proto::fv1::spine_service_client::SpineServiceClient;
use futures_util::Stream;
use futures_util::StreamExt as _;
use tonic::metadata::MetadataValue;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel as TonicChannel;
use tonic::{Request, Status};

use crate::convert::{envelope_from_proto, envelope_to_proto};
use crate::error::SdkError;

/// Interceptor that attaches a `Bearer` token to every request, when set.
///
/// Public because it appears in the transport type of the [`crate::ServiceClients`]
/// accessors (`InterceptedService<Channel, AuthInterceptor>`); callers do not construct
/// it directly.
#[derive(Clone, Default)]
pub struct AuthInterceptor {
    pub(crate) token: Option<String>,
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(token) = &self.token {
            let value = format!("Bearer {token}");
            match MetadataValue::try_from(value) {
                Ok(v) => {
                    req.metadata_mut().insert("authorization", v);
                }
                Err(_) => return Err(Status::invalid_argument("invalid bearer token")),
            }
        }
        Ok(req)
    }
}

type SpineClient = SpineServiceClient<InterceptedService<TonicChannel, AuthInterceptor>>;

/// Hand-written Rust client for Flint Realtime Fabric.
///
/// The single home for connection lifecycle and the publish/subscribe API over
/// the gateway's `SpineService`. Reconnection/backoff and CRDT merge are layered
/// on top of this core in later changes (p16-c012).
pub struct FrfClient {
    spine: SpineClient,
}

impl FrfClient {
    /// Connect to the gateway's gRPC endpoint, optionally authenticated with a
    /// bearer token.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::InvalidEndpoint`] if `endpoint` is not a valid URI and
    /// [`SdkError::Connect`] if the transport connection cannot be established.
    pub async fn connect(
        endpoint: impl Into<String>,
        token: Option<String>,
    ) -> Result<Self, SdkError> {
        let endpoint = endpoint.into();
        let channel = TonicChannel::from_shared(endpoint.clone())
            .map_err(|e| SdkError::InvalidEndpoint(format!("{endpoint}: {e}")))?
            .connect()
            .await
            .map_err(|e| SdkError::Connect(e.to_string()))?;

        let interceptor = AuthInterceptor { token };
        let spine = SpineServiceClient::with_interceptor(channel, interceptor);
        Ok(Self { spine })
    }

    /// Publish an event to the spine, returning the assigned offset.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Request`] if the gateway rejects the publish, and
    /// [`SdkError::InvalidResponse`] if the response is malformed.
    pub async fn publish(&mut self, envelope: &EventEnvelope) -> Result<Offset, SdkError> {
        let request = fv1::PublishRequest {
            envelope: Some(envelope_to_proto(envelope)),
        };
        let response = self.spine.publish(request).await?.into_inner();
        let offset = response
            .offset
            .ok_or_else(|| SdkError::InvalidResponse("missing offset in response".to_owned()))?;
        Ok(Offset(offset.value))
    }

    /// Subscribe to a channel, returning a stream of domain [`EventEnvelope`]s
    /// starting from `from`.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Request`] if the subscribe RPC fails to open.
    pub async fn subscribe(
        &mut self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<impl Stream<Item = Result<EventEnvelope, SdkError>> + use<>, SdkError> {
        let request = fv1::SubscribeRequest {
            channel_id: channel_id.to_string(),
            consumer_id,
            from: Some(fv1::Offset { value: from.0 }),
        };
        // `subscribe` clones the underlying channel handle, so the returned
        // Streaming owns its transport — it does NOT borrow `self`. Cloning the
        // client for the call frees `self` for concurrent publish/ack.
        let mut spine = self.spine.clone();
        let stream = spine.subscribe(request).await?.into_inner();
        Ok(stream.map(|item| match item {
            Ok(proto) => envelope_from_proto(proto),
            Err(status) => Err(SdkError::from(status)),
        }))
    }

    /// Acknowledge consumption up to `offset` for a channel/consumer.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Request`] if the ack RPC fails.
    pub async fn ack(
        &mut self,
        channel_id: ChannelId,
        consumer_id: impl Into<String>,
        offset: Offset,
    ) -> Result<(), SdkError> {
        let request = fv1::AckRequest {
            channel_id: channel_id.to_string(),
            consumer_id: consumer_id.into(),
            offset: Some(fv1::Offset { value: offset.0 }),
        };
        self.spine.ack(request).await?;
        Ok(())
    }
}
