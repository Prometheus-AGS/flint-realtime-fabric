#![allow(clippy::expect_used)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use frf_app::{AppError, SubscribePipeline, SubscribeRequest};
use frf_authz_keto::KetoAuthzProvider;
use frf_domain::ids::EventId;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, EventKind, Offset, TenantId};
use frf_identity_ory::OryIdentityVerifier;
use frf_ports::{EventStream, IdentityVerifier, LogBroker, PortError};
use futures_util::StreamExt as _;
use uuid::Uuid;

const CHANNEL_UUID: Uuid = Uuid::from_u128(0xc005_0001);
const ALLOWED_EVENT_UUID: Uuid = Uuid::from_u128(0xc005_0002);
const DENIED_EVENT_UUID: Uuid = Uuid::from_u128(0xc005_0003);

struct OneEventBroker(Mutex<Option<EventEnvelope>>);

#[async_trait]
impl LogBroker for OneEventBroker {
    async fn publish(&self, _envelope: EventEnvelope) -> Result<Offset, PortError> {
        Err(PortError::Transport("fixture is read-only".to_owned()))
    }

    async fn subscribe(
        &self,
        _channel_id: ChannelId,
        _consumer_id: String,
        _from: Offset,
    ) -> Result<EventStream, PortError> {
        let event = self
            .0
            .lock()
            .map_err(|_| PortError::Transport("fixture lock poisoned".to_owned()))?
            .take();
        Ok(Box::pin(futures_util::stream::iter(
            event.into_iter().map(Ok),
        )))
    }

    async fn seek(&self, _cursor: Cursor) -> Result<(), PortError> {
        Ok(())
    }

    async fn ack(
        &self,
        _channel_id: ChannelId,
        _consumer_id: &str,
        _offset: Offset,
    ) -> Result<(), PortError> {
        Ok(())
    }

    async fn ensure_channel(&self, _channel: Channel) -> Result<(), PortError> {
        Ok(())
    }
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required by the owned runner"))
}

fn event(tenant_id: TenantId, event_id: Uuid) -> EventEnvelope {
    let mut envelope = EventEnvelope::new(
        Channel {
            id: ChannelId::from_uuid(CHANNEL_UUID),
            tenant_id,
            path: "authority/c005".to_owned(),
        },
        Offset(1),
        EventKind::EntityChange,
        serde_json::json!({"protected": true}),
    );
    envelope.id = EventId::from_uuid(event_id);
    envelope
}

async fn pipeline(
    token: &str,
    event: EventEnvelope,
    verifier: Arc<OryIdentityVerifier>,
    authz: Arc<KetoAuthzProvider>,
) -> Result<EventStream, AppError> {
    SubscribePipeline::new(
        Arc::new(OneEventBroker(Mutex::new(Some(event)))),
        authz,
        verifier,
    )
    .execute(SubscribeRequest {
        channel_id: ChannelId::from_uuid(CHANNEL_UUID),
        bearer_token: token.to_owned(),
        from: Offset::BEGINNING,
    })
    .await
}

async fn assert_revocation_after_cache_bound(
    token: &str,
    tenant: TenantId,
    verifier: Arc<OryIdentityVerifier>,
    authz: Arc<KetoAuthzProvider>,
) {
    let revoke_url = format!(
        "{}/admin/relation-tuples?namespace=default&object={CHANNEL_UUID}&relation=subscribe&subject_id=user%3Aallowed",
        required("FRF_AUTHORITY_KETO_WRITE_URL")
    );
    reqwest::Client::new()
        .delete(revoke_url)
        .send()
        .await
        .expect("revoke channel grant")
        .error_for_status()
        .expect("Keto accepted revocation");
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    assert!(matches!(
        pipeline(token, event(tenant, ALLOWED_EVENT_UUID), verifier, authz,).await,
        Err(AppError::Forbidden(_))
    ));
}

#[tokio::test]
#[ignore = "requires scripts/run-authority-lifetime.sh owned Gate/Keto fixture"]
async fn real_gate_tokens_and_keto_enforce_subject_tenant_and_object() {
    let issuer = required("FRF_AUTHORITY_ISSUER");
    let audience = required("FRF_AUTHORITY_AUDIENCE");
    let tenant = TenantId::from_uuid(
        Uuid::parse_str(&required("FRF_AUTHORITY_TENANT")).expect("tenant UUID"),
    );
    let cross_tenant = TenantId::from_uuid(
        Uuid::parse_str(&required("FRF_AUTHORITY_CROSS_TENANT")).expect("cross-tenant UUID"),
    );
    let allowed_token = required("FRF_AUTHORITY_ALLOWED_TOKEN");
    let denied_token = required("FRF_AUTHORITY_DENIED_TOKEN");
    let cross_token = required("FRF_AUTHORITY_CROSS_TOKEN");
    let verifier = Arc::new(OryIdentityVerifier::with_issuer(
        required("FRF_AUTHORITY_JWKS_URL"),
        &audience,
        &issuer,
    ));
    let authz = Arc::new(KetoAuthzProvider::new(
        required("FRF_AUTHORITY_KETO_READ_URL"),
        "default",
    ));

    let claims = verifier.verify(&allowed_token).await.expect("Gate token");
    assert_eq!(claims.subject, "user:allowed");
    assert_eq!(claims.tenant_id, tenant);
    assert!(
        OryIdentityVerifier::with_issuer(
            required("FRF_AUTHORITY_JWKS_URL"),
            "wrong-audience",
            &issuer,
        )
        .verify(&allowed_token)
        .await
        .is_err()
    );
    assert!(
        OryIdentityVerifier::with_issuer(
            required("FRF_AUTHORITY_JWKS_URL"),
            &audience,
            "https://wrong-issuer.invalid",
        )
        .verify(&allowed_token)
        .await
        .is_err()
    );

    let mut allowed = pipeline(
        &allowed_token,
        event(tenant, ALLOWED_EVENT_UUID),
        Arc::clone(&verifier),
        Arc::clone(&authz),
    )
    .await
    .expect("authorized subscription");
    assert!(
        matches!(allowed.next().await, Some(Ok(envelope)) if envelope.id == EventId::from_uuid(ALLOWED_EVENT_UUID))
    );

    assert!(matches!(
        pipeline(
            &denied_token,
            event(tenant, ALLOWED_EVENT_UUID),
            Arc::clone(&verifier),
            Arc::clone(&authz),
        )
        .await,
        Err(AppError::Forbidden(_))
    ));

    let mut object_denied = pipeline(
        &allowed_token,
        event(tenant, DENIED_EVENT_UUID),
        Arc::clone(&verifier),
        Arc::clone(&authz),
    )
    .await
    .expect("channel grant remains valid");
    assert!(object_denied.next().await.is_none());

    let mut tenant_denied = pipeline(
        &cross_token,
        event(tenant, ALLOWED_EVENT_UUID),
        Arc::clone(&verifier),
        Arc::clone(&authz),
    )
    .await
    .expect("cross subject has a channel tuple to reach the tenant guard");
    assert_ne!(cross_tenant, tenant);
    assert!(tenant_denied.next().await.is_none());

    assert_revocation_after_cache_bound(&allowed_token, tenant, verifier, authz).await;
}
