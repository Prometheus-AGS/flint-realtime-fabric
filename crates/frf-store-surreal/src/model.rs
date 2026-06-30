use surrealdb::types::SurrealValue;

/// Wire format for a CRDT snapshot row in `SurrealDB`.
///
/// `encoded_hex` stores the Loro snapshot as hex so the `SurrealDB` client
/// round-trips it cleanly as a `String` field. The hex is internal — callers
/// of `CrdtStore` always receive `Bytes`.
#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub struct SnapshotRow {
    pub entity_id: String,
    pub tenant_id: String,
    /// Hex-encoded Loro snapshot bytes.
    pub encoded_hex: String,
    pub version: u64,
}
