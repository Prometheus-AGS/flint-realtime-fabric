use std::sync::Arc;

use frf_domain::{ChangeOp, TenantId};
use pg_walstream::RowData;

use crate::{
    canonical::{CanonicalError, canonical_key, canonical_value},
    catalog::RelationMapping,
    config::TenantMode,
    model::{EntityField, EntityKey, KeyPart},
};

#[derive(Debug, Clone)]
pub(crate) struct PendingMutation {
    pub schema: String,
    pub table: String,
    pub projection: String,
    pub entity_type: String,
    pub tenant_id: TenantId,
    pub key: EntityKey,
    pub op: ChangeOp,
    pub record: Option<Vec<EntityField>>,
    pub unchanged_toast: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("required source column '{column}' is absent for '{relation}'")]
    MissingColumn { relation: String, column: String },
    #[error("tenant column '{column}' is not a UUID for '{relation}'")]
    InvalidTenant { relation: String, column: String },
    #[error("update old-key tuple is incomplete for '{0}'")]
    MissingOldKey(String),
}

pub(crate) fn decode_insert(
    relation: &RelationMapping,
    fixed_tenant: TenantId,
    row: &RowData,
) -> Result<Vec<PendingMutation>, DecodeError> {
    let key = extract_key(relation, row)?;
    let tenant_id = extract_tenant(relation, fixed_tenant, row)?;
    let (record, unchanged_toast) = extract_record(relation, row, false)?;
    Ok(vec![pending(
        relation,
        tenant_id,
        key,
        ChangeOp::Insert,
        Some(record),
        unchanged_toast,
    )])
}

pub(crate) fn decode_update(
    relation: &RelationMapping,
    fixed_tenant: TenantId,
    old_row: Option<&RowData>,
    new_row: &RowData,
    replica_identity: pg_walstream::ReplicaIdentity,
    key_columns: &[Arc<str>],
) -> Result<Vec<PendingMutation>, DecodeError> {
    let new_key = extract_key(relation, new_row)?;
    let new_tenant = extract_tenant(relation, fixed_tenant, new_row)?;
    let (record, unchanged_toast) = extract_record(relation, new_row, true)?;
    let Some(old_row) = old_row else {
        if !relation.update_without_old_key_is_safe(replica_identity, key_columns) {
            return Err(DecodeError::MissingOldKey(relation.entity_type()));
        }
        return Ok(vec![pending(
            relation,
            new_tenant,
            new_key,
            ChangeOp::Update,
            Some(record),
            unchanged_toast,
        )]);
    };

    let old_key = extract_key(relation, old_row)
        .map_err(|_| DecodeError::MissingOldKey(relation.entity_type()))?;
    let old_tenant = extract_tenant(relation, fixed_tenant, old_row)
        .map_err(|_| DecodeError::MissingOldKey(relation.entity_type()))?;
    if old_key.canonical_id == new_key.canonical_id && old_tenant == new_tenant {
        return Ok(vec![pending(
            relation,
            new_tenant,
            new_key,
            ChangeOp::Update,
            Some(record),
            unchanged_toast,
        )]);
    }

    Ok(vec![
        pending(
            relation,
            old_tenant,
            old_key,
            ChangeOp::Delete,
            None,
            Vec::new(),
        ),
        pending(
            relation,
            new_tenant,
            new_key,
            ChangeOp::Insert,
            Some(record),
            unchanged_toast,
        ),
    ])
}

pub(crate) fn decode_delete(
    relation: &RelationMapping,
    fixed_tenant: TenantId,
    old_row: &RowData,
) -> Result<Vec<PendingMutation>, DecodeError> {
    let key = extract_key(relation, old_row)?;
    let tenant_id = extract_tenant(relation, fixed_tenant, old_row)?;
    Ok(vec![pending(
        relation,
        tenant_id,
        key,
        ChangeOp::Delete,
        None,
        Vec::new(),
    )])
}

fn pending(
    relation: &RelationMapping,
    tenant_id: TenantId,
    key: EntityKey,
    op: ChangeOp,
    record: Option<Vec<EntityField>>,
    unchanged_toast: Vec<String>,
) -> PendingMutation {
    PendingMutation {
        schema: relation.schema.clone(),
        table: relation.table.clone(),
        projection: relation.projection.clone(),
        entity_type: relation.entity_type(),
        tenant_id,
        key,
        op,
        record,
        unchanged_toast,
    }
}

fn extract_key(relation: &RelationMapping, row: &RowData) -> Result<EntityKey, DecodeError> {
    let mut parts = Vec::with_capacity(relation.primary_key.len());
    for index in &relation.primary_key {
        let column = &relation.columns[*index];
        let value = row
            .get(&column.name)
            .ok_or_else(|| DecodeError::MissingColumn {
                relation: relation.entity_type(),
                column: column.name.clone(),
            })?;
        parts.push(KeyPart {
            column: column.name.clone(),
            value: canonical_value(&column.name, column.source_type, value)?,
        });
    }
    Ok(canonical_key(parts)?)
}

fn extract_tenant(
    relation: &RelationMapping,
    fixed_tenant: TenantId,
    row: &RowData,
) -> Result<TenantId, DecodeError> {
    let TenantMode::Column { column } = &relation.tenant else {
        return Ok(fixed_tenant);
    };
    let mapping = relation
        .column(column)
        .ok_or_else(|| DecodeError::MissingColumn {
            relation: relation.entity_type(),
            column: column.clone(),
        })?;
    let source = row.get(column).ok_or_else(|| DecodeError::MissingColumn {
        relation: relation.entity_type(),
        column: column.clone(),
    })?;
    let value = canonical_value(column, mapping.source_type, source)?;
    let crate::model::CanonicalValue::Uuid(value) = value else {
        return Err(DecodeError::InvalidTenant {
            relation: relation.entity_type(),
            column: column.clone(),
        });
    };
    uuid::Uuid::parse_str(&value)
        .map(TenantId::from_uuid)
        .map_err(|_| DecodeError::InvalidTenant {
            relation: relation.entity_type(),
            column: column.clone(),
        })
}

fn extract_record(
    relation: &RelationMapping,
    row: &RowData,
    allow_unchanged: bool,
) -> Result<(Vec<EntityField>, Vec<String>), DecodeError> {
    let mut record = Vec::new();
    let mut unchanged_toast = Vec::new();
    for column in relation.columns.iter().filter(|column| column.projected) {
        let Some(value) = row.get(&column.name) else {
            if allow_unchanged {
                unchanged_toast.push(column.name.clone());
                continue;
            }
            return Err(DecodeError::MissingColumn {
                relation: relation.entity_type(),
                column: column.name.clone(),
            });
        };
        record.push(EntityField {
            column: column.name.clone(),
            value: canonical_value(&column.name, column.source_type, value)?,
        });
    }
    Ok((record, unchanged_toast))
}
