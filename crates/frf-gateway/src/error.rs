use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use frf_app::AppError;
use frf_ports::PortError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("authentication failed: {0}")]
    Identity(PortError),

    #[error("forbidden: {0}")]
    Authz(PortError),

    #[error("publish failed: {0}")]
    Publish(#[from] AppError),

    #[error("bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            Self::Identity(_) | Self::Publish(AppError::Identity(_)) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            Self::Authz(_) | Self::Publish(AppError::Unauthorized(_) | AppError::Forbidden(_)) => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
            Self::Publish(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };
        (status, body).into_response()
    }
}
