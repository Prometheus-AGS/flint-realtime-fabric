pub use frf_domain::{CanonicalValue, EntityField, EntityKey, KeyPart, SourcePosition};

/// Backward-compatible adapter name for the shared committed mutation model.
pub type CdcMutation = frf_domain::CommittedEntityMutation;
