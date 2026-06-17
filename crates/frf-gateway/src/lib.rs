mod routes;
mod ws;

use axum::{Router, routing::get};

/// Builds the production Axum router. Exported so integration tests can
/// construct the app without spawning a real TCP listener.
pub fn build_router() -> Router {
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/ws", get(ws::ws_echo))
}
