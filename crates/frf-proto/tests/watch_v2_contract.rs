#![allow(clippy::expect_used)]

use frf_proto::fv2::{
    BrokerPosition, ChangeOp, ClientCheckpoint, EntityKey, EntityMutation, EntityRecord,
    EntityType, KeyPart, LiveStart, ResnapshotReason, ResnapshotRequired, SourcePosition,
    WatchEntityTypeRequest, WatchEntityTypeResponse, canonical_value, watch_entity_type_request,
    watch_entity_type_response,
};
use prost::Message;

fn checkpoint() -> ClientCheckpoint {
    ClientCheckpoint {
        version: 1,
        token: b"opaque-authorized-checkpoint".to_vec(),
        generation: 7,
    }
}

fn entity_type() -> EntityType {
    EntityType {
        schema: "public".to_owned(),
        name: "orders".to_owned(),
        projection: "default".to_owned(),
    }
}

#[test]
fn v1_entity_watch_and_v2_type_watch_namespaces_coexist() {
    let v1 = frf_proto::fv1::WatchEntityRequest {
        entity_id: "entity-1".to_owned(),
        tenant_id: "tenant-1".to_owned(),
    };
    let v2 = WatchEntityTypeRequest {
        entity_type: Some(EntityType {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            projection: "default".to_owned(),
        }),
        tenant_id: "tenant-1".to_owned(),
        start: Some(watch_entity_type_request::Start::Live(LiveStart {})),
    };

    assert!(!v1.encode_to_vec().is_empty());
    assert!(!v2.encode_to_vec().is_empty());
}

#[test]
fn request_start_is_an_exclusive_oneof() {
    let request = WatchEntityTypeRequest {
        entity_type: Some(EntityType {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            projection: "default".to_owned(),
        }),
        tenant_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        start: Some(watch_entity_type_request::Start::Live(LiveStart {})),
    };

    let decoded = WatchEntityTypeRequest::decode(request.encode_to_vec().as_slice())
        .expect("v2 request must round-trip");
    assert!(matches!(
        decoded.start,
        Some(watch_entity_type_request::Start::Live(_))
    ));
}

#[test]
fn mutation_keeps_source_broker_event_and_checkpoint_distinct() {
    let mutation = EntityMutation {
        event_id: "frfevent:v1:stable".to_owned(),
        key: Some(EntityKey {
            parts: vec![KeyPart {
                column: "key".to_owned(),
                value: Some(frf_proto::fv2::CanonicalValue {
                    kind: Some(canonical_value::Kind::TextValue("case-42".to_owned())),
                }),
            }],
            canonical_id: "frfkey:v1:opaque".to_owned(),
        }),
        op: ChangeOp::Delete.into(),
        record: None,
        source: Some(SourcePosition {
            epoch: "epoch-a".to_owned(),
            commit_lsn: 123_456_789,
            transaction_index: 2,
        }),
        broker: Some(BrokerPosition {
            partition: 3,
            offset: 99,
        }),
        checkpoint: Some(checkpoint()),
        committed_at: None,
        entity_type: Some(entity_type()),
        tenant_id: "tenant-1".to_owned(),
    };
    let response = WatchEntityTypeResponse {
        frame: Some(watch_entity_type_response::Frame::Mutation(mutation)),
    };

    let decoded = WatchEntityTypeResponse::decode(response.encode_to_vec().as_slice())
        .expect("v2 response must round-trip");
    let Some(watch_entity_type_response::Frame::Mutation(decoded)) = decoded.frame else {
        panic!("expected mutation frame");
    };
    assert_eq!(decoded.event_id, "frfevent:v1:stable");
    assert_eq!(
        decoded.source.expect("source position").transaction_index,
        2
    );
    assert_eq!(decoded.broker.expect("broker position").offset, 99);
    assert_eq!(decoded.checkpoint.expect("client checkpoint").generation, 7);
    assert_eq!(decoded.entity_type.expect("entity type").name, "orders");
    assert_eq!(decoded.tenant_id, "tenant-1");
    assert!(decoded.record.is_none(), "delete must not carry a record");
}

#[test]
fn resnapshot_is_a_typed_terminal_control_frame() {
    let response = WatchEntityTypeResponse {
        frame: Some(watch_entity_type_response::Frame::ResnapshotRequired(
            ResnapshotRequired {
                reason: ResnapshotReason::HistoryExpired.into(),
                message: "retained history no longer covers this checkpoint".to_owned(),
            },
        )),
    };

    let decoded = WatchEntityTypeResponse::decode(response.encode_to_vec().as_slice())
        .expect("terminal response must round-trip");
    let Some(watch_entity_type_response::Frame::ResnapshotRequired(control)) = decoded.frame else {
        panic!("expected resnapshot control");
    };
    assert_eq!(control.reason, i32::from(ResnapshotReason::HistoryExpired));
}

#[test]
fn projected_record_fields_are_typed_not_json_structs() {
    let record = EntityRecord {
        fields: vec![frf_proto::fv2::EntityField {
            column: "amount".to_owned(),
            value: Some(frf_proto::fv2::CanonicalValue {
                kind: Some(canonical_value::Kind::DecimalValue("123.45".to_owned())),
            }),
        }],
    };

    assert!(matches!(
        record.fields[0].value.as_ref().and_then(|value| value.kind.as_ref()),
        Some(canonical_value::Kind::DecimalValue(value)) if value == "123.45"
    ));
}
