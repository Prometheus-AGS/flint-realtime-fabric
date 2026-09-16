use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ChangeOp, TenantId};

/// Lossless canonical source value carried by committed entity mutations.
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

/// One committed, canonical entity mutation before broker-position attachment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommittedEntityMutation {
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

/// Server-approved schema-qualified entity type and projection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityTypeSelector {
    pub schema: String,
    pub name: String,
    pub projection: String,
}

impl EntityTypeSelector {
    #[must_use]
    pub fn canonical_name(&self) -> String {
        format!("{}.{}@{}", self.schema, self.name, self.projection)
    }

    #[must_use]
    pub fn matches(&self, mutation: &CommittedEntityMutation) -> bool {
        self.schema == mutation.schema
            && self.name == mutation.table
            && self.projection == mutation.projection
            && self.canonical_name() == mutation.entity_type
    }
}

/// A committed mutation with its authoritative broker position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTypeDelivery {
    pub mutation: CommittedEntityMutation,
    pub broker_partition: u32,
    pub broker_offset: u64,
}
