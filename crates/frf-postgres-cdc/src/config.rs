use frf_domain::TenantId;
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum TenantMode {
    /// Read the effective tenant UUID from this enrolled source column.
    Column { column: String },
    /// Use [`CdcConfig::tenant_id`] for a deliberately global/fixed source.
    Fixed,
}

/// Server-owned enrollment for one schema-qualified source table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableEnrollment {
    pub schema: String,
    pub table: String,
    pub projection: String,
    /// Approved projection membership; emitted fields retain source column order.
    pub columns: Vec<String>,
    pub tenant: TenantMode,
    /// Domain columns explicitly approved as unsigned integers.
    #[serde(default)]
    pub unsigned_columns: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("CDC enrollment JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CDC source epoch must be 1-128 safe ASCII characters")]
    InvalidSourceEpoch,
}

/// Configuration for the Postgres CDC consumer.
#[derive(Debug, Clone)]
pub struct CdcConfig {
    pub database_url: String,
    pub slot_name: String,
    pub publication_name: String,
    pub tenant_id: TenantId,
    /// Stable identity of the logical history. Rotate it when the slot/history
    /// is recreated, and keep it unchanged across ordinary process restarts.
    pub source_epoch: Option<String>,
    /// Explicit schema-qualified allowlist and projection contract.
    pub enrollments: Vec<TableEnrollment>,
    /// Logical channel path on the event spine (e.g. `"entity/changes"`).
    pub channel_path: String,
    /// How many WAL messages to process before sending a `StandbyStatusUpdate`.
    pub lsn_checkpoint_interval: u64,
}

impl CdcConfig {
    #[must_use]
    pub fn new(
        database_url: impl Into<String>,
        slot_name: impl Into<String>,
        publication_name: impl Into<String>,
        tenant_id: TenantId,
        channel_path: impl Into<String>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            slot_name: slot_name.into(),
            publication_name: publication_name.into(),
            tenant_id,
            source_epoch: None,
            enrollments: Vec::new(),
            channel_path: channel_path.into(),
            lsn_checkpoint_interval: 1000,
        }
    }

    #[must_use]
    pub fn with_source_epoch(mut self, source_epoch: impl Into<String>) -> Self {
        self.source_epoch = Some(source_epoch.into());
        self
    }

    #[must_use]
    pub fn with_enrollments(mut self, enrollments: Vec<TableEnrollment>) -> Self {
        self.enrollments = enrollments;
        self
    }

    /// Parse the server-owned enrollment JSON supplied by deployment config.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Json`] for malformed JSON. Catalog validation is
    /// deliberately performed against PostgreSQL before replication starts.
    pub fn parse_enrollments(json: &str) -> Result<Vec<TableEnrollment>, ConfigError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Validate the stable logical-history identifier shared by gateway and adapter.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidSourceEpoch`] when the value is empty, exceeds
    /// 128 bytes, or contains characters outside the portable configured set.
    pub fn validate_source_epoch(epoch: &str) -> Result<(), ConfigError> {
        if epoch.is_empty()
            || epoch.len() > 128
            || !epoch
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._:-".contains(character))
        {
            return Err(ConfigError::InvalidSourceEpoch);
        }
        Ok(())
    }

    /// Return the database URL with `replication=database` appended.
    ///
    /// `pg_walstream` requires this query parameter to open a replication
    /// connection rather than a standard query connection.
    #[must_use]
    pub fn replication_url(&self) -> String {
        if self
            .database_url
            .split(['?', '&'])
            .any(|part| part.starts_with("replication="))
        {
            return self.database_url.clone();
        }
        if self.database_url.contains('?') {
            format!("{}&replication=database", self.database_url)
        } else {
            format!("{}?replication=database", self.database_url)
        }
    }

    /// Return a normal SQL connection URL for catalog enrollment queries.
    #[must_use]
    pub fn catalog_url(&self) -> String {
        let Some((base, query)) = self.database_url.split_once('?') else {
            return self.database_url.clone();
        };
        let query = query
            .split('&')
            .filter(|part| !part.starts_with("replication="))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            base.to_owned()
        } else {
            format!("{base}?{query}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CdcConfig, ConfigError};

    #[test]
    fn source_epoch_accepts_documented_portable_form() {
        CdcConfig::validate_source_epoch("c008-core_v1:restore.2")
            .expect("documented source epoch must be accepted");
    }

    #[test]
    fn source_epoch_rejects_unsafe_or_oversized_values() {
        assert!(matches!(
            CdcConfig::validate_source_epoch("restore/2"),
            Err(ConfigError::InvalidSourceEpoch)
        ));
        assert!(matches!(
            CdcConfig::validate_source_epoch(&"a".repeat(129)),
            Err(ConfigError::InvalidSourceEpoch)
        ));
    }
}
