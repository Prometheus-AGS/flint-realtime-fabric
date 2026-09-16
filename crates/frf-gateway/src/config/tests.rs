use super::*;

#[test]
fn federation_is_off_by_default() {
    // Safe-by-default: federation must never be silently active. The gateway
    // only wires Matrix/ATProto bridges when this is explicitly enabled.
    let cfg = GatewayConfig::test_default();
    assert!(!cfg.federation_enabled);
    assert!(cfg.federation_tenant_id.is_none());
    assert!(cfg.federation_channel_id.is_none());
}

#[test]
fn valid_default_config_passes_validation() {
    // test_default is Sovereign SFU + CDC off + federation off, with JWT_ISSUER set →
    // no unmet semantic requirements, so validation succeeds.
    GatewayConfig::test_default()
        .validate()
        .expect("default config should be valid");
}

#[test]
fn empty_optional_cdc_tenant_is_unset() {
    assert_eq!(
        parse_optional_uuid("CDC_TENANT_ID", Some(String::new())).unwrap(),
        None
    );
    assert_eq!(
        parse_optional_uuid("CDC_TENANT_ID", Some("  ".to_owned())).unwrap(),
        None
    );
}

#[test]
fn malformed_optional_cdc_tenant_is_rejected() {
    let error = parse_optional_uuid("CDC_TENANT_ID", Some("not-a-uuid".to_owned()))
        .expect_err("non-empty malformed UUID must fail");
    assert!(error.to_string().contains("CDC_TENANT_ID"));
}

// The issuer-mandatory rule only exists in production (non-`dev-endpoints`) builds;
// in dev builds a missing issuer is a warning, not a hard error (see `main.rs`).
#[cfg(not(feature = "dev-endpoints"))]
#[test]
fn production_config_without_jwt_issuer_fails_validation() {
    // Arrange: an otherwise-valid config missing JWT_ISSUER.
    let mut cfg = GatewayConfig::test_default();
    cfg.jwt_issuer = None;

    // Act
    let result = cfg.validate();

    // Assert: rejected, with a message naming the offending variable.
    let err = result.expect_err("missing JWT_ISSUER must fail validation in prod builds");
    assert!(
        err.to_string().contains("JWT_ISSUER"),
        "error should name JWT_ISSUER, got: {err}"
    );
}

#[test]
fn cdc_enabled_without_replication_url_fails_fast() {
    let mut cfg = GatewayConfig::test_default();
    cfg.cdc_enabled = true;
    cfg.cdc_replication_url = None;
    let err = cfg.validate().expect_err("expected validation to fail");
    assert!(
        err.to_string().contains("CDC_REPLICATION_URL"),
        "error should name the missing field, got: {err}"
    );
}

#[test]
fn full_profile_without_broker_fails_fast() {
    let mut cfg = GatewayConfig::test_default();
    cfg.iggy_connection_string.clear();
    let error = cfg.validate().expect_err("full profile requires a broker");
    assert!(error.to_string().contains("IGGY_CONNECTION_STRING"));
}

#[test]
fn cdc_enabled_with_all_fields_passes() {
    let mut cfg = GatewayConfig::test_default();
    cfg.cdc_enabled = true;
    cfg.cdc_replication_url = Some("postgres://x".to_owned());
    cfg.cdc_slot_name = Some("frf_slot".to_owned());
    cfg.cdc_publication_name = Some("frf_pub".to_owned());
    cfg.cdc_tenant_id = Some(uuid::Uuid::nil());
    cfg.cdc_channel_path = Some("entities".to_owned());
    cfg.cdc_source_epoch = Some("test-epoch-1".to_owned());
    cfg.cdc_enrollments_json = Some(
        r#"[{"schema":"public","table":"items","projection":"default","columns":["id"],"tenant":{"mode":"fixed"}}]"#.to_owned(),
    );
    set_projection_config(&mut cfg);
    cfg.validate()
        .expect("CDC config with all fields should be valid");
}

#[test]
fn cdc_enabled_without_explicit_enrollment_fails_fast() {
    let mut cfg = GatewayConfig::test_default();
    cfg.cdc_enabled = true;
    cfg.cdc_replication_url = Some("postgres://x".to_owned());
    cfg.cdc_slot_name = Some("frf_slot".to_owned());
    cfg.cdc_publication_name = Some("frf_pub".to_owned());
    cfg.cdc_tenant_id = Some(uuid::Uuid::nil());
    cfg.cdc_channel_path = Some("entities".to_owned());
    cfg.cdc_source_epoch = Some("epoch".to_owned());
    let error = cfg.validate().expect_err("missing enrollment must fail");
    assert!(error.to_string().contains("CDC_ENROLLMENTS_JSON"));
}

#[test]
fn cdc_enabled_without_durable_projection_fails_fast() {
    let mut cfg = GatewayConfig::test_default();
    cfg.cdc_enabled = true;
    cfg.cdc_replication_url = Some("postgres://x".to_owned());
    cfg.cdc_slot_name = Some("frf_slot".to_owned());
    cfg.cdc_publication_name = Some("frf_pub".to_owned());
    cfg.cdc_tenant_id = Some(uuid::Uuid::nil());
    cfg.cdc_channel_path = Some("entities".to_owned());
    cfg.cdc_source_epoch = Some("epoch".to_owned());
    cfg.cdc_enrollments_json = Some(
        r#"[{"schema":"public","table":"items","projection":"default","columns":["id"],"tenant":{"mode":"fixed"}}]"#.to_owned(),
    );
    let error = cfg
        .validate()
        .expect_err("projection config must be required");
    assert!(error.to_string().contains("ENTITY_PROJECTION_URL"));
}

#[test]
fn cdc_enabled_with_invalid_source_epoch_fails_fast() {
    let mut cfg = GatewayConfig::test_default();
    cfg.cdc_enabled = true;
    cfg.cdc_replication_url = Some("postgres://x".to_owned());
    cfg.cdc_slot_name = Some("frf_slot".to_owned());
    cfg.cdc_publication_name = Some("frf_pub".to_owned());
    cfg.cdc_tenant_id = Some(uuid::Uuid::nil());
    cfg.cdc_channel_path = Some("entities".to_owned());
    cfg.cdc_source_epoch = Some("restore/2".to_owned());
    let error = cfg.validate().expect_err("unsafe source epoch must fail");
    assert!(error.to_string().contains("CDC_SOURCE_EPOCH"));
}

#[test]
fn federation_enabled_without_tenant_fails_fast() {
    let mut cfg = GatewayConfig::test_default();
    cfg.federation_enabled = true;
    cfg.federation_tenant_id = None;
    let err = cfg.validate().expect_err("expected validation to fail");
    assert!(err.to_string().contains("FEDERATION_TENANT_ID"));
}

#[test]
fn federation_enabled_without_channel_fails_fast() {
    // Tenant is set (passes the tenant guard) but channel is not — without this
    // guard, ingested events would land on a random per-boot channel.
    let mut cfg = GatewayConfig::test_default();
    cfg.federation_enabled = true;
    cfg.federation_tenant_id = Some(uuid::Uuid::nil());
    cfg.federation_channel_id = None;
    let err = cfg.validate().expect_err("expected validation to fail");
    assert!(err.to_string().contains("FEDERATION_CHANNEL_ID"));
}

#[test]
fn atproto_pds_writer_all_set_passes_validation() {
    // All three PDS-writer vars set → the outbound writer is fully configured, so the
    // all-or-none guard is satisfied and validation succeeds.
    let mut cfg = GatewayConfig::test_default();
    cfg.atproto_pds_url = Some("https://bsky.social".to_owned());
    cfg.atproto_pds_identifier = Some("alice.bsky.social".to_owned());
    cfg.atproto_pds_app_password = Some("app-pw".to_owned());
    cfg.validate()
        .expect("fully-configured ATProto writer should pass validation");
}

#[test]
fn atproto_pds_writer_partial_fails_fast() {
    // Only URL + identifier set (no app-password) → half-configured writer would never
    // authenticate; validation must fail fast naming the missing var.
    let mut cfg = GatewayConfig::test_default();
    cfg.atproto_pds_url = Some("https://bsky.social".to_owned());
    cfg.atproto_pds_identifier = Some("alice.bsky.social".to_owned());
    cfg.atproto_pds_app_password = None;
    let err = cfg.validate().expect_err("expected validation to fail");
    assert!(
        err.to_string().contains("ATPROTO_PDS_APP_PASSWORD"),
        "error should name the missing var, got: {err}"
    );
}

#[test]
fn atproto_pds_writer_none_set_stays_inbound_only() {
    // No PDS-writer vars → inbound-only, no requirement. test_default already has them
    // all None, so validation passes with no ATProto-writer constraint triggered.
    let cfg = GatewayConfig::test_default();
    assert!(cfg.atproto_pds_url.is_none());
    assert!(cfg.atproto_pds_identifier.is_none());
    assert!(cfg.atproto_pds_app_password.is_none());
    cfg.validate()
        .expect("inbound-only config should pass validation");
}

#[test]
fn hosted_sfu_without_livekit_creds_fails_fast() {
    // Hosted SFU requires LiveKit env. This test env does not set LIVEKIT_*,
    // so validation must fail fast rather than silently disabling signaling.
    // (Guard: skip if the env happens to have creds set, to avoid a flaky
    // false-negative in an environment that provides them.)
    if std::env::var("LIVEKIT_API_KEY").is_ok() {
        return;
    }
    let mut cfg = GatewayConfig::test_default();
    cfg.sfu_mode = SfuMode::Hosted;
    cfg.lanes.media = true;
    let err = cfg.validate().expect_err("expected validation to fail");
    assert!(err.to_string().contains("LIVEKIT_"));
}

#[test]
#[cfg(feature = "shape-facade")]
fn shape_only_rejects_event_spine_and_media_authority() {
    let mut cfg = GatewayConfig::test_default();
    cfg.profile = GatewayProfile::ShapeOnly;
    cfg.shape_electric_url = Some("http://electric:3000".to_owned());
    cfg.shape_catalog_path = Some("/run/config/shape-catalog.json".to_owned());
    cfg.iggy_connection_string = "iggy://user:pass@iggy:8090".to_owned();
    let error = cfg.validate().expect_err("shape-only must reject Iggy");
    assert!(error.to_string().contains("IGGY_CONNECTION_STRING"));

    cfg.iggy_connection_string.clear();
    cfg.lanes.media = true;
    let error = cfg.validate().expect_err("shape-only must reject media");
    assert!(error.to_string().contains("MEDIA_ENABLED"));
}

#[test]
#[cfg(feature = "shape-facade")]
fn shape_only_requires_complete_shape_authority() {
    let mut cfg = GatewayConfig::test_default();
    cfg.profile = GatewayProfile::ShapeOnly;
    cfg.iggy_connection_string.clear();
    cfg.shape_electric_url = Some("http://electric:3000".to_owned());
    let error = cfg
        .validate()
        .expect_err("shape-only must require a catalog with Electric");
    assert!(error.to_string().contains("must be set together"));
}

#[test]
fn full_profile_rejects_shape_only_endpoints() {
    let mut cfg = GatewayConfig::test_default();
    cfg.shape_electric_url = Some("http://electric:3000".to_owned());
    cfg.shape_catalog_path = Some("/run/config/shape-catalog.json".to_owned());
    let error = cfg
        .validate()
        .expect_err("full profile must reject the Electric facade");
    assert!(error.to_string().contains("use the shape-only profile"));
}

fn set_projection_config(cfg: &mut GatewayConfig) {
    cfg.entity_projection_url = Some("ws://surreal:8000".to_owned());
    cfg.entity_projection_username = Some("root".to_owned());
    cfg.entity_projection_password = Some("secret".to_owned());
    cfg.entity_projection_namespace = Some("frf".to_owned());
    cfg.entity_projection_database = Some("projection".to_owned());
}
