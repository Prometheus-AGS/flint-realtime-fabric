use chrono::Utc;
use frf_domain::{ChangeOp, EntityChange, EntityId, TenantId};
use serde_json::{Map, Value};

/// Lightweight representation of a pgoutput relation (table) descriptor.
///
/// In the actual WAL stream this arrives as a `RelationMessage` before any
/// row-level messages for that relation.
#[derive(Debug, Clone)]
pub struct Relation {
    pub oid: u32,
    pub namespace: String,
    pub name: String,
    /// Ordered list of column names in the relation.
    pub columns: Vec<String>,
    /// Index of the primary-key column within `columns` (0-based).
    pub pk_index: usize,
}

/// A single column value decoded from a pgoutput tuple data.
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name (from the associated `Relation`).
    pub name: String,
    /// Text-decoded value. `None` means SQL NULL.
    pub value: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("null primary key in relation '{0}'")]
    NullPrimaryKey(String),
    #[error("invalid entity id '{value}' in relation '{relation}': {source}")]
    InvalidEntityId {
        relation: String,
        value: String,
        source: uuid::Error,
    },
    #[error("pk_index {0} out of bounds for relation '{1}'")]
    PkIndexOutOfBounds(usize, String),
}

fn columns_to_json(cols: &[Column]) -> Value {
    let mut map = Map::new();
    for col in cols {
        map.insert(
            col.name.clone(),
            col.value
                .as_deref()
                .map_or(Value::Null, |v| Value::String(v.to_owned())),
        );
    }
    Value::Object(map)
}

fn extract_entity_id(relation: &Relation, row: &[Column]) -> Result<EntityId, DecodeError> {
    let col = row
        .get(relation.pk_index)
        .ok_or_else(|| DecodeError::PkIndexOutOfBounds(relation.pk_index, relation.name.clone()))?;
    let raw = col
        .value
        .as_deref()
        .ok_or_else(|| DecodeError::NullPrimaryKey(relation.name.clone()))?;
    let uuid = uuid::Uuid::parse_str(raw).map_err(|source| DecodeError::InvalidEntityId {
        relation: relation.name.clone(),
        value: raw.to_owned(),
        source,
    })?;
    Ok(EntityId::from_uuid(uuid))
}

/// Decode a pgoutput `INSERT` tuple into an `EntityChange`.
///
/// # Errors
///
/// Returns [`DecodeError`] if the primary key column is absent, null, or not a valid UUID.
pub fn decode_insert(
    relation: &Relation,
    tenant_id: TenantId,
    row: &[Column],
) -> Result<EntityChange, DecodeError> {
    let entity_id = extract_entity_id(relation, row)?;
    Ok(EntityChange {
        entity_id,
        tenant_id,
        entity_type: relation.name.clone(),
        op: ChangeOp::Insert,
        data: columns_to_json(row),
        previous: None,
        session_id: None,
        timestamp: Utc::now(),
        version: 0,
    })
}

/// Decode a pgoutput `UPDATE` tuple into an `EntityChange`.
///
/// # Errors
///
/// Returns [`DecodeError`] if the primary key column is absent, null, or not a valid UUID.
pub fn decode_update(
    relation: &Relation,
    tenant_id: TenantId,
    old_row: Option<&[Column]>,
    new_row: &[Column],
) -> Result<EntityChange, DecodeError> {
    let entity_id = extract_entity_id(relation, new_row)?;
    Ok(EntityChange {
        entity_id,
        tenant_id,
        entity_type: relation.name.clone(),
        op: ChangeOp::Update,
        data: columns_to_json(new_row),
        previous: old_row.map(columns_to_json),
        session_id: None,
        timestamp: Utc::now(),
        version: 0,
    })
}

/// Decode a pgoutput `DELETE` tuple into an `EntityChange`.
///
/// # Errors
///
/// Returns [`DecodeError`] if the primary key column is absent, null, or not a valid UUID.
pub fn decode_delete(
    relation: &Relation,
    tenant_id: TenantId,
    old_row: &[Column],
) -> Result<EntityChange, DecodeError> {
    let entity_id = extract_entity_id(relation, old_row)?;
    Ok(EntityChange {
        entity_id,
        tenant_id,
        entity_type: relation.name.clone(),
        op: ChangeOp::Delete,
        data: Value::Null,
        previous: Some(columns_to_json(old_row)),
        session_id: None,
        timestamp: Utc::now(),
        version: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_tenant() -> TenantId {
        TenantId::from_uuid(Uuid::nil())
    }

    fn test_relation() -> Relation {
        Relation {
            oid: 1234,
            namespace: "public".to_owned(),
            name: "users".to_owned(),
            columns: vec!["id".to_owned(), "email".to_owned(), "name".to_owned()],
            pk_index: 0,
        }
    }

    fn id_col(uuid: &str) -> Column {
        Column {
            name: "id".to_owned(),
            value: Some(uuid.to_owned()),
        }
    }

    fn email_col(email: &str) -> Column {
        Column {
            name: "email".to_owned(),
            value: Some(email.to_owned()),
        }
    }

    fn name_col(name: &str) -> Column {
        Column {
            name: "name".to_owned(),
            value: Some(name.to_owned()),
        }
    }

    #[test]
    fn decode_insert_produces_insert_op() {
        let rel = test_relation();
        let id = Uuid::new_v4().to_string();
        let row = vec![id_col(&id), email_col("a@b.com"), name_col("Alice")];

        let change = decode_insert(&rel, test_tenant(), &row).expect("decode_insert failed");

        assert_eq!(change.op, ChangeOp::Insert);
        assert_eq!(change.entity_type, "users");
        assert!(change.previous.is_none());
        assert_eq!(change.data["email"], "a@b.com");
    }

    #[test]
    fn decode_update_with_old_row() {
        let rel = test_relation();
        let id = Uuid::new_v4().to_string();
        let old = vec![id_col(&id), email_col("old@b.com"), name_col("Old")];
        let new = vec![id_col(&id), email_col("new@b.com"), name_col("New")];

        let change =
            decode_update(&rel, test_tenant(), Some(&old), &new).expect("decode_update failed");

        assert_eq!(change.op, ChangeOp::Update);
        assert!(change.previous.is_some());
        assert_eq!(change.data["email"], "new@b.com");
        assert_eq!(change.previous.as_ref().unwrap()["email"], "old@b.com");
    }

    #[test]
    fn decode_update_without_old_row() {
        let rel = test_relation();
        let id = Uuid::new_v4().to_string();
        let new = vec![id_col(&id), email_col("new@b.com"), name_col("New")];

        let change = decode_update(&rel, test_tenant(), None, &new).expect("decode_update failed");

        assert_eq!(change.op, ChangeOp::Update);
        assert!(change.previous.is_none());
    }

    #[test]
    fn decode_delete_produces_delete_op() {
        let rel = test_relation();
        let id = Uuid::new_v4().to_string();
        let old = vec![id_col(&id), email_col("gone@b.com"), name_col("Gone")];

        let change = decode_delete(&rel, test_tenant(), &old).expect("decode_delete failed");

        assert_eq!(change.op, ChangeOp::Delete);
        assert_eq!(change.data, Value::Null);
        assert_eq!(change.previous.as_ref().unwrap()["email"], "gone@b.com");
    }

    #[test]
    fn null_pk_returns_error() {
        let rel = test_relation();
        let row = vec![
            Column {
                name: "id".to_owned(),
                value: None,
            },
            email_col("x@y.com"),
            name_col("X"),
        ];

        let result = decode_insert(&rel, test_tenant(), &row);
        assert!(
            matches!(result, Err(DecodeError::NullPrimaryKey(_))),
            "expected NullPrimaryKey"
        );
    }

    #[test]
    fn invalid_uuid_pk_returns_error() {
        let rel = test_relation();
        let row = vec![
            Column {
                name: "id".to_owned(),
                value: Some("not-a-uuid".to_owned()),
            },
            email_col("x@y.com"),
            name_col("X"),
        ];

        let result = decode_insert(&rel, test_tenant(), &row);
        assert!(
            matches!(result, Err(DecodeError::InvalidEntityId { .. })),
            "expected InvalidEntityId"
        );
    }
}
