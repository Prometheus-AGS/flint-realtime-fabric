use serde::{Deserialize, Serialize};

/// Application-Level Protocol Negotiation identifier for fabric peer sessions.
///
/// iroh requires an ALPN on both sides; a peer offering a different ALPN is
/// rejected by the QUIC handshake before any fabric code runs.
pub const FRF_P2P_ALPN: &[u8] = b"frf/p2p/1";

/// Runtime configuration for peer-to-peer operation.
///
/// **This subsystem is compiled into every build and is on by default.** It is
/// disabled through configuration, never through a Cargo feature: a build-time
/// gate would mean the code is absent from default binaries, never exercised by
/// default tests, and adopters would have to opt in at build time. Decentralized
/// operation is default scope, so it ships in the default build.
///
/// Default-on does **not** mean auth-optional. With no compile-time gate
/// standing between a default build and peer reachability, the fail-closed
/// posture is what carries the security guarantee — see
/// [`crate::identity::DenyAllVerifier`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PeerConfig {
    /// Whether peer-to-peer operation is enabled. **Defaults to `true`.**
    pub enabled: bool,

    /// Whether to discover peers on the local network via mDNS.
    ///
    /// Discovery is LAN + explicit pairing by design, not a DHT. Discovering a
    /// peer never implies trusting it: pairing and token verification are
    /// separate, and both are required.
    pub mdns_discovery: bool,

    /// Endpoint ids this node is paired with, as configuration.
    ///
    /// Empty by default — a node pairs with nobody until told to.
    pub paired_endpoints: Vec<String>,
}

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mdns_discovery: true,
            paired_endpoints: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_operation_is_on_by_default() {
        let config = PeerConfig::default();
        assert!(
            config.enabled,
            "P2P is default scope and must default to enabled"
        );
    }

    #[test]
    fn default_config_pairs_with_nobody() {
        assert!(
            PeerConfig::default().paired_endpoints.is_empty(),
            "on-by-default must not mean paired-with-anyone"
        );
    }

    #[test]
    fn serde_default_yields_enabled() {
        // An empty config block must produce an enabled subsystem, matching the
        // `#[serde(default)]` behaviour a host will actually hit.
        let config: PeerConfig =
            serde_json::from_str("{}").expect("empty object deserializes via serde(default)");
        assert_eq!(config, PeerConfig::default());
        assert!(config.enabled);
    }

    #[test]
    fn explicit_disable_is_honoured() {
        let config: PeerConfig =
            serde_json::from_str(r#"{"enabled": false}"#).expect("explicit disable deserializes");
        assert!(!config.enabled);
        // Disabling the subsystem must not silently flip unrelated fields.
        assert!(config.mdns_discovery);
    }
}
