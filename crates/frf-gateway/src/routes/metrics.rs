//! Prometheus `/metrics` endpoint.
//!
//! A process-global `PrometheusHandle` is installed once at startup
//! ([`install_recorder`]); the `/metrics` handler renders its current snapshot in
//! the Prometheus text exposition format. Application code records metrics through
//! the `metrics` facade macros (`counter!`, `histogram!`), independent of this
//! module.

use std::sync::OnceLock;

use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Install the global Prometheus recorder. Call once at startup, before any
/// metric is recorded. Idempotent: a second call is a no-op and returns `Ok`.
///
/// # Errors
///
/// Returns an error string if the recorder could not be installed (e.g. another
/// global metrics recorder is already set by a different subsystem).
pub fn install_recorder() -> Result<(), String> {
    if HANDLE.get().is_some() {
        return Ok(());
    }
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| format!("failed to install Prometheus recorder: {e}"))?;
    // If another thread won the race, keep the first handle.
    let _ = HANDLE.set(handle);
    Ok(())
}

/// Record the outcome of a publish request. `result` is `"ok"` or `"error"`.
pub fn record_publish(result: &'static str) {
    metrics::counter!("frf_publish_total", "result" => result).increment(1);
}

/// Record that one event was delivered to a subscriber's fan-out stream.
pub fn record_delivery() {
    metrics::counter!("frf_events_delivered_total").increment(1);
}

/// Record that a subscription stream was opened.
pub fn record_subscribe_opened() {
    metrics::counter!("frf_subscriptions_opened_total").increment(1);
}

/// `GET /metrics` — render the current metrics in Prometheus text format.
///
/// Returns 503 if the recorder was never installed (metrics disabled).
pub async fn metrics() -> impl IntoResponse {
    match HANDLE.get() {
        Some(handle) => (
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            handle.render(),
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "metrics recorder not installed",
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_metrics_appear_in_the_scrape_output() {
        // Install once (idempotent) and record through the public helpers, then
        // scrape the rendered exposition and assert the series appear. This is
        // the /metrics endpoint's data path without an HTTP server.
        install_recorder().expect("install recorder");

        record_publish("ok");
        record_publish("error");
        record_delivery();
        record_subscribe_opened();

        let handle = HANDLE.get().expect("handle installed");
        let rendered = handle.render();

        assert!(
            rendered.contains("frf_publish_total"),
            "expected frf_publish_total in scrape output, got:\n{rendered}"
        );
        assert!(rendered.contains("frf_events_delivered_total"));
        assert!(rendered.contains("frf_subscriptions_opened_total"));
    }
}
