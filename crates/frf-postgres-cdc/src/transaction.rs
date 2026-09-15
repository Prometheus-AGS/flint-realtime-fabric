use std::collections::HashSet;

use chrono::{DateTime, Utc};
use frf_domain::TenantId;
use pg_walstream::{EventType, Lsn};

use crate::{
    canonical::{CanonicalError, stable_event_id},
    catalog::{Catalog, CatalogError},
    decode::{DecodeError, PendingMutation, decode_delete, decode_insert, decode_update},
    model::{CdcMutation, SourcePosition},
};

pub(crate) struct CommittedTransaction {
    pub end_lsn: u64,
    pub mutations: Vec<(CdcMutation, uuid::Uuid)>,
}

#[derive(Default)]
pub(crate) struct TransactionAssembler {
    active: Option<ActiveTransaction>,
    validated_relations: HashSet<u32>,
}

struct ActiveTransaction {
    transaction_id: u32,
    final_lsn: u64,
    pending: Vec<PendingMutation>,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("received transaction {incoming} while transaction {active} is active")]
    Nested { active: u32, incoming: u32 },
    #[error("received {event} outside a transaction")]
    OutsideTransaction { event: &'static str },
    #[error("commit LSN {commit_lsn} does not match BEGIN final LSN {final_lsn}")]
    CommitMismatch { final_lsn: u64, commit_lsn: u64 },
    #[error("row for enrolled relation OID {0} arrived before validated relation metadata")]
    RelationNotValidated(u32),
    #[error("transaction mutation count exceeds the source-position contract")]
    TooManyMutations,
    #[error("unsupported transactional event '{0}'")]
    Unsupported(&'static str),
}

impl TransactionAssembler {
    /// `pg_walstream` 0.6.3 consumes the first Relation frame internally and
    /// exposes later Relation frames only when its cached metadata changes.
    /// Catalog enrollment therefore establishes the initial validated set;
    /// any later relation frame is still checked as schema drift before rows.
    pub fn from_catalog(catalog: &Catalog) -> Self {
        Self {
            active: None,
            validated_relations: catalog.relation_oids().collect(),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn accept(
        &mut self,
        event_type: EventType,
        catalog: &Catalog,
        fixed_tenant: TenantId,
        epoch: &str,
    ) -> Result<Option<CommittedTransaction>, TransactionError> {
        match event_type {
            EventType::Begin {
                transaction_id,
                final_lsn,
                ..
            } => {
                if let Some(active) = &self.active {
                    return Err(TransactionError::Nested {
                        active: active.transaction_id,
                        incoming: transaction_id,
                    });
                }
                self.active = Some(ActiveTransaction {
                    transaction_id,
                    final_lsn: final_lsn.0,
                    pending: Vec::new(),
                });
                Ok(None)
            }
            EventType::Relation {
                relation_id,
                namespace,
                relation_name,
                replica_identity,
                columns,
            } => {
                if let Some(relation) = catalog.relation(relation_id) {
                    relation.validate_stream_relation(
                        &namespace,
                        &relation_name,
                        replica_identity,
                        &columns,
                    )?;
                    self.validated_relations.insert(relation_id);
                }
                Ok(None)
            }
            EventType::Insert {
                relation_oid, data, ..
            } => {
                if let Some(relation) = self.validated_relation(catalog, relation_oid)? {
                    let decoded = decode_insert(relation, fixed_tenant, &data)?;
                    self.active_mut("INSERT")?.pending.extend(decoded);
                } else {
                    self.active_mut("INSERT")?;
                }
                Ok(None)
            }
            EventType::Update {
                relation_oid,
                old_data,
                new_data,
                replica_identity,
                key_columns,
                ..
            } => {
                if let Some(relation) = self.validated_relation(catalog, relation_oid)? {
                    let decoded = decode_update(
                        relation,
                        fixed_tenant,
                        old_data.as_ref(),
                        &new_data,
                        replica_identity,
                        &key_columns,
                    )?;
                    self.active_mut("UPDATE")?.pending.extend(decoded);
                } else {
                    self.active_mut("UPDATE")?;
                }
                Ok(None)
            }
            EventType::Delete {
                relation_oid,
                old_data,
                ..
            } => {
                if let Some(relation) = self.validated_relation(catalog, relation_oid)? {
                    let decoded = decode_delete(relation, fixed_tenant, &old_data)?;
                    self.active_mut("DELETE")?.pending.extend(decoded);
                } else {
                    self.active_mut("DELETE")?;
                }
                Ok(None)
            }
            EventType::Commit {
                commit_timestamp,
                commit_lsn,
                end_lsn,
            } => self
                .commit(epoch, commit_timestamp, commit_lsn, end_lsn)
                .map(Some),
            EventType::Type { .. }
            | EventType::Origin { .. }
            | EventType::Message { flags: 0, .. } => Ok(None),
            EventType::Message { .. } => {
                self.active_mut("MESSAGE")?;
                Ok(None)
            }
            EventType::Truncate(_) => Err(TransactionError::Unsupported("TRUNCATE")),
            EventType::StreamStart { .. }
            | EventType::StreamStop
            | EventType::StreamCommit { .. }
            | EventType::StreamAbort { .. } => {
                Err(TransactionError::Unsupported("streaming transaction"))
            }
            EventType::BeginPrepare { .. }
            | EventType::Prepare { .. }
            | EventType::CommitPrepared { .. }
            | EventType::RollbackPrepared { .. }
            | EventType::StreamPrepare { .. } => {
                Err(TransactionError::Unsupported("two-phase transaction"))
            }
        }
    }

    fn validated_relation<'a>(
        &self,
        catalog: &'a Catalog,
        oid: u32,
    ) -> Result<Option<&'a crate::catalog::RelationMapping>, TransactionError> {
        let relation = catalog.relation(oid);
        if relation.is_some() && !self.validated_relations.contains(&oid) {
            return Err(TransactionError::RelationNotValidated(oid));
        }
        Ok(relation)
    }

    fn active_mut(
        &mut self,
        event: &'static str,
    ) -> Result<&mut ActiveTransaction, TransactionError> {
        self.active
            .as_mut()
            .ok_or(TransactionError::OutsideTransaction { event })
    }

    fn commit(
        &mut self,
        epoch: &str,
        committed_at: DateTime<Utc>,
        commit_lsn: Lsn,
        end_lsn: Lsn,
    ) -> Result<CommittedTransaction, TransactionError> {
        let active = self
            .active
            .take()
            .ok_or(TransactionError::OutsideTransaction { event: "COMMIT" })?;
        if active.final_lsn != commit_lsn.0 {
            return Err(TransactionError::CommitMismatch {
                final_lsn: active.final_lsn,
                commit_lsn: commit_lsn.0,
            });
        }
        let mut mutations = Vec::with_capacity(active.pending.len());
        for (index, pending) in active.pending.into_iter().enumerate() {
            let transaction_index =
                u32::try_from(index).map_err(|_| TransactionError::TooManyMutations)?;
            let (event_id, envelope_id) = stable_event_id(
                epoch,
                &pending.entity_type,
                &pending.key.canonical_id,
                commit_lsn.0,
                transaction_index,
            )?;
            mutations.push((
                CdcMutation {
                    event_id,
                    schema: pending.schema,
                    table: pending.table,
                    projection: pending.projection,
                    entity_type: pending.entity_type,
                    tenant_id: pending.tenant_id,
                    key: pending.key,
                    op: pending.op,
                    record: pending.record,
                    unchanged_toast: pending.unchanged_toast,
                    source: SourcePosition {
                        epoch: epoch.to_owned(),
                        commit_lsn: commit_lsn.0,
                        transaction_index,
                    },
                    committed_at,
                },
                envelope_id,
            ));
        }
        Ok(CommittedTransaction {
            end_lsn: end_lsn.0,
            mutations,
        })
    }
}
