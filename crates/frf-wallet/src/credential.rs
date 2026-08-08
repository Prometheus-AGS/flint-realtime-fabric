use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The W3C Verifiable Credentials 2.0 base context.
///
/// VC 2.0 reached W3C Recommendation in May 2025 and is stable. Note that 2.0
/// replaced 1.1's `issuanceDate`/`expirationDate` with
/// [`validFrom`](Credential::valid_from) and
/// [`validUntil`](Credential::valid_until).
pub const VC_CONTEXT_V2: &str = "https://www.w3.org/ns/credentials/v2";

/// The credential type identifying an owner-to-node delegation.
pub const NODE_DELEGATION_TYPE: &str = "NodeDelegationCredential";

/// A capability an owner may delegate to one of their nodes.
///
/// Capabilities are explicit and enumerated rather than free-form strings so a
/// verifier cannot be tricked into accepting an unrecognised grant: an unknown
/// variant fails to deserialize instead of being silently ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Capability {
    /// May accept peer sessions on the owner's behalf.
    AcceptPeerSessions,
    /// May execute inference requests routed from a peer.
    ExecuteInference,
    /// May synchronise CRDT state with peers.
    SyncState,
    /// May advertise its capabilities to paired peers.
    AdvertiseCapabilities,
}

/// The claims a delegation credential asserts about its subject node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialSubject {
    /// The node's DID — the subject being granted capabilities.
    pub id: String,
    /// Capabilities the owner grants to this node.
    pub capabilities: Vec<Capability>,
}

/// An unsigned verifiable credential.
///
/// Serialized field names follow the VC 2.0 data model, so a document produced
/// here is recognisable to any conforming verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(rename = "type")]
    pub types: Vec<String>,
    /// DID of the owner issuing this credential.
    pub issuer: String,
    #[serde(rename = "validFrom")]
    pub valid_from: DateTime<Utc>,
    #[serde(rename = "validUntil", skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject,
}

impl Credential {
    /// Builds an owner-to-node delegation credential.
    ///
    /// `issuer_did` is the owner (a person or an organisation); `subject_did`
    /// is the node being granted `capabilities`.
    #[must_use]
    pub fn delegation(
        issuer_did: impl Into<String>,
        subject_did: impl Into<String>,
        capabilities: Vec<Capability>,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            context: vec![VC_CONTEXT_V2.to_owned()],
            types: vec![
                "VerifiableCredential".to_owned(),
                NODE_DELEGATION_TYPE.to_owned(),
            ],
            issuer: issuer_did.into(),
            valid_from,
            valid_until,
            credential_subject: CredentialSubject {
                id: subject_did.into(),
                capabilities,
            },
        }
    }

    /// Returns whether the credential is within its validity window at `now`.
    #[must_use]
    pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
        if now < self.valid_from {
            return false;
        }
        self.valid_until.is_none_or(|until| now <= until)
    }

    /// Returns whether this credential grants `capability`.
    #[must_use]
    pub fn grants(&self, capability: Capability) -> bool {
        self.credential_subject.capabilities.contains(&capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    fn at(offset_hours: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + offset_hours * 3600, 0)
            .unwrap_or(DateTime::UNIX_EPOCH)
    }

    fn sample() -> Credential {
        Credential::delegation(
            "did:key:owner",
            "did:key:node",
            vec![Capability::ExecuteInference],
            at(0),
            Some(at(24)),
        )
    }

    #[test]
    fn delegation_uses_the_vc_2_0_context_and_types() {
        let c = sample();
        assert_eq!(c.context.first().map(String::as_str), Some(VC_CONTEXT_V2));
        assert!(c.types.contains(&"VerifiableCredential".to_owned()));
        assert!(c.types.contains(&NODE_DELEGATION_TYPE.to_owned()));
    }

    #[test]
    fn serializes_with_vc_2_0_field_names() {
        // VC 2.0 renamed issuanceDate -> validFrom; emitting the 1.1 name would
        // make the document unrecognisable to a conforming verifier.
        let json = serde_json::to_value(sample()).expect("credential serializes");
        assert!(json.get("validFrom").is_some());
        assert!(json.get("validUntil").is_some());
        assert!(json.get("issuanceDate").is_none());
        assert!(json.get("credentialSubject").is_some());
        assert_eq!(
            json.get("@context")
                .and_then(|c| c.get(0))
                .and_then(serde_json::Value::as_str),
            Some(VC_CONTEXT_V2)
        );
    }

    #[test]
    fn validity_window_is_enforced_at_both_ends() {
        let c = sample();
        assert!(
            !c.is_valid_at(at(0) - TimeDelta::seconds(1)),
            "before start"
        );
        assert!(c.is_valid_at(at(0)), "at start");
        assert!(c.is_valid_at(at(12)), "midway");
        assert!(c.is_valid_at(at(24)), "at expiry");
        assert!(
            !c.is_valid_at(at(24) + TimeDelta::seconds(1)),
            "after expiry"
        );
    }

    #[test]
    fn a_credential_without_an_expiry_never_expires() {
        let c = Credential::delegation("did:key:o", "did:key:n", vec![], at(0), None);
        assert!(c.is_valid_at(at(100_000)));
    }

    #[test]
    fn grants_only_what_was_delegated() {
        let c = sample();
        assert!(c.grants(Capability::ExecuteInference));
        assert!(!c.grants(Capability::SyncState));
        assert!(!c.grants(Capability::AcceptPeerSessions));
    }

    #[test]
    fn unknown_capabilities_fail_to_deserialize() {
        // A verifier must not silently drop a grant it does not understand.
        let json = r#"{"id":"did:key:n","capabilities":["mint-money"]}"#;
        assert!(serde_json::from_str::<CredentialSubject>(json).is_err());
    }
}
