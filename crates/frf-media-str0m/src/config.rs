//! Media transport network configuration (p24-c001).
//!
//! GAP-1 from phase-24: the sovereign SFU historically bound `127.0.0.1:0` (loopback,
//! ephemeral) and advertised a `127.0.0.1` host candidate — unreachable by a browser in
//! another process/container. `MediaConfig` makes the bind address, advertised candidate IP,
//! and UDP port configurable so a real browser↔gateway ICE path can complete. The default is
//! the historical loopback-ephemeral behaviour, so existing tests are unaffected.

use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

/// How the sovereign media transport binds its UDP socket and advertises its host candidate.
#[derive(Debug, Clone)]
pub struct MediaConfig {
    /// Address the per-session UDP socket binds to (e.g. `0.0.0.0` for a container).
    pub bind_addr: IpAddr,
    /// Host **or IP** advertised in the SDP host candidate. When `None`, the bound local address is
    /// used (correct for loopback/dev). A container sets this to a host-reachable value — an IP is
    /// used as-is; a hostname (e.g. `host.docker.internal`) is DNS-resolved to an `IpAddr` at
    /// negotiate time (p28-c002). str0m rejects a hostname or `0.0.0.0` as an ICE candidate, so the
    /// value must resolve to a concrete routable IP.
    pub advertise_host: Option<String>,
    /// UDP port to bind. `0` = ephemeral (loopback/dev); a fixed port is required when the
    /// socket must be reachable through a mapped container port.
    pub udp_port: u16,
}

impl MediaConfig {
    /// The historical default: bind loopback on an ephemeral port, advertise the bound address.
    /// Keeps every existing test and the in-process two-peer proof working unchanged.
    #[must_use]
    pub fn loopback() -> Self {
        Self {
            bind_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            advertise_host: None,
            udp_port: 0,
        }
    }

    /// Build from environment (used by the gateway for `SFU_MODE=sovereign`):
    /// `MEDIA_BIND_ADDR` (default `0.0.0.0`), `MEDIA_ADVERTISE_IP` (optional host or IP),
    /// `MEDIA_UDP_PORT` (default `0`). An unparseable bind/port falls back to the default.
    #[must_use]
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("MEDIA_BIND_ADDR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        // Keep the raw string — it may be a hostname (`host.docker.internal`); resolution happens
        // at negotiate time via `resolve_advertised_ip`.
        let advertise_host = std::env::var("MEDIA_ADVERTISE_IP")
            .ok()
            .filter(|v| !v.is_empty());
        let udp_port = std::env::var("MEDIA_UDP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        Self {
            bind_addr,
            advertise_host,
            udp_port,
        }
    }

    /// Resolve the advertised host to a concrete `IpAddr` for the ICE host candidate: a literal IP
    /// is returned as-is; a hostname is DNS-resolved (first non-loopback-preferring result). Returns
    /// `None` when no `advertise_host` is set (caller uses the bound address).
    ///
    /// # Errors
    /// [`std::io::Error`] if a hostname is set but cannot be resolved to any address.
    pub fn resolve_advertised_ip(&self) -> std::io::Result<Option<IpAddr>> {
        let Some(host) = &self.advertise_host else {
            return Ok(None);
        };
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(Some(ip));
        }
        // Hostname → resolve via the OS resolver (port is irrelevant for the lookup).
        let addr = (host.as_str(), 0u16)
            .to_socket_addrs()?
            .map(|s| s.ip())
            .next()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("MEDIA_ADVERTISE_IP host {host} resolved to no address"),
                )
            })?;
        Ok(Some(addr))
    }
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self::loopback()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_default_binds_localhost_ephemeral_and_advertises_bound_addr() {
        let cfg = MediaConfig::loopback();
        assert_eq!(cfg.bind_addr, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(cfg.udp_port, 0, "ephemeral");
        assert!(cfg.advertise_host.is_none(), "advertise the bound address");
    }

    #[test]
    fn default_matches_loopback() {
        let d = MediaConfig::default();
        let l = MediaConfig::loopback();
        assert_eq!(d.bind_addr, l.bind_addr);
        assert_eq!(d.udp_port, l.udp_port);
        assert_eq!(d.advertise_host, l.advertise_host);
    }

    #[test]
    fn resolve_advertised_ip_passes_a_literal_ip_through() {
        let cfg = MediaConfig {
            bind_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            advertise_host: Some("203.0.113.7".to_owned()),
            udp_port: 40000,
        };
        assert_eq!(
            cfg.resolve_advertised_ip().expect("resolve"),
            Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)))
        );
    }

    #[test]
    fn resolve_advertised_ip_is_none_without_a_host() {
        assert_eq!(
            MediaConfig::loopback()
                .resolve_advertised_ip()
                .expect("resolve"),
            None
        );
    }
}
