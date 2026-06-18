use frf_ports::PortError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("JWKS fetch failed: {0}")]
    JwksFetch(String),
    #[error("JWT verification failed: {0}")]
    Verification(String),
    #[error("missing required claim: {0}")]
    MissingClaim(String),
    #[error("no matching JWK for kid '{0}'")]
    UnknownKid(String),
}

impl From<IdentityError> for PortError {
    fn from(err: IdentityError) -> Self {
        match err {
            IdentityError::Verification(msg) | IdentityError::MissingClaim(msg) => {
                PortError::PermissionDenied(msg)
            }
            IdentityError::JwksFetch(msg) => PortError::Transport(msg),
            IdentityError::UnknownKid(msg) => PortError::PermissionDenied(msg),
        }
    }
}
