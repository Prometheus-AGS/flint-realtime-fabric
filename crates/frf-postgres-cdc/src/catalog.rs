use std::collections::{HashMap, HashSet};

use pg_walstream::{RelationColumn, ReplicaIdentity};
use tokio_postgres::{Client, NoTls};

use crate::{
    canonical::SourceType,
    catalog_validation::{
        classify_type, expected_replica_identity, validate_columns, validate_config,
    },
    config::{CdcConfig, TableEnrollment, TenantMode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ColumnMapping {
    pub name: String,
    pub type_id: u32,
    pub type_modifier: i32,
    pub source_type: SourceType,
    pub projected: bool,
    pub identity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationMapping {
    pub oid: u32,
    pub schema: String,
    pub table: String,
    pub projection: String,
    pub columns: Vec<ColumnMapping>,
    pub primary_key: Vec<usize>,
    pub tenant: TenantMode,
}

impl RelationMapping {
    #[must_use]
    pub fn entity_type(&self) -> String {
        format!("{}.{}@{}", self.schema, self.table, self.projection)
    }

    #[must_use]
    pub fn column(&self, name: &str) -> Option<&ColumnMapping> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// PostgreSQL omits an UPDATE old tuple only when its replica-identity
    /// columns did not change. Accept that form only when parser metadata proves
    /// every routing key, including a column tenant, is in the identity.
    #[must_use]
    pub fn update_without_old_key_is_safe(
        &self,
        replica_identity: ReplicaIdentity,
        key_columns: &[std::sync::Arc<str>],
    ) -> bool {
        if expected_replica_identity(self) != replica_identity {
            return false;
        }
        let is_key = |name: &str| key_columns.iter().any(|key| key.as_ref() == name);
        self.primary_key
            .iter()
            .all(|index| is_key(&self.columns[*index].name))
            && match &self.tenant {
                TenantMode::Column { column } => is_key(column),
                TenantMode::Fixed => true,
            }
    }

    pub fn validate_stream_relation(
        &self,
        schema: &str,
        table: &str,
        replica_identity: ReplicaIdentity,
        columns: &[RelationColumn],
    ) -> Result<(), CatalogError> {
        let expected_identity = expected_replica_identity(self);
        if self.schema != schema
            || self.table != table
            || expected_identity != replica_identity
            || self.columns.len() != columns.len()
        {
            return Err(CatalogError::SchemaChanged(self.entity_type()));
        }
        for (expected, actual) in self.columns.iter().zip(columns) {
            if expected.name != actual.name.as_ref()
                || expected.type_id != actual.type_id
                || expected.type_modifier != actual.type_modifier
                || expected.identity != actual.is_key
            {
                return Err(CatalogError::SchemaChanged(self.entity_type()));
            }
        }
        Ok(())
    }
}

pub(crate) struct Catalog {
    by_oid: HashMap<u32, RelationMapping>,
    client: Option<Client>,
}

impl Catalog {
    pub async fn load(config: &CdcConfig) -> Result<Self, CatalogError> {
        validate_config(config)?;
        let (client, connection) = tokio_postgres::connect(&config.catalog_url(), NoTls)
            .await
            .map_err(|error| CatalogError::Connection(error.to_string()))?;
        let _connection_task = tokio::spawn(async move {
            if let Err(error) = connection.await {
                tracing::warn!(%error, "CDC catalog connection stopped");
            }
        });
        load_relations(&client, config).await.map(|relations| Self {
            by_oid: relations
                .into_iter()
                .map(|relation| (relation.oid, relation))
                .collect(),
            client: Some(client),
        })
    }

    #[must_use]
    pub fn relation(&self, oid: u32) -> Option<&RelationMapping> {
        self.by_oid.get(&oid)
    }

    pub fn relation_oids(&self) -> impl Iterator<Item = u32> + '_ {
        self.by_oid.keys().copied()
    }

    /// Re-read enrollment metadata at a source commit boundary. This closes the
    /// race created by `pg_walstream` 0.6.3 consuming each stream's first
    /// Relation frame internally: schema DDL committed before this source
    /// transaction cannot pass publication or checkpointing under stale types.
    pub async fn revalidate(&self, config: &CdcConfig) -> Result<(), CatalogError> {
        let Some(client) = &self.client else {
            return Ok(());
        };
        let current: HashMap<u32, RelationMapping> = load_relations(client, config)
            .await?
            .into_iter()
            .map(|relation| (relation.oid, relation))
            .collect();
        if current == self.by_oid {
            return Ok(());
        }
        let changed = self
            .by_oid
            .values()
            .find(|relation| current.get(&relation.oid) != Some(*relation))
            .map_or_else(
                || "enrolled catalog".to_owned(),
                RelationMapping::entity_type,
            );
        Err(CatalogError::SchemaChanged(changed))
    }

    #[cfg(test)]
    pub(crate) fn from_relations(relations: Vec<RelationMapping>) -> Self {
        Self {
            by_oid: relations
                .into_iter()
                .map(|relation| (relation.oid, relation))
                .collect(),
            client: None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("CDC catalog connection failed: {0}")]
    Connection(String),
    #[error("CDC source epoch must be 1-128 safe ASCII characters")]
    InvalidEpoch,
    #[error("CDC requires at least one explicit table enrollment")]
    EmptyEnrollment,
    #[error("CDC LSN checkpoint interval must be greater than zero")]
    InvalidCheckpointInterval,
    #[error("invalid CDC identifier '{0}'")]
    InvalidIdentifier(String),
    #[error("duplicate CDC enrollment for '{0}'")]
    DuplicateEnrollment(String),
    #[error("publication '{0}' does not exist")]
    PublicationMissing(String),
    #[error("publication '{0}' is FOR ALL TABLES; explicit enrollment is required")]
    PublicationAllTables(String),
    #[error("publication '{0}' must publish INSERT, UPDATE, and DELETE")]
    PublicationActions(String),
    #[error("enrolled table '{0}' is not in the configured publication")]
    TableNotPublished(String),
    #[error("enrolled table '{0}' has a publication row filter")]
    PublicationRowFilter(String),
    #[error("enrolled table '{0}' does not exist")]
    TableMissing(String),
    #[error("enrolled relation '{0}' must be an ordinary table")]
    UnsupportedRelation(String),
    #[error("enrolled table '{0}' has no primary key")]
    MissingPrimaryKey(String),
    #[error("enrolled table '{0}' has inadequate replica identity for its key/tenant")]
    ReplicaIdentity(String),
    #[error("enrollment '{relation}' has duplicate or unknown projection column '{column}'")]
    InvalidProjection { relation: String, column: String },
    #[error("enrollment '{relation}' has an invalid tenant column '{column}'")]
    InvalidTenant { relation: String, column: String },
    #[error("unsupported PostgreSQL type '{source_type}' for '{relation}.{column}'")]
    UnsupportedType {
        relation: String,
        column: String,
        source_type: String,
    },
    #[error("relation metadata changed for '{0}'; enrollment must be reviewed")]
    SchemaChanged(String),
}

async fn load_relations(
    client: &Client,
    config: &CdcConfig,
) -> Result<Vec<RelationMapping>, CatalogError> {
    let publication = client
        .query_opt(
            "SELECT puballtables, pubinsert, pubupdate, pubdelete
             FROM pg_catalog.pg_publication WHERE pubname = $1",
            &[&config.publication_name],
        )
        .await
        .map_err(|error| CatalogError::Connection(error.to_string()))?
        .ok_or_else(|| CatalogError::PublicationMissing(config.publication_name.clone()))?;
    if publication.get::<_, bool>(0) {
        return Err(CatalogError::PublicationAllTables(
            config.publication_name.clone(),
        ));
    }
    if !(publication.get::<_, bool>(1)
        && publication.get::<_, bool>(2)
        && publication.get::<_, bool>(3))
    {
        return Err(CatalogError::PublicationActions(
            config.publication_name.clone(),
        ));
    }

    let mut relations = Vec::with_capacity(config.enrollments.len());
    for enrollment in &config.enrollments {
        relations.push(load_relation(client, &config.publication_name, enrollment).await?);
    }
    Ok(relations)
}

#[allow(clippy::too_many_lines)]
async fn load_relation(
    client: &Client,
    publication: &str,
    enrollment: &TableEnrollment,
) -> Result<RelationMapping, CatalogError> {
    let qualified = format!("{}.{}", enrollment.schema, enrollment.table);
    let published = client
        .query_opt(
            "SELECT attnames, rowfilter FROM pg_catalog.pg_publication_tables
             WHERE pubname = $1 AND schemaname = $2 AND tablename = $3",
            &[&publication, &enrollment.schema, &enrollment.table],
        )
        .await
        .map_err(|error| CatalogError::Connection(error.to_string()))?
        .ok_or_else(|| CatalogError::TableNotPublished(qualified.clone()))?;
    let published_columns: Option<HashSet<String>> = published
        .get::<_, Option<Vec<String>>>(0)
        .map(|columns| columns.into_iter().collect());
    if published.get::<_, Option<String>>(1).is_some() {
        return Err(CatalogError::PublicationRowFilter(qualified));
    }

    let table = client
        .query_opt(
            "SELECT c.oid::bigint, c.relreplident::text, c.relkind::text \
             FROM pg_catalog.pg_class c \
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = $1 AND c.relname = $2",
            &[&enrollment.schema, &enrollment.table],
        )
        .await
        .map_err(|error| CatalogError::Connection(error.to_string()))?
        .ok_or_else(|| CatalogError::TableMissing(qualified.clone()))?;
    let oid = u32::try_from(table.get::<_, i64>(0))
        .map_err(|_| CatalogError::TableMissing(qualified.clone()))?;
    let replica_identity = table.get::<_, String>(1);
    if table.get::<_, String>(2) != "r" {
        return Err(CatalogError::UnsupportedRelation(qualified));
    }

    let rows = client
        .query(
            "SELECT a.attname, a.atttypid::bigint, a.atttypmod, \
                    t.typname, tn.nspname, t.typtype::text, \
                    COALESCE(bt.typname, ''), COALESCE(btn.nspname, '') \
             FROM pg_catalog.pg_attribute a \
             JOIN pg_catalog.pg_type t ON t.oid = a.atttypid \
             JOIN pg_catalog.pg_namespace tn ON tn.oid = t.typnamespace \
             LEFT JOIN pg_catalog.pg_type bt ON bt.oid = t.typbasetype \
             LEFT JOIN pg_catalog.pg_namespace btn ON btn.oid = bt.typnamespace \
             WHERE a.attrelid::bigint = $1 AND a.attnum > 0 AND NOT a.attisdropped \
             ORDER BY a.attnum",
            &[&(i64::from(oid))],
        )
        .await
        .map_err(|error| CatalogError::Connection(error.to_string()))?;

    let primary_key = index_columns(client, oid, true).await?;
    if primary_key.is_empty() {
        return Err(CatalogError::MissingPrimaryKey(qualified));
    }
    let identity_names = match replica_identity.as_str() {
        "f" => rows.iter().map(|row| row.get::<_, String>(0)).collect(),
        "d" => primary_key.clone(),
        "i" => index_columns(client, oid, false).await?,
        _ => Vec::new(),
    };
    let identity: HashSet<&str> = identity_names.iter().map(String::as_str).collect();
    let projected: HashSet<&str> = enrollment.columns.iter().map(String::as_str).collect();
    if projected.len() != enrollment.columns.len() {
        return Err(CatalogError::InvalidProjection {
            relation: qualified,
            column: "duplicate".to_owned(),
        });
    }
    let unsigned: HashSet<&str> = enrollment
        .unsigned_columns
        .iter()
        .map(String::as_str)
        .collect();

    let mut columns = Vec::with_capacity(rows.len());
    for row in rows {
        let name = row.get::<_, String>(0);
        if published_columns
            .as_ref()
            .is_some_and(|columns| !columns.contains(&name))
        {
            continue;
        }
        let type_id =
            u32::try_from(row.get::<_, i64>(1)).map_err(|_| CatalogError::UnsupportedType {
                relation: qualified.clone(),
                column: name.clone(),
                source_type: "OID overflow".to_owned(),
            })?;
        let source_type = classify_type(
            &qualified,
            &name,
            &row.get::<_, String>(3),
            &row.get::<_, String>(4),
            &row.get::<_, String>(5),
            &row.get::<_, String>(6),
            &row.get::<_, String>(7),
            unsigned.contains(name.as_str()),
        )?;
        columns.push(ColumnMapping {
            name: name.clone(),
            type_id,
            type_modifier: row.get(2),
            source_type,
            projected: projected.contains(name.as_str()),
            identity: identity.contains(name.as_str()),
        });
    }
    validate_columns(&qualified, enrollment, &columns, &primary_key, &identity)?;
    let primary_key = primary_key
        .iter()
        .map(|name| {
            columns
                .iter()
                .position(|column| column.name == *name)
                .ok_or_else(|| CatalogError::MissingPrimaryKey(qualified.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RelationMapping {
        oid,
        schema: enrollment.schema.clone(),
        table: enrollment.table.clone(),
        projection: enrollment.projection.clone(),
        columns,
        primary_key,
        tenant: enrollment.tenant.clone(),
    })
}

async fn index_columns(
    client: &Client,
    oid: u32,
    primary: bool,
) -> Result<Vec<String>, CatalogError> {
    let predicate = if primary {
        "i.indisprimary"
    } else {
        "i.indisreplident"
    };
    let sql = format!(
        "SELECT a.attname FROM pg_catalog.pg_index i \
         CROSS JOIN LATERAL unnest(i.indkey::smallint[]) WITH ORDINALITY AS k(attnum, ord) \
         JOIN pg_catalog.pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = k.attnum \
         WHERE i.indrelid::bigint = $1 AND {predicate} ORDER BY k.ord"
    );
    client
        .query(&sql, &[&(i64::from(oid))])
        .await
        .map(|rows| rows.into_iter().map(|row| row.get(0)).collect())
        .map_err(|error| CatalogError::Connection(error.to_string()))
}
