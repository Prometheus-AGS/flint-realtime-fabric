//! The [`ShapeFacade`] adapter over an Electric upstream.
//!
//! This type is a pure transport: it receives an [`AuthorizedShapeRequest`] that the gateway
//! has already resolved and authorized, builds the Electric query from it, and maps the
//! response back. It performs **no** authorization of its own and holds no authz dependency —
//! composition of (shape × authz) happens only in `frf-gateway`, mirroring ADR-007's
//! media-path split.

use async_trait::async_trait;
use frf_ports::{AuthorizedShapeRequest, Cursor, PortError, ShapeFacade, ShapeResponse};

use crate::client::ElectricUpstream;

/// A [`ShapeFacade`] backed by an Electric server.
pub struct ElectricShapeFacade<U: ElectricUpstream> {
    upstream: U,
}

impl<U: ElectricUpstream> ElectricShapeFacade<U> {
    /// Wrap an upstream exchange.
    pub const fn new(upstream: U) -> Self {
        Self { upstream }
    }

    /// Build Electric's query parameters from a server-derived request.
    ///
    /// Only `handle`/`offset` come from the client, and only as opaque echoes of values
    /// Electric itself issued — the table, columns and row filter are always the policy's.
    fn query_for(request: &AuthorizedShapeRequest) -> Vec<(String, String)> {
        let mut query = vec![
            ("table".to_owned(), request.table.clone()),
            ("columns".to_owned(), request.columns.join(",")),
        ];
        if let Some(where_clause) = &request.where_clause {
            query.push(("where".to_owned(), where_clause.clone()));
        }
        match &request.cursor {
            Cursor::Initial => {
                // Electric's own sentinel for "no prior state".
                query.push(("offset".to_owned(), "-1".to_owned()));
            }
            Cursor::Now => {
                query.push(("offset".to_owned(), "now".to_owned()));
            }
            Cursor::Resume { handle, offset } => {
                query.push(("handle".to_owned(), handle.clone()));
                query.push(("offset".to_owned(), offset.clone()));
            }
        }
        if let Some(live) = &request.protocol.live {
            query.push(("live".to_owned(), live.clone()));
        }
        if let Some(cursor) = &request.protocol.cursor {
            query.push(("cursor".to_owned(), cursor.clone()));
        }
        query
    }
}

#[async_trait]
impl<U: ElectricUpstream> ShapeFacade for ElectricShapeFacade<U> {
    #[tracing::instrument(
        name = "shape_facade::fetch",
        skip(self, request),
        fields(table = %request.table, columns = request.columns.len())
    )]
    async fn fetch(&self, request: &AuthorizedShapeRequest) -> Result<ShapeResponse, PortError> {
        let query = Self::query_for(request);
        self.upstream
            .get_shape(&query, request.protocol.if_none_match.as_deref())
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ShapeError;
    use frf_ports::{ShapeHeader, ShapeProtocol};
    use std::sync::Mutex;

    /// Records the query it was asked to send, so request construction is assertable
    /// without a live Electric server.
    struct SpyUpstream {
        seen: Mutex<Vec<(String, String)>>,
        seen_etag: Mutex<Option<String>>,
        response: Mutex<Option<ShapeResponse>>,
    }

    #[async_trait]
    impl ElectricUpstream for SpyUpstream {
        async fn get_shape(
            &self,
            query: &[(String, String)],
            if_none_match: Option<&str>,
        ) -> Result<ShapeResponse, ShapeError> {
            *self.seen.lock().expect("lock") = query.to_vec();
            *self.seen_etag.lock().expect("etag lock") = if_none_match.map(str::to_owned);
            self.response
                .lock()
                .expect("response lock")
                .take()
                .ok_or_else(|| ShapeError::Upstream("test response already consumed".to_owned()))
        }
    }

    fn upstream() -> SpyUpstream {
        SpyUpstream {
            seen: Mutex::new(Vec::new()),
            seen_etag: Mutex::new(None),
            response: Mutex::new(Some(ShapeResponse::new(
                200,
                vec![ShapeHeader::new("electric-handle", b"h-1")],
                b"[]".to_vec(),
            ))),
        }
    }

    fn request(cursor: Cursor) -> AuthorizedShapeRequest {
        AuthorizedShapeRequest {
            table: "prior_auth_request".to_owned(),
            columns: vec!["id".to_owned(), "status".to_owned()],
            where_clause: Some("practice_id = 'p-1'".to_owned()),
            cursor,
            protocol: ShapeProtocol::default(),
        }
    }

    fn value<'a>(seen: &'a [(String, String)], key: &str) -> Option<&'a str> {
        seen.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    #[tokio::test]
    async fn initial_cursor_sends_electrics_no_prior_state_sentinel() {
        let facade = ElectricShapeFacade::new(upstream());
        facade.fetch(&request(Cursor::Initial)).await.expect("ok");
        let seen = facade.upstream.seen.lock().expect("lock").clone();

        assert_eq!(value(&seen, "offset"), Some("-1"));
        assert_eq!(value(&seen, "handle"), None, "no handle on a cold start");
        assert_eq!(value(&seen, "table"), Some("prior_auth_request"));
        assert_eq!(value(&seen, "columns"), Some("id,status"));
        assert_eq!(value(&seen, "where"), Some("practice_id = 'p-1'"));
    }

    #[tokio::test]
    async fn resume_cursor_echoes_the_handle_and_offset_upstream() {
        let facade = ElectricShapeFacade::new(upstream());
        facade
            .fetch(&request(Cursor::Resume {
                handle: "h-9".to_owned(),
                offset: "17".to_owned(),
            }))
            .await
            .expect("ok");
        let seen = facade.upstream.seen.lock().expect("lock").clone();

        assert_eq!(value(&seen, "handle"), Some("h-9"));
        assert_eq!(value(&seen, "offset"), Some("17"));
    }

    #[tokio::test]
    async fn now_cursor_skips_history_without_inventing_a_handle() {
        let facade = ElectricShapeFacade::new(upstream());
        facade.fetch(&request(Cursor::Now)).await.expect("ok");
        let seen = facade.upstream.seen.lock().expect("lock").clone();

        assert_eq!(value(&seen, "offset"), Some("now"));
        assert_eq!(value(&seen, "handle"), None);
    }

    #[tokio::test]
    async fn live_cursor_etag_status_headers_and_control_body_are_preserved()
    -> Result<(), PortError> {
        let upstream = SpyUpstream {
            seen: Mutex::new(Vec::new()),
            seen_etag: Mutex::new(None),
            response: Mutex::new(Some(ShapeResponse::new(
                409,
                vec![
                    ShapeHeader::new("electric-handle", b"h-2"),
                    ShapeHeader::new("electric-schema", br#"{"id":"uuid"}"#),
                    ShapeHeader::new("etag", b"shape:2"),
                ],
                br#"[{"headers":{"control":"must-refetch"}}]"#.to_vec(),
            ))),
        };
        let facade = ElectricShapeFacade::new(upstream);
        let mut request = request(Cursor::Initial);
        request.protocol = ShapeProtocol {
            live: Some("true".to_owned()),
            cursor: Some("1720000000000".to_owned()),
            if_none_match: Some("shape:1".to_owned()),
        };
        let response = facade.fetch(&request).await.expect("ok");
        let seen = facade.upstream.seen.lock().expect("lock").clone();

        assert_eq!(value(&seen, "live"), Some("true"));
        assert_eq!(value(&seen, "cursor"), Some("1720000000000"));
        assert_eq!(
            facade
                .upstream
                .seen_etag
                .lock()
                .expect("etag lock")
                .as_deref(),
            Some("shape:1")
        );
        assert_eq!(response.status, 409);
        assert_eq!(response.header("electric-handle"), Some(b"h-2".as_slice()));
        assert_eq!(response.header("etag"), Some(b"shape:2".as_slice()));
        let frames = futures_util::StreamExt::collect::<Vec<_>>(response.body).await;
        let body = frames.into_iter().collect::<Result<Vec<_>, _>>()?.concat();
        assert_eq!(body, br#"[{"headers":{"control":"must-refetch"}}]"#);
        Ok::<(), PortError>(())
    }
}
