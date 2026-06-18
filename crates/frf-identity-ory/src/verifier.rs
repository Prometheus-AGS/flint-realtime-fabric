use async_trait::async_trait;
use frf_ports::{IdentityVerifier, PortError, VerifiedClaims};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use tracing::instrument;

use crate::{
    claims::{FrfClaims, to_verified_claims},
    error::IdentityError,
    jwks::{JwksCache, get_or_fetch, new_cache, refresh},
};

pub struct OryIdentityVerifier {
    http: reqwest::Client,
    jwks_url: String,
    jwks_cache: JwksCache,
    audience: String,
}

impl OryIdentityVerifier {
    #[must_use]
    pub fn new(jwks_url: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            jwks_url: jwks_url.into(),
            jwks_cache: new_cache(),
            audience: audience.into(),
        }
    }

    async fn decode_token(&self, token: &str, fresh: bool) -> Result<FrfClaims, IdentityError> {
        let header =
            decode_header(token).map_err(|e| IdentityError::Verification(e.to_string()))?;

        let jwks = if fresh {
            refresh(&self.http, &self.jwks_url, &self.jwks_cache).await?
        } else {
            get_or_fetch(&self.http, &self.jwks_url, &self.jwks_cache).await?
        };

        let kid = header.kid.as_deref().unwrap_or("");
        let jwk = jwks
            .find(kid)
            .ok_or_else(|| IdentityError::UnknownKid(kid.to_owned()))?;

        let decoding_key =
            DecodingKey::from_jwk(jwk).map_err(|e| IdentityError::Verification(e.to_string()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.audience]);

        let data = decode::<FrfClaims>(token, &decoding_key, &validation)
            .map_err(|e| IdentityError::Verification(e.to_string()))?;

        Ok(data.claims)
    }
}

#[async_trait]
impl IdentityVerifier for OryIdentityVerifier {
    /// Verify a raw JWT bearer token and return extracted claims.
    ///
    /// On `InvalidSignature` or `InvalidKeyFormat`, refreshes the JWKS cache
    /// and retries once to handle key rotation.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::PermissionDenied`] on invalid or expired tokens.
    /// Returns [`PortError::Transport`] if the JWKS endpoint is unreachable.
    #[instrument(name = "port::IdentityVerifier::verify", skip(self, token))]
    async fn verify(&self, token: &str) -> Result<VerifiedClaims, PortError> {
        match self.decode_token(token, false).await {
            Ok(claims) => to_verified_claims(claims).map_err(PortError::from),
            Err(IdentityError::Verification(ref msg)) => {
                let should_retry =
                    msg.contains("InvalidSignature") || msg.contains("InvalidKeyFormat");

                if should_retry {
                    let claims = self.decode_token(token, true).await?;
                    to_verified_claims(claims).map_err(PortError::from)
                } else {
                    Err(PortError::from(IdentityError::Verification(msg.clone())))
                }
            }
            Err(IdentityError::UnknownKid(kid)) => {
                let claims = self
                    .decode_token(token, true)
                    .await
                    .map_err(|_| PortError::PermissionDenied(format!("unknown kid: {kid}")))?;
                to_verified_claims(claims).map_err(PortError::from)
            }
            Err(e) => Err(PortError::from(e)),
        }
    }
}
