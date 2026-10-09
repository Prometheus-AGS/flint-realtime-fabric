//! Both peers present their own credential, bound to the QUIC-proven key.
use crate::protocol::{MAX_CREDENTIAL_BYTES, read_frame, write_frame};
use crate::transport::PeerTransport;
use crate::{P2pError, PeerIdentity};
use iroh::endpoint::Connection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Credential {
    version: u8,
    token: String,
}

impl PeerTransport {
    pub(crate) async fn admit(
        &self,
        connection: &Connection,
        token: &str,
        incoming: bool,
    ) -> Result<PeerIdentity, P2pError> {
        let remote = connection
            .remote_id()
            .map_err(|_| P2pError::Unauthenticated("QUIC peer identity unavailable".into()))?
            .to_string();
        if !self.is_paired(&remote)? {
            return Err(P2pError::NotPaired(remote));
        }
        let local = self
            .verifier
            .verify(token, &self.endpoint.id().to_string())
            .await?;
        if local.endpoint_id != self.endpoint.id().to_string() {
            return Err(P2pError::Unauthenticated(
                "local credential is not endpoint-bound".into(),
            ));
        }
        let (mut send, mut recv) = if incoming {
            connection.accept_bi().await
        } else {
            connection.open_bi().await
        }
        .map_err(|e| P2pError::Stream(e.to_string()))?;
        let own = serde_json::to_vec(&Credential {
            version: 1,
            token: token.into(),
        })
        .map_err(|e| P2pError::Codec(e.to_string()))?;
        if !incoming {
            write_frame(&mut send, &own, MAX_CREDENTIAL_BYTES).await?;
        }
        let presented: Credential =
            serde_json::from_slice(&read_frame(&mut recv, MAX_CREDENTIAL_BYTES).await?)
                .map_err(|_| P2pError::Unauthenticated("invalid peer credential frame".into()))?;
        if presented.version != 1 {
            return Err(P2pError::Unauthenticated(
                "unsupported admission version".into(),
            ));
        }
        let identity = self.verifier.verify(&presented.token, &remote).await?;
        if identity.endpoint_id != remote
            || identity.subject.is_empty()
            || identity.tenant_id != local.tenant_id
        {
            return Err(P2pError::Unauthenticated(
                "peer credential identity mismatch".into(),
            ));
        }
        if incoming {
            write_frame(&mut send, &own, MAX_CREDENTIAL_BYTES).await?;
        }
        // A peer cannot obtain an admitted session until both credentials were accepted.
        const ACK: &[u8] = b"frf/p2p/1/admitted";
        if !incoming {
            write_frame(&mut send, ACK, MAX_CREDENTIAL_BYTES).await?;
        }
        if read_frame(&mut recv, MAX_CREDENTIAL_BYTES).await? != ACK {
            return Err(P2pError::Unauthenticated(
                "mutual admission acknowledgement missing".into(),
            ));
        }
        if incoming {
            write_frame(&mut send, ACK, MAX_CREDENTIAL_BYTES).await?;
        }
        send.finish().map_err(|e| P2pError::Stream(e.to_string()))?;
        Ok(identity)
    }
}
