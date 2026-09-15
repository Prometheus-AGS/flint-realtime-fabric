use std::collections::HashSet;

use pg_walstream::ReplicaIdentity;

use crate::{
    canonical::SourceType,
    catalog::{CatalogError, ColumnMapping, RelationMapping},
    config::{CdcConfig, TableEnrollment, TenantMode},
};

pub(super) fn validate_config(config: &CdcConfig) -> Result<(), CatalogError> {
    let epoch = config.source_epoch.as_deref().unwrap_or_default();
    CdcConfig::validate_source_epoch(epoch).map_err(|_| CatalogError::InvalidEpoch)?;
    if config.enrollments.is_empty() {
        return Err(CatalogError::EmptyEnrollment);
    }
    if config.lsn_checkpoint_interval == 0 {
        return Err(CatalogError::InvalidCheckpointInterval);
    }
    validate_identifier(&config.slot_name)?;
    validate_identifier(&config.publication_name)?;
    let mut seen = HashSet::new();
    for enrollment in &config.enrollments {
        for identifier in [
            enrollment.schema.as_str(),
            enrollment.table.as_str(),
            enrollment.projection.as_str(),
        ] {
            validate_identifier(identifier)?;
        }
        let qualified = format!("{}.{}", enrollment.schema, enrollment.table);
        if !seen.insert(qualified.clone()) {
            return Err(CatalogError::DuplicateEnrollment(qualified));
        }
    }
    Ok(())
}

fn validate_identifier(identifier: &str) -> Result<(), CatalogError> {
    let mut chars = identifier.chars();
    let first = chars
        .next()
        .ok_or_else(|| CatalogError::InvalidIdentifier(identifier.to_owned()))?;
    if identifier.len() > 63
        || !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(CatalogError::InvalidIdentifier(identifier.to_owned()));
    }
    Ok(())
}

pub(super) fn validate_columns(
    relation: &str,
    enrollment: &TableEnrollment,
    columns: &[ColumnMapping],
    primary_key: &[String],
    identity: &HashSet<&str>,
) -> Result<(), CatalogError> {
    for projected in &enrollment.columns {
        if !columns.iter().any(|column| column.name == *projected) {
            return Err(CatalogError::InvalidProjection {
                relation: relation.to_owned(),
                column: projected.clone(),
            });
        }
    }
    let tenant_column = match &enrollment.tenant {
        TenantMode::Column { column } => {
            let Some(mapping) = columns.iter().find(|mapping| mapping.name == *column) else {
                return Err(CatalogError::InvalidTenant {
                    relation: relation.to_owned(),
                    column: column.clone(),
                });
            };
            if mapping.source_type != SourceType::Uuid {
                return Err(CatalogError::InvalidTenant {
                    relation: relation.to_owned(),
                    column: column.clone(),
                });
            }
            Some(column.as_str())
        }
        TenantMode::Fixed => None,
    };
    if !primary_key
        .iter()
        .all(|column| identity.contains(column.as_str()))
        || tenant_column.is_some_and(|column| !identity.contains(column))
    {
        return Err(CatalogError::ReplicaIdentity(relation.to_owned()));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn classify_type(
    relation: &str,
    column: &str,
    name: &str,
    namespace: &str,
    kind: &str,
    base_name: &str,
    base_namespace: &str,
    configured_unsigned: bool,
) -> Result<SourceType, CatalogError> {
    if kind == "d" {
        if configured_unsigned
            && base_namespace == "pg_catalog"
            && matches!(base_name, "int2" | "int4" | "int8" | "numeric")
        {
            return Ok(SourceType::UnsignedInteger);
        }
        return Err(CatalogError::UnsupportedType {
            relation: relation.to_owned(),
            column: column.to_owned(),
            source_type: format!("{namespace}.{name}"),
        });
    }
    if configured_unsigned {
        return Err(CatalogError::UnsupportedType {
            relation: relation.to_owned(),
            column: column.to_owned(),
            source_type: format!("unsigned mapping requires a domain, got {namespace}.{name}"),
        });
    }
    let source_type = if kind == "e" {
        Some(SourceType::Text)
    } else if namespace != "pg_catalog" {
        None
    } else {
        match name {
            "bool" => Some(SourceType::Bool),
            "int2" | "int4" | "int8" => Some(SourceType::SignedInteger),
            "float4" | "float8" => Some(SourceType::Float),
            "text" | "varchar" | "bpchar" | "name" => Some(SourceType::Text),
            "bytea" => Some(SourceType::Bytes),
            "numeric" => Some(SourceType::Decimal),
            "uuid" => Some(SourceType::Uuid),
            "timestamptz" => Some(SourceType::Timestamp),
            "date" => Some(SourceType::Date),
            "time" | "timetz" => Some(SourceType::Time),
            "timestamp" => Some(SourceType::TimestampWithoutZone),
            "json" | "jsonb" => Some(SourceType::Json),
            _ => None,
        }
    };
    source_type.ok_or_else(|| CatalogError::UnsupportedType {
        relation: relation.to_owned(),
        column: column.to_owned(),
        source_type: format!("{namespace}.{name}"),
    })
}

pub(super) fn expected_replica_identity(relation: &RelationMapping) -> ReplicaIdentity {
    if relation.columns.iter().all(|column| column.identity) {
        ReplicaIdentity::Full
    } else {
        let identity: Vec<&str> = relation
            .columns
            .iter()
            .filter(|column| column.identity)
            .map(|column| column.name.as_str())
            .collect();
        let primary: Vec<&str> = relation
            .primary_key
            .iter()
            .map(|index| relation.columns[*index].name.as_str())
            .collect();
        if identity == primary {
            ReplicaIdentity::Default
        } else {
            ReplicaIdentity::Index
        }
    }
}
