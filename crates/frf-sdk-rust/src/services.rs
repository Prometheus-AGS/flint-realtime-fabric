//! Typed clients for the gateway services beyond `SpineService`.
//!
//! Rust parity with the TS/Go/C# "thin wrapper" factories (p16-c013): each accessor
//! returns the generated tonic client wrapped with the shared bearer-token interceptor.
//! Unlike those SDKs — which excluded Entity/Authz because no gateway server existed —
//! all five services now have live servers (p17-c004/c005), so Entity and Authz are
//! bound here too.

use frf_proto::fv1::agent_service_client::AgentServiceClient;
use frf_proto::fv1::authz_service_client::AuthzServiceClient;
use frf_proto::fv1::entity_service_client::EntityServiceClient;
use frf_proto::fv1::signal_service_client::SignalServiceClient;
use frf_proto::fv1::sync_service_client::SyncServiceClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel as TonicChannel;

use crate::client::AuthInterceptor;
use crate::error::SdkError;

/// A gateway service client with the bearer-token interceptor applied.
type Intercepted = InterceptedService<TonicChannel, AuthInterceptor>;

/// Typed clients for the non-Spine gateway services, sharing one connection and one
/// bearer token. Construct with [`ServiceClients::connect`], then take the client you
/// need via the accessor methods.
pub struct ServiceClients {
    channel: TonicChannel,
    interceptor: AuthInterceptor,
}

impl ServiceClients {
    /// Connect to the gateway and prepare typed clients for every non-Spine service.
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
        Ok(Self {
            channel,
            interceptor: AuthInterceptor { token },
        })
    }

    /// CRDT sync client (`SyncService`).
    #[must_use]
    pub fn sync(&self) -> SyncServiceClient<Intercepted> {
        SyncServiceClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
    }

    /// Agent runs client (`AgentService`).
    #[must_use]
    pub fn agent(&self) -> AgentServiceClient<Intercepted> {
        AgentServiceClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
    }

    /// WebRTC signaling client (`SignalService`).
    #[must_use]
    pub fn signal(&self) -> SignalServiceClient<Intercepted> {
        SignalServiceClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
    }

    /// Entity read/watch client (`EntityService`, live since p17-c004).
    #[must_use]
    pub fn entity(&self) -> EntityServiceClient<Intercepted> {
        EntityServiceClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
    }

    /// Authorization client (`AuthzService`, live since p17-c005).
    #[must_use]
    pub fn authz(&self) -> AuthzServiceClient<Intercepted> {
        AuthzServiceClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_rejects_invalid_endpoint() {
        // A malformed URI fails at URI parsing before any network I/O, so this test
        // needs no live gateway. The `::` scheme is not a valid URI.
        // `ServiceClients` isn't `Debug`, so inspect the error via `.err()` rather than
        // `matches!` on the whole `Result`.
        let err = ServiceClients::connect("::not a uri::", None)
            .await
            .err()
            .expect("invalid endpoint should error");
        assert!(
            matches!(err, SdkError::InvalidEndpoint(_)),
            "expected InvalidEndpoint, got {err:?}"
        );
    }
}
