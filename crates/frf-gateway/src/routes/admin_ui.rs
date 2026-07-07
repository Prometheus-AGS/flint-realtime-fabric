//! Serves the embedded admin UI (single-binary deploy).
//!
//! The built `admin-ui/dist` is compiled into the gateway binary via
//! `rust-embed`. Any non-API GET falls back to `index.html` so the SPA's
//! client-side routing works on deep links.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// The built admin UI assets, embedded at compile time.
///
/// The `dist/` directory must exist at build time (run the admin-ui build
/// first). If it is empty, the gateway still compiles and simply serves 404s
/// for UI routes — the API is unaffected.
#[derive(RustEmbed)]
#[folder = "../../admin-ui/dist"]
struct AdminUiAssets;

/// Serve an embedded admin-UI asset by path, falling back to `index.html`.
///
/// `GET /` and `GET /assets/*` resolve to embedded files; anything else that
/// does not match a file falls back to `index.html` (SPA client routing).
pub async fn serve_admin_ui(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match AdminUiAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => serve_index_fallback(),
    }
}

/// SPA fallback: serve `index.html` for unknown paths so client routing works.
fn serve_index_fallback() -> Response {
    match AdminUiAssets::get("index.html") {
        Some(content) => (
            [(header::CONTENT_TYPE, "text/html")],
            content.data.into_owned(),
        )
            .into_response(),
        // dist was empty at build time — the UI was not built.
        None => (
            StatusCode::NOT_FOUND,
            "admin UI not embedded (build admin-ui first)",
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn root_serves_embedded_index_html() {
        let resp = serve_admin_ui("/".parse().unwrap()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(ct.contains("html"), "expected html content-type, got {ct}");
    }

    #[tokio::test]
    async fn unknown_path_falls_back_to_index() {
        // A client-side route with no matching asset must return index.html so
        // the SPA can route it — NOT a 404.
        let resp = serve_admin_ui("/entities/deep/link".parse().unwrap()).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn asset_is_served_with_its_mime() {
        // The embedded JS asset resolves and is served as a real file.
        let assets: Vec<String> = AdminUiAssets::iter().map(|f| f.to_string()).collect();
        let js = assets.iter().find(|p| {
            std::path::Path::new(p.as_str())
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("js"))
        });
        if let Some(js_path) = js {
            let resp = serve_admin_ui(format!("/{js_path}").parse().unwrap()).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
