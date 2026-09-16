use super::{GatewayConfig, GatewayProfile, SfuMode};

pub(super) fn parse_env_number<T>(
    name: &str,
    value: Option<String>,
    default: T,
) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value.map_or(Ok(default), |value| {
        value
            .parse()
            .map_err(|error| anyhow::anyhow!("{name} must be a valid number: {error}"))
    })
}

impl GatewayConfig {
    /// Validate semantic configuration and profile authority before binding.
    ///
    /// # Errors
    ///
    /// Returns the first missing dependency or cross-profile endpoint.
    pub fn validate(&self) -> anyhow::Result<()> {
        self.validate_profile_boundary()?;
        self.validate_shape_boundary()?;
        self.validate_media()?;
        self.validate_cdc()?;
        self.validate_federation()?;
        self.validate_atproto_writer()?;

        #[cfg(not(feature = "dev-endpoints"))]
        anyhow::ensure!(
            self.jwt_issuer.is_some(),
            "JWT_ISSUER must be set in production so the token issuer (iss) is validated \
             and only your IdP's tokens are accepted. (This check is relaxed to a warning \
             only in `dev-endpoints` builds.)"
        );
        Ok(())
    }

    fn validate_profile_boundary(&self) -> anyhow::Result<()> {
        if self.profile == GatewayProfile::ShapeOnly {
            anyhow::ensure!(
                cfg!(feature = "shape-facade"),
                "GATEWAY_PROFILE=shape-only requires the shape-facade build feature"
            );
            anyhow::ensure!(
                self.grpc_port.is_none(),
                "GATEWAY_PROFILE=shape-only requires GRPC_PORT=0"
            );
            anyhow::ensure!(
                !self.cdc_enabled
                    && !self.federation_enabled
                    && !self.lanes.media
                    && !self.lanes.agent
                    && !self.lanes.admin,
                "GATEWAY_PROFILE=shape-only requires CDC_ENABLED=false, \
                 FEDERATION_ENABLED=false, MEDIA_ENABLED=false, AGENT_ENABLED=false, \
                 and ADMIN_ENABLED=false"
            );
            anyhow::ensure!(
                self.iggy_connection_string.is_empty(),
                "GATEWAY_PROFILE=shape-only rejects IGGY_CONNECTION_STRING because the event \
                 spine is outside this profile's authority"
            );
        } else {
            anyhow::ensure!(
                !self.iggy_connection_string.trim().is_empty(),
                "GATEWAY_PROFILE=full requires IGGY_CONNECTION_STRING"
            );
        }
        Ok(())
    }

    fn validate_shape_boundary(&self) -> anyhow::Result<()> {
        match (&self.shape_electric_url, &self.shape_catalog_path) {
            (Some(_), Some(_)) if self.profile == GatewayProfile::ShapeOnly => Ok(()),
            (None, None) if self.profile == GatewayProfile::ShapeOnly => anyhow::bail!(
                "GATEWAY_PROFILE=shape-only requires SHAPE_ELECTRIC_URL and SHAPE_CATALOG_PATH"
            ),
            (None, None) => Ok(()),
            (Some(_), Some(_)) => anyhow::bail!(
                "GATEWAY_PROFILE=full rejects SHAPE_ELECTRIC_URL and SHAPE_CATALOG_PATH; \
                 use the shape-only profile"
            ),
            _ => anyhow::bail!("SHAPE_ELECTRIC_URL and SHAPE_CATALOG_PATH must be set together"),
        }
    }

    fn validate_media(&self) -> anyhow::Result<()> {
        if self.lanes.media && self.sfu_mode == SfuMode::Hosted {
            for var in [
                "LIVEKIT_API_KEY",
                "LIVEKIT_API_SECRET",
                "LIVEKIT_SERVER_URL",
            ] {
                anyhow::ensure!(
                    std::env::var(var).is_ok_and(|v| !v.trim().is_empty()),
                    "SFU_MODE=hosted requires {var} to be set to a non-empty value \
                     (LiveKit signaling would otherwise be silently disabled)"
                );
            }
        }
        Ok(())
    }

    fn validate_cdc(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.entity_watch_retention_seconds >= 86_400,
            "ENTITY_WATCH_RETENTION_SECONDS must be at least 86400"
        );
        if self.cdc_enabled {
            anyhow::ensure!(
                self.cdc_replication_url
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "CDC_ENABLED=true requires CDC_REPLICATION_URL"
            );
            anyhow::ensure!(
                self.cdc_slot_name
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "CDC_ENABLED=true requires CDC_SLOT_NAME"
            );
            anyhow::ensure!(
                self.cdc_publication_name
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "CDC_ENABLED=true requires CDC_PUBLICATION_NAME"
            );
            anyhow::ensure!(
                self.cdc_tenant_id.is_some(),
                "CDC_ENABLED=true requires CDC_TENANT_ID"
            );
            anyhow::ensure!(
                self.cdc_channel_path
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "CDC_ENABLED=true requires CDC_CHANNEL_PATH"
            );
            let source_epoch = self
                .cdc_source_epoch
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("CDC_ENABLED=true requires CDC_SOURCE_EPOCH"))?;
            frf_postgres_cdc::CdcConfig::validate_source_epoch(source_epoch)
                .map_err(|error| anyhow::anyhow!("CDC_SOURCE_EPOCH is invalid: {error}"))?;
            let enrollment_json = self
                .cdc_enrollments_json
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("CDC_ENABLED=true requires CDC_ENROLLMENTS_JSON"))?;
            let enrollments = frf_postgres_cdc::CdcConfig::parse_enrollments(enrollment_json)
                .map_err(|error| anyhow::anyhow!(error))?;
            anyhow::ensure!(
                !enrollments.is_empty(),
                "CDC_ENROLLMENTS_JSON must contain at least one enrollment"
            );
            for (name, value) in [
                ("ENTITY_PROJECTION_URL", &self.entity_projection_url),
                (
                    "ENTITY_PROJECTION_USERNAME",
                    &self.entity_projection_username,
                ),
                (
                    "ENTITY_PROJECTION_PASSWORD",
                    &self.entity_projection_password,
                ),
                (
                    "ENTITY_PROJECTION_NAMESPACE",
                    &self.entity_projection_namespace,
                ),
                (
                    "ENTITY_PROJECTION_DATABASE",
                    &self.entity_projection_database,
                ),
            ] {
                anyhow::ensure!(
                    value.as_ref().is_some_and(|value| !value.trim().is_empty()),
                    "CDC_ENABLED=true requires {name}"
                );
            }
            anyhow::ensure!(
                self.entity_watch_checkpoint_key
                    .as_ref()
                    .is_some_and(|value| value.len() >= 32),
                "CDC_ENABLED=true requires ENTITY_WATCH_CHECKPOINT_KEY with at least 32 bytes"
            );
            anyhow::ensure!(
                self.entity_watch_checkpoint_generation > 0,
                "ENTITY_WATCH_CHECKPOINT_GENERATION must be greater than zero"
            );
            anyhow::ensure!(
                self.entity_watch_buffer_capacity > 0,
                "ENTITY_WATCH_BUFFER_CAPACITY must be greater than zero"
            );
        }
        Ok(())
    }

    fn validate_federation(&self) -> anyhow::Result<()> {
        if self.federation_enabled {
            anyhow::ensure!(
                self.federation_tenant_id.is_some(),
                "FEDERATION_ENABLED=true requires FEDERATION_TENANT_ID"
            );
            anyhow::ensure!(
                self.federation_channel_id.is_some(),
                "FEDERATION_ENABLED=true requires FEDERATION_CHANNEL_ID"
            );
        }
        Ok(())
    }

    fn validate_atproto_writer(&self) -> anyhow::Result<()> {
        let fields = [
            ("ATPROTO_PDS_URL", self.atproto_pds_url.as_ref()),
            (
                "ATPROTO_PDS_IDENTIFIER",
                self.atproto_pds_identifier.as_ref(),
            ),
            (
                "ATPROTO_PDS_APP_PASSWORD",
                self.atproto_pds_app_password.as_ref(),
            ),
        ];
        if fields.iter().any(|(_, value)| value.is_some()) {
            for (name, value) in fields {
                anyhow::ensure!(
                    value.is_some(),
                    "ATProto outbound writer is partially configured: {name} must also be set"
                );
            }
        }
        Ok(())
    }
}
