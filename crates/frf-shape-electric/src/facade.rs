//! The [`ShapeFacade`] adapter over an Electric upstream.
//!
//! This type is a pure transport: it receives an [`AuthorizedShapeRequest`] that the gateway
//! has already resolved and authorized, builds the Electric query from it, and maps the
//! response back. It performs **no** authorization of its own and holds no authz dependency —
//! composition of (shape × authz) happens only in `frf-gateway`, mirroring ADR-007's
//! media-path split.

use async_trait::async_trait;
use frf_ports::{AuthorizedShapeRequest, Cursor, PortError, ShapeChunk, ShapeFacade};

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
            ("where".to_owned(), request.where_clause.clone()),
        ];
        match &request.cursor {
            Cursor::Initial => {
                // Electric's own sentinel for "no prior state".
                query.push(("offset".to_owned(), "-1".to_owned()));
            }
            Cursor::Resume { handle, offset } => {
                query.push(("handle".to_owned(), handle.clone()));
                query.push(("offset".to_owned(), offset.clone()));
            }
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
    async fn fetch(&self, request: &AuthorizedShapeRequest) -> Result<ShapeChunk, PortError> {
        let query = Self::query_for(request);
        let response = self.upstream.get_shape(&query).await?;
        Ok(ShapeChunk::new(
            response.handle,
            response.next_offset,
            response.body,
            response.must_refetch,
            response.up_to_date,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::UpstreamResponse;
    use crate::error::ShapeError;
    use std::sync::Mutex;

    /// Records the query it was asked to send, so request construction is assertable
    /// without a live Electric server.
    #[derive(Default)]
    struct SpyUpstream {
        seen: Mutex<Vec<(String, String)>>,
        refetch: bool,
        up_to_date: bool,
    }

    #[async_trait]
    impl ElectricUpstream for SpyUpstream {
        async fn get_shape(
            &self,
            query: &[(String, String)],
        ) -> Result<UpstreamResponse, ShapeError> {
            *self.seen.lock().expect("lock") = query.to_vec();
            Ok(UpstreamResponse {
                handle: "h-1".to_owned(),
                next_offset: "42".to_owned(),
                up_to_date: self.up_to_date,
                must_refetch: self.refetch,
                body: b"[]".to_vec(),
            })
        }
    }

    fn request(cursor: Cursor) -> AuthorizedShapeRequest {
        AuthorizedShapeRequest {
            table: "prior_auth_request".to_owned(),
            columns: vec!["id".to_owned(), "status".to_owned()],
            where_clause: "practice_id = 'p-1'".to_owned(),
            cursor,
        }
    }

    fn value<'a>(seen: &'a [(String, String)], key: &str) -> Option<&'a str> {
        seen.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    #[tokio::test]
    async fn initial_cursor_sends_electrics_no_prior_state_sentinel() {
        let facade = ElectricShapeFacade::new(SpyUpstream::default());
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
        let facade = ElectricShapeFacade::new(SpyUpstream::default());
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
    async fn refetch_and_snapshot_signals_are_surfaced_to_the_client() {
        // The client must be able to rebuild the replica generation rather than merge a new
        // snapshot into stale rows, so these protocol signals cannot be swallowed.
        let facade = ElectricShapeFacade::new(SpyUpstream {
            refetch: true,
            up_to_date: true,
            ..SpyUpstream::default()
        });
        let chunk = facade.fetch(&request(Cursor::Initial)).await.expect("ok");

        assert!(chunk.must_refetch);
        assert!(chunk.snapshot_complete);
        assert_eq!(chunk.handle, "h-1");
        assert_eq!(chunk.next_offset, "42");
    }
}
