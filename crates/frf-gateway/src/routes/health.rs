use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
};
use serde_json::{Value, json};

use crate::AppStateArc;
use crate::config::{AuthzBackend, GatewayProfile};

/// Liveness probe: the process is up and serving HTTP. Never checks
/// dependencies — used by orchestrators to decide whether to RESTART the pod.
pub async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// Readiness probe: the gateway's configured dependencies are reachable,
/// so it is safe to route traffic here. Returns 503 with per-dependency detail
/// when any check fails — used by orchestrators to decide whether to add/remove
/// the pod from the load-balancer ROTATION (distinct from liveness).
pub async fn readyz<L, A, I, M, B, P>(
    State(state): State<AppStateArc<L, A, I, M, B, P>>,
) -> impl IntoResponse
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    let cfg = &state.config;
    let http = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
        .unwrap_or_default();

    let authorization = match cfg.authz_backend {
        AuthzBackend::VerifiedIdentity => Check {
            ok: true,
            detail: "verified identity + tenant guards; durable data uses PostgreSQL RLS"
                .to_owned(),
        },
        AuthzBackend::Keto => {
            probe_http(&http, &format!("{}/health/ready", cfg.keto_base_url)).await
        }
    };
    let jwks = probe_jwks(&http, &cfg.gateway_jwks_url).await;
    let iggy = if cfg.profile == GatewayProfile::Full {
        probe_tcp(&cfg.iggy_connection_string).await
    } else {
        disabled("outside shape-only profile")
    };
    let electric = if cfg.profile == GatewayProfile::ShapeOnly {
        match &cfg.shape_electric_url {
            Some(origin) => {
                probe_http(
                    &http,
                    &format!("{}/v1/health", origin.trim_end_matches('/')),
                )
                .await
            }
            None => Check {
                ok: false,
                detail: "SHAPE_ELECTRIC_URL is missing".to_owned(),
            },
        }
    } else {
        disabled("outside full profile")
    };
    let cdc = if cfg.cdc_enabled {
        if *state.cdc_readiness.borrow() {
            Check {
                ok: true,
                detail: "logical replication stream active".to_owned(),
            }
        } else {
            Check {
                ok: false,
                detail: "logical replication stream inactive".to_owned(),
            }
        }
    } else {
        disabled("CDC_ENABLED is false")
    };

    let all_ok = authorization.ok && jwks.ok && iggy.ok && electric.ok && cdc.ok;
    let body = json!({
        "status": if all_ok { "ready" } else { "not_ready" },
        "checks": {
            "authorization": check_json(&authorization),
            "jwks": check_json(&jwks),
            "iggy": check_json(&iggy),
            "electric": check_json(&electric),
            "cdc": check_json(&cdc),
        }
    });

    let status = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(body))
}

struct Check {
    ok: bool,
    detail: String,
}

fn disabled(detail: &str) -> Check {
    Check {
        ok: true,
        detail: format!("disabled: {detail}"),
    }
}

fn check_json(c: &Check) -> Value {
    json!({ "ok": c.ok, "detail": c.detail })
}

async fn probe_http(client: &reqwest::Client, url: &str) -> Check {
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => Check {
            ok: true,
            detail: format!("{} {}", resp.status().as_u16(), url),
        },
        Ok(resp) => Check {
            ok: false,
            detail: format!("unexpected status {} from {}", resp.status().as_u16(), url),
        },
        Err(e) => Check {
            ok: false,
            detail: format!("unreachable {url}: {e}"),
        },
    }
}

async fn probe_jwks(client: &reqwest::Client, url: &str) -> Check {
    let response = match client.get(url).send().await {
        Ok(response) if response.status().is_success() => response,
        Ok(response) => {
            return Check {
                ok: false,
                detail: format!(
                    "unexpected status {} from {url}",
                    response.status().as_u16()
                ),
            };
        }
        Err(error) => {
            return Check {
                ok: false,
                detail: format!("unreachable {url}: {error}"),
            };
        }
    };
    match response.json::<Value>().await {
        Ok(document)
            if document
                .get("keys")
                .and_then(Value::as_array)
                .is_some_and(|keys| !keys.is_empty()) =>
        {
            Check {
                ok: true,
                detail: format!("valid non-empty JWKS from {url}"),
            }
        }
        Ok(_) => Check {
            ok: false,
            detail: format!("empty or invalid JWKS from {url}"),
        },
        Err(error) => Check {
            ok: false,
            detail: format!("invalid JWKS response from {url}: {error}"),
        },
    }
}

/// TCP-connect to the host:port embedded in an Iggy connection string
/// (`iggy://user:pass@host:port`).
async fn probe_tcp(conn: &str) -> Check {
    let Some(host_port) = conn.rsplit('@').next().filter(|s| s.contains(':')) else {
        return Check {
            ok: false,
            detail: format!("cannot parse host:port from {conn}"),
        };
    };
    match tokio::time::timeout(PROBE_TIMEOUT, tokio::net::TcpStream::connect(host_port)).await {
        Ok(Ok(_)) => Check {
            ok: true,
            detail: format!("connected {host_port}"),
        },
        Ok(Err(e)) => Check {
            ok: false,
            detail: format!("connect failed {host_port}: {e}"),
        },
        Err(_) => Check {
            ok: false,
            detail: format!("connect timed out {host_port}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn serve_json_once(body: &'static str) -> String {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fixture");
        let address = listener.local_addr().expect("fixture address");
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept fixture request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await.expect("read request");
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write response");
        });
        format!("http://{address}/jwks")
    }

    #[tokio::test]
    async fn tcp_probe_fails_when_dependency_is_down() {
        // Port 1 on localhost is not listening — the probe must report NOT ok,
        // which is what makes /readyz return 503 when a dependency is down.
        let check = probe_tcp("iggy://user:pass@127.0.0.1:1").await;
        assert!(!check.ok, "expected probe to fail for a down dependency");
    }

    #[tokio::test]
    async fn tcp_probe_reports_unparseable_connection_string() {
        let check = probe_tcp("not-a-valid-conn-string").await;
        assert!(!check.ok);
        assert!(check.detail.contains("cannot parse"));
    }

    #[tokio::test]
    async fn http_probe_fails_for_unreachable_url() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .unwrap_or_default();
        // Reserved TEST-NET-1 address — connection will fail/timeout.
        let check = probe_http(&client, "http://192.0.2.1:4466/health/ready").await;
        assert!(!check.ok, "expected probe to fail for an unreachable URL");
    }

    #[tokio::test]
    async fn jwks_probe_rejects_empty_key_set() {
        let url = serve_json_once(r#"{"keys":[]}"#).await;
        let check = probe_jwks(&reqwest::Client::new(), &url).await;
        assert!(!check.ok);
        assert!(check.detail.contains("empty or invalid"));
    }

    #[tokio::test]
    async fn jwks_probe_accepts_non_empty_key_set() {
        let url = serve_json_once(r#"{"keys":[{"kty":"RSA"}]}"#).await;
        let check = probe_jwks(&reqwest::Client::new(), &url).await;
        assert!(check.ok, "{}", check.detail);
    }
}
