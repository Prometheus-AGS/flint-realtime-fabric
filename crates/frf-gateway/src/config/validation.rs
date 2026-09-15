use super::{GatewayConfig, GatewayProfile, SfuMode};

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
        if self.cdc_enabled {
            anyhow::ensure!(
                self.cdc_replication_url.is_some(),
                "CDC_ENABLED=true requires CDC_REPLICATION_URL"
            );
            anyhow::ensure!(
                self.cdc_slot_name.is_some(),
                "CDC_ENABLED=true requires CDC_SLOT_NAME"
            );
            anyhow::ensure!(
                self.cdc_publication_name.is_some(),
                "CDC_ENABLED=true requires CDC_PUBLICATION_NAME"
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
