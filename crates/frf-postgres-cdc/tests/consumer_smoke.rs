use frf_domain::TenantId;
use frf_postgres_cdc::CdcConfig;
use uuid::Uuid;

#[test]
fn cdc_config_builds_from_fields() {
    let tenant = TenantId::from_uuid(Uuid::nil());
    let config = CdcConfig::new(
        "postgres://user:pass@localhost/mydb",
        "frf_cdc_slot",
        "frf_pub",
        tenant,
        "entity/changes",
    );

    assert_eq!(config.slot_name, "frf_cdc_slot");
    assert_eq!(config.publication_name, "frf_pub");
    assert_eq!(config.channel_path, "entity/changes");
    assert_eq!(config.lsn_checkpoint_interval, 1000);
}

#[test]
fn replication_url_appends_query_param_when_no_existing_query() {
    let tenant = TenantId::from_uuid(Uuid::nil());
    let config = CdcConfig::new("postgres://localhost/mydb", "slot", "pub", tenant, "path");

    let url = config.replication_url();
    assert!(url.ends_with("?replication=database"), "got: {url}");
}

#[test]
fn replication_url_appends_query_param_when_existing_query() {
    let tenant = TenantId::from_uuid(Uuid::nil());
    let config = CdcConfig::new(
        "postgres://localhost/mydb?sslmode=require",
        "slot",
        "pub",
        tenant,
        "path",
    );

    let url = config.replication_url();
    assert!(url.ends_with("&replication=database"), "got: {url}");
}
