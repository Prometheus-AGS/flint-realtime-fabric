use frf_domain::{EntityKey, EntityTypeDelivery, EntityTypeSelector, SourcePosition};
use frf_ports::PortError;
use sha2::{Digest as _, Sha256};

use crate::AppError;

pub(super) fn disclosed_barrier(
    source_epoch: &str,
    deliveries: &[EntityTypeDelivery],
) -> SourcePosition {
    deliveries
        .iter()
        .map(|delivery| &delivery.mutation.source)
        .max_by_key(|source| (source.commit_lsn, source.transaction_index))
        .cloned()
        .unwrap_or_else(|| SourcePosition {
            epoch: source_epoch.to_owned(),
            commit_lsn: 0,
            transaction_index: 0,
        })
}

pub(super) fn snapshot_id(
    epoch: &str,
    entity_type: &EntityTypeSelector,
    key: &EntityKey,
    barrier_lsn: u64,
) -> Result<String, AppError> {
    let identity = serde_json::json!({
        "barrier_lsn": barrier_lsn.to_string(),
        "entity_type": entity_type.canonical_name(),
        "epoch": epoch,
        "key": key.canonical_id,
    });
    let bytes = serde_json_canonicalizer::to_vec(&identity)
        .map_err(|error| PortError::Serialization(error.to_string()))?;
    Ok(format!("frfsnapshot:v1:{:x}", Sha256::digest(bytes)))
}
