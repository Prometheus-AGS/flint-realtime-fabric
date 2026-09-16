#![allow(clippy::expect_used)]

use frf_proto::fv2::{
    EntityType, LiveStart, WatchEntityTypeRequest, entity_service_client::EntityServiceClient,
    watch_entity_type_request,
};

#[tokio::test]
async fn forge_facing_tonic_client_surface_compiles() {
    let channel = tonic::transport::Endpoint::from_static("http://127.0.0.1:9090").connect_lazy();
    let mut client = EntityServiceClient::new(channel);
    let request = WatchEntityTypeRequest {
        entity_type: Some(EntityType {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            projection: "default".to_owned(),
        }),
        tenant_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        start: Some(watch_entity_type_request::Start::Live(LiveStart {})),
    };

    let call = client.watch_entity_type(request);
    drop(call);
    let _concurrent_consumer = client.clone();
}
