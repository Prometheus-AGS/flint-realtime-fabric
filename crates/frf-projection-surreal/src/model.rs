use chrono::{DateTime, Utc};
use frf_domain::{ChangeOp, EntityChange, EntityId, TenantId};
use frf_ports::ProjectionCursor;
use surrealdb::types::SurrealValue;
use uuid::Uuid;

use crate::error::SurrealProjectionError;

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub(crate) struct EntityRow {
    pub entity_id: String,
    pub tenant_id: String,
    pub entity_type: String,
    pub op: String,
    pub data_json: String,
    pub previous_json: Option<String>,
    pub timestamp: String,
    pub version: String,
}

impl EntityRow {
    pub(crate) fn from_change(change: &EntityChange) -> Result<Self, SurrealProjectionError> {
        Ok(Self {
            entity_id: change.entity_id.to_string(),
            tenant_id: change.tenant_id.to_string(),
            entity_type: change.entity_type.clone(),
            op: op_name(&change.op).to_owned(),
            data_json: serde_json::to_string(&change.data)
                .map_err(|error| SurrealProjectionError::InvalidData(error.to_string()))?,
            previous_json: change
                .previous
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(|error| SurrealProjectionError::InvalidData(error.to_string()))?,
            timestamp: change.timestamp.to_rfc3339(),
            version: change.version.to_string(),
        })
    }

    pub(crate) fn into_change(self) -> Result<EntityChange, SurrealProjectionError> {
        Ok(EntityChange {
            entity_id: EntityId::from_uuid(parse_uuid("entity_id", &self.entity_id)?),
            tenant_id: TenantId::from_uuid(parse_uuid("tenant_id", &self.tenant_id)?),
            entity_type: self.entity_type,
            op: parse_op(&self.op)?,
            data: serde_json::from_str(&self.data_json)
                .map_err(|error| SurrealProjectionError::InvalidData(error.to_string()))?,
            previous: self
                .previous_json
                .map(|value| serde_json::from_str(&value))
                .transpose()
                .map_err(|error| SurrealProjectionError::InvalidData(error.to_string()))?,
            session_id: None,
            timestamp: DateTime::<Utc>::from(
                DateTime::parse_from_rfc3339(&self.timestamp)
                    .map_err(|error| SurrealProjectionError::InvalidData(error.to_string()))?,
            ),
            version: self.version.parse().map_err(|error| {
                SurrealProjectionError::InvalidData(format!("version: {error}"))
            })?,
        })
    }
}

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub(crate) struct CursorRow {
    pub source_epoch: String,
    pub commit_lsn: String,
    pub transaction_index: String,
    pub broker_offset: String,
}

impl CursorRow {
    pub(crate) fn from_cursor(cursor: &ProjectionCursor) -> Self {
        Self {
            source_epoch: cursor.source_epoch.clone(),
            commit_lsn: cursor.commit_lsn.to_string(),
            transaction_index: cursor.transaction_index.to_string(),
            broker_offset: cursor.broker_offset.to_string(),
        }
    }

    pub(crate) fn into_cursor(self) -> Result<ProjectionCursor, SurrealProjectionError> {
        Ok(ProjectionCursor {
            source_epoch: self.source_epoch,
            commit_lsn: parse_number("commit_lsn", &self.commit_lsn)?,
            transaction_index: parse_number("transaction_index", &self.transaction_index)?,
            broker_offset: parse_number("broker_offset", &self.broker_offset)?,
        })
    }
}

fn parse_uuid(field: &str, value: &str) -> Result<Uuid, SurrealProjectionError> {
    Uuid::parse_str(value)
        .map_err(|error| SurrealProjectionError::InvalidData(format!("{field}: {error}")))
}

fn parse_number<T>(field: &str, value: &str) -> Result<T, SurrealProjectionError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .parse()
        .map_err(|error| SurrealProjectionError::InvalidData(format!("{field}: {error}")))
}

fn op_name(op: &ChangeOp) -> &'static str {
    match op {
        ChangeOp::Insert => "insert",
        ChangeOp::Update => "update",
        ChangeOp::Delete => "delete",
        ChangeOp::Upsert => "upsert",
        _ => "unknown",
    }
}

fn parse_op(value: &str) -> Result<ChangeOp, SurrealProjectionError> {
    match value {
        "insert" => Ok(ChangeOp::Insert),
        "update" => Ok(ChangeOp::Update),
        "delete" => Ok(ChangeOp::Delete),
        "upsert" => Ok(ChangeOp::Upsert),
        other => Err(SurrealProjectionError::InvalidData(format!(
            "unsupported stored operation {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{op_name, parse_op};
    use frf_domain::ChangeOp;

    #[test]
    fn persisted_change_operations_round_trip() {
        for op in [
            ChangeOp::Insert,
            ChangeOp::Update,
            ChangeOp::Delete,
            ChangeOp::Upsert,
        ] {
            assert_eq!(parse_op(op_name(&op)).expect("parse stored op"), op);
        }
    }
}
