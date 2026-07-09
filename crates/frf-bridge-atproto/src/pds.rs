//! `ATProto` PDS (Personal Data Server) write client.
//!
//! Implements the outbound half of the bridge: authenticate a session with
//! `com.atproto.server.createSession` (identifier + app-password), then write records with
//! `com.atproto.repo.createRecord`. The access JWT is cached and lazily (re-)created —
//! a write that fails auth clears the session so the next call re-authenticates.

use tokio::sync::Mutex;

use crate::error::AtProtoBridgeError;

/// Configuration for authenticated PDS writes.
#[derive(Debug, Clone)]
pub struct PdsConfig {
    /// PDS base URL, e.g. `https://bsky.social`.
    pub service_url: String,
    /// Account identifier (handle or DID), e.g. `alice.bsky.social`.
    pub identifier: String,
    /// App password (NOT the main account password) — from a secret manager.
    pub app_password: String,
}

/// A cached authenticated session.
#[derive(Debug, Clone)]
struct Session {
    access_jwt: String,
    did: String,
}

/// Authenticated client for writing records to a PDS.
pub struct PdsClient {
    http: reqwest::Client,
    config: PdsConfig,
    session: Mutex<Option<Session>>,
}

impl PdsClient {
    #[must_use]
    pub fn new(config: PdsConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
            session: Mutex::new(None),
        }
    }

    /// Return a valid session, creating one if none is cached.
    async fn session(&self) -> Result<Session, AtProtoBridgeError> {
        let mut guard = self.session.lock().await;
        if let Some(s) = guard.as_ref() {
            return Ok(s.clone());
        }
        let fresh = self.create_session().await?;
        *guard = Some(fresh.clone());
        Ok(fresh)
    }

    /// Authenticate with `com.atproto.server.createSession` and return the session.
    async fn create_session(&self) -> Result<Session, AtProtoBridgeError> {
        let url = format!(
            "{}/xrpc/com.atproto.server.createSession",
            self.config.service_url
        );
        let body = serde_json::json!({
            "identifier": self.config.identifier,
            "password": self.config.app_password,
        });

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AtProtoBridgeError::Write(format!("createSession request: {e}")))?
            .error_for_status()
            .map_err(|e| AtProtoBridgeError::Write(format!("createSession rejected: {e}")))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AtProtoBridgeError::Write(format!("createSession body: {e}")))?;

        let access_jwt = json
            .get("accessJwt")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| AtProtoBridgeError::Write("createSession: no accessJwt".to_owned()))?
            .to_owned();
        let did = json
            .get("did")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| AtProtoBridgeError::Write("createSession: no did".to_owned()))?
            .to_owned();

        Ok(Session { access_jwt, did })
    }

    /// Write a record to the account's repo via `com.atproto.repo.createRecord`.
    ///
    /// `collection` is the lexicon type (e.g. `app.bsky.feed.post`); `record` is the record
    /// body. On an auth failure the cached session is dropped so the next call re-auths.
    ///
    /// # Errors
    ///
    /// [`AtProtoBridgeError::Write`] if authentication or the write request fails.
    pub async fn create_record(
        &self,
        collection: &str,
        record: serde_json::Value,
    ) -> Result<(), AtProtoBridgeError> {
        let session = self.session().await?;
        let url = format!(
            "{}/xrpc/com.atproto.repo.createRecord",
            self.config.service_url
        );
        let body = serde_json::json!({
            "repo": session.did,
            "collection": collection,
            "record": record,
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&session.access_jwt)
            .json(&body)
            .send()
            .await
            .map_err(|e| AtProtoBridgeError::Write(format!("createRecord request: {e}")))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            // The cached session is stale — drop it so the next call re-authenticates.
            *self.session.lock().await = None;
            return Err(AtProtoBridgeError::Write(
                "createRecord: session expired (cleared; retry to re-auth)".to_owned(),
            ));
        }

        resp.error_for_status()
            .map_err(|e| AtProtoBridgeError::Write(format!("createRecord rejected: {e}")))?;
        Ok(())
    }
}
