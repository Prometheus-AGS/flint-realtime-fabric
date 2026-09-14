#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfuMode {
    /// Sovereign SFU using `str0m` (no cloud dependency).
    Sovereign,
    /// Hosted SFU using `LiveKit`.
    Hosted,
}

impl From<SfuMode> for frf_domain::SfuMode {
    fn from(mode: SfuMode) -> Self {
        match mode {
            SfuMode::Sovereign => frf_domain::SfuMode::Sovereign,
            SfuMode::Hosted => frf_domain::SfuMode::Hosted,
        }
    }
}

/// Selects the `ActionPolicyProvider` implementation at startup.
///
/// Controlled by the `POLICY_ENGINE` env var.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEngineMode {
    /// No-op: every action is permitted (default, zero overhead).
    None,
    /// Cedar: in-memory `PolicySet` loaded from the bundled `policy.cedar`.
    Cedar,
}

/// Selects the authorization adapter composed behind verified JWT identity and
/// tenant-equality guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthzBackend {
    /// Sansaba mode: authentication is the non-admin authorization boundary and
    /// durable data authorization remains in `PostgreSQL` RLS.
    VerifiedIdentity,
    /// Generic platform relationship authorization through Ory Keto.
    Keto,
}

/// Which public gateway surface is mounted by this process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayProfile {
    /// All event, agent, signaling, gRPC, and optional shape surfaces.
    Full,
    /// Only health/readiness and the authorized Electric shape facade.
    ShapeOnly,
}
