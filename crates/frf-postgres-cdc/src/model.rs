use chrono::{DateTime, Utc};
use frf_domain::{ChangeOp, TenantId};
use serde::{Deserialize, Serialize};

/// Lossless canonical source value carried by the CDC spine payload.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CanonicalValue {
    Null,
    Bool(bool),
    SignedInteger(i64),
    UnsignedInteger(u64),
    Float(f64),
    Text(String),
    /// Unpadded base64url.
    Bytes(String),
    Decimal(String),
    Uuid(String),
    Timestamp {
        seconds: i64,
        nanos: i32,
    },
    Date(String),
    Time(String),
    TimestampWithoutZone(String),
    /// Unpadded base64url of RFC 8785 JSON bytes.
    Json(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyPart {
    pub column: String,
    pub value: CanonicalValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityKey {
    pub parts: Vec<KeyPart>,
    pub canonical_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityField {
    pub column: String,
    pub value: CanonicalValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePosition {
    pub epoch: String,
    pub commit_lsn: u64,
    pub transaction_index: u32,
}

/// One committed source mutation. Updates may omit fields listed in
/// `unchanged_toast`; projection consumers preserve their previous values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdcMutation {
    pub event_id: String,
    pub schema: String,
    pub table: String,
    pub projection: String,
    pub entity_type: String,
    pub tenant_id: TenantId,
    pub key: EntityKey,
    pub op: ChangeOp,
    pub record: Option<Vec<EntityField>>,
    pub unchanged_toast: Vec<String>,
    pub source: SourcePosition,
    pub committed_at: DateTime<Utc>,
}
