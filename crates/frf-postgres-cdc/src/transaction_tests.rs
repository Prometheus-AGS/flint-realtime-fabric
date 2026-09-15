#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use chrono::Utc;
use frf_domain::{ChangeOp, TenantId};
use pg_walstream::{ColumnValue, EventType, Lsn, RelationColumn, ReplicaIdentity, RowData};
use uuid::Uuid;

use crate::{
    canonical::SourceType,
    catalog::{Catalog, ColumnMapping, RelationMapping},
    config::TenantMode,
    transaction::{TransactionAssembler, TransactionError},
};

const RELATION_OID: u32 = 42;

fn relation() -> RelationMapping {
    RelationMapping {
        oid: RELATION_OID,
        schema: "sales".to_owned(),
        table: "orders".to_owned(),
        projection: "summary".to_owned(),
        columns: vec![
            column("tenant_id", 2950, SourceType::Uuid, true, true),
            column("region", 25, SourceType::Text, true, true),
            column("number", 23, SourceType::SignedInteger, true, true),
            column("description", 25, SourceType::Text, true, true),
        ],
        primary_key: vec![1, 2],
        tenant: TenantMode::Column {
            column: "tenant_id".to_owned(),
        },
    }
}

fn column(
    name: &str,
    type_id: u32,
    source_type: SourceType,
    projected: bool,
    identity: bool,
) -> ColumnMapping {
    ColumnMapping {
        name: name.to_owned(),
        type_id,
        type_modifier: -1,
        source_type,
        projected,
        identity,
    }
}

fn catalog() -> Catalog {
    Catalog::from_relations(vec![relation()])
}

fn stream_relation() -> EventType {
    EventType::Relation {
        relation_id: RELATION_OID,
        namespace: Arc::from("sales"),
        relation_name: Arc::from("orders"),
        replica_identity: ReplicaIdentity::Full,
        columns: relation()
            .columns
            .into_iter()
            .map(|column| RelationColumn {
                name: Arc::from(column.name),
                type_id: column.type_id,
                type_modifier: column.type_modifier,
                is_key: column.identity,
            })
            .collect(),
    }
}

fn row(tenant: Uuid, region: &str, number: i32, description: Option<&str>) -> RowData {
    let mut row = RowData::new();
    row.push(
        Arc::from("tenant_id"),
        ColumnValue::text(&tenant.to_string()),
    );
    row.push(Arc::from("region"), ColumnValue::text(region));
    row.push(Arc::from("number"), ColumnValue::text(&number.to_string()));
    if let Some(description) = description {
        row.push(Arc::from("description"), ColumnValue::text(description));
    }
    row
}

fn insert(data: RowData) -> EventType {
    EventType::Insert {
        schema: Arc::from("sales"),
        table: Arc::from("orders"),
        relation_oid: RELATION_OID,
        data,
    }
}

fn begin() -> EventType {
    EventType::Begin {
        transaction_id: 7,
        final_lsn: Lsn(100),
        commit_timestamp: Utc::now(),
    }
}

fn commit() -> EventType {
    EventType::Commit {
        commit_timestamp: Utc::now(),
        commit_lsn: Lsn(100),
        end_lsn: Lsn(120),
    }
}

fn accept(
    assembler: &mut TransactionAssembler,
    event: EventType,
) -> Result<Option<crate::transaction::CommittedTransaction>, TransactionError> {
    assembler.accept(
        event,
        &catalog(),
        TenantId::from_uuid(Uuid::nil()),
        "cluster-slot-1",
    )
}

#[test]
fn multi_row_commit_is_atomic_and_uses_composite_non_uuid_keys() {
    let tenant = Uuid::new_v4();
    let mut assembler = TransactionAssembler::default();
    assert!(accept(&mut assembler, stream_relation()).unwrap().is_none());
    assert!(accept(&mut assembler, begin()).unwrap().is_none());
    assert!(
        accept(&mut assembler, insert(row(tenant, "us", 7, Some("one"))))
            .unwrap()
            .is_none()
    );
    assert!(
        accept(&mut assembler, insert(row(tenant, "eu", 8, Some("two"))))
            .unwrap()
            .is_none()
    );

    let committed = accept(&mut assembler, commit()).unwrap().unwrap();
    assert_eq!(committed.end_lsn, 120);
    assert_eq!(committed.mutations.len(), 2);
    let first = &committed.mutations[0].0;
    assert_eq!(first.entity_type, "sales.orders@summary");
    assert_eq!(first.tenant_id, TenantId::from_uuid(tenant));
    assert_eq!(first.key.parts[0].column, "region");
    assert_eq!(first.key.parts[1].column, "number");
    assert_eq!(first.source.transaction_index, 0);
    assert_eq!(committed.mutations[1].0.source.transaction_index, 1);
}

#[test]
fn replayed_commit_retains_stable_event_and_envelope_ids() {
    let tenant = Uuid::new_v4();
    let run = || {
        let mut assembler = TransactionAssembler::default();
        accept(&mut assembler, stream_relation()).unwrap();
        accept(&mut assembler, begin()).unwrap();
        accept(&mut assembler, insert(row(tenant, "us", 7, Some("same")))).unwrap();
        accept(&mut assembler, commit()).unwrap().unwrap()
    };
    let first = run();
    let retry = run();
    assert_eq!(first.mutations[0].0.event_id, retry.mutations[0].0.event_id);
    assert_eq!(first.mutations[0].1, retry.mutations[0].1);
}

#[test]
fn unchanged_toast_is_explicit_in_update() {
    let tenant = Uuid::new_v4();
    let mut assembler = TransactionAssembler::default();
    accept(&mut assembler, stream_relation()).unwrap();
    accept(&mut assembler, begin()).unwrap();
    let update = EventType::Update {
        schema: Arc::from("sales"),
        table: Arc::from("orders"),
        relation_oid: RELATION_OID,
        old_data: None,
        new_data: row(tenant, "us", 7, None),
        replica_identity: ReplicaIdentity::Full,
        key_columns: vec![
            Arc::from("tenant_id"),
            Arc::from("region"),
            Arc::from("number"),
            Arc::from("description"),
        ],
    };
    accept(&mut assembler, update).unwrap();
    let committed = accept(&mut assembler, commit()).unwrap().unwrap();
    assert_eq!(committed.mutations[0].0.op, ChangeOp::Update);
    assert_eq!(
        committed.mutations[0].0.unchanged_toast,
        vec!["description"]
    );
}

#[test]
fn missing_old_key_poison_blocks_the_transaction() {
    let tenant = Uuid::new_v4();
    let mut assembler = TransactionAssembler::default();
    accept(&mut assembler, stream_relation()).unwrap();
    accept(&mut assembler, begin()).unwrap();
    let mut incomplete_old = RowData::new();
    incomplete_old.push(
        Arc::from("tenant_id"),
        ColumnValue::text(&tenant.to_string()),
    );
    incomplete_old.push(Arc::from("region"), ColumnValue::text("us"));
    let update = EventType::Update {
        schema: Arc::from("sales"),
        table: Arc::from("orders"),
        relation_oid: RELATION_OID,
        old_data: Some(incomplete_old),
        new_data: row(tenant, "eu", 8, Some("moved")),
        replica_identity: ReplicaIdentity::Full,
        key_columns: vec![Arc::from("region"), Arc::from("number")],
    };
    assert!(matches!(
        accept(&mut assembler, update),
        Err(TransactionError::Decode(
            crate::decode::DecodeError::MissingOldKey(_)
        ))
    ));
}

#[test]
fn absent_old_tuple_requires_identity_proof_for_every_routing_column() {
    let tenant = Uuid::new_v4();
    let mut assembler = TransactionAssembler::default();
    accept(&mut assembler, stream_relation()).unwrap();
    accept(&mut assembler, begin()).unwrap();
    let update = EventType::Update {
        schema: Arc::from("sales"),
        table: Arc::from("orders"),
        relation_oid: RELATION_OID,
        old_data: None,
        new_data: row(tenant, "us", 7, Some("changed")),
        replica_identity: ReplicaIdentity::Full,
        key_columns: vec![Arc::from("region"), Arc::from("number")],
    };
    assert!(matches!(
        accept(&mut assembler, update),
        Err(TransactionError::Decode(
            crate::decode::DecodeError::MissingOldKey(_)
        ))
    ));
}

#[test]
fn schema_drift_and_poison_tenant_stop_before_commit() {
    let mut assembler = TransactionAssembler::default();
    let mut changed = relation();
    changed.columns[3].type_id = 23;
    let event = EventType::Relation {
        relation_id: RELATION_OID,
        namespace: Arc::from("sales"),
        relation_name: Arc::from("orders"),
        replica_identity: ReplicaIdentity::Full,
        columns: changed
            .columns
            .into_iter()
            .map(|column| RelationColumn {
                name: Arc::from(column.name),
                type_id: column.type_id,
                type_modifier: column.type_modifier,
                is_key: true,
            })
            .collect(),
    };
    assert!(matches!(
        accept(&mut assembler, event),
        Err(TransactionError::Catalog(_))
    ));

    let mut assembler = TransactionAssembler::default();
    accept(&mut assembler, stream_relation()).unwrap();
    accept(&mut assembler, begin()).unwrap();
    let mut poison = row(Uuid::nil(), "us", 7, Some("bad"));
    poison = {
        let mut replacement = RowData::new();
        replacement.push(Arc::from("tenant_id"), ColumnValue::text("not-a-uuid"));
        for (name, value) in poison.iter().skip(1) {
            replacement.push(Arc::clone(name), value.clone());
        }
        replacement
    };
    assert!(matches!(
        accept(&mut assembler, insert(poison)),
        Err(TransactionError::Decode(_))
    ));
}

#[test]
fn catalog_bootstrap_handles_suppressed_initial_relation_event() {
    let tenant = Uuid::new_v4();
    let catalog = catalog();
    let mut assembler = TransactionAssembler::from_catalog(&catalog);
    accept(&mut assembler, begin()).unwrap();
    assert!(
        accept(
            &mut assembler,
            insert(row(tenant, "us", 1, Some("catalog validated")))
        )
        .unwrap()
        .is_none()
    );
}
