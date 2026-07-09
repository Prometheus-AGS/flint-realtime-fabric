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
fn cdc_enabled_with_all_fields_passes() {
    let mut cfg = GatewayConfig::test_default();
    cfg.cdc_enabled = true;
    cfg.cdc_replication_url = Some("postgres://x".to_owned());
    cfg.cdc_slot_name = Some("frf_slot".to_owned());
    cfg.cdc_publication_name = Some("frf_pub".to_owned());
    cfg.validate()
        .expect("CDC config with all fields should be valid");
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
    let err = cfg.validate().expect_err("expected validation to fail");
    assert!(err.to_string().contains("LIVEKIT_"));
}
