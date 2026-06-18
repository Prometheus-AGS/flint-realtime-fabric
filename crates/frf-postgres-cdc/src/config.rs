use frf_domain::TenantId;

/// Configuration for the Postgres CDC consumer.
#[derive(Debug, Clone)]
pub struct CdcConfig {
    pub database_url: String,
    pub slot_name: String,
    pub publication_name: String,
    pub tenant_id: TenantId,
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
            channel_path: channel_path.into(),
            lsn_checkpoint_interval: 1000,
        }
    }

    /// Return the database URL with `replication=database` appended.
    ///
    /// `pg_walstream` requires this query parameter to open a replication
    /// connection rather than a standard query connection.
    #[must_use]
    pub fn replication_url(&self) -> String {
        if self.database_url.contains('?') {
            format!("{}&replication=database", self.database_url)
        } else {
            format!("{}?replication=database", self.database_url)
        }
    }
}
