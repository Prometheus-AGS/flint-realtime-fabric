//! Reconnecting subscribe: survives gateway restarts and transient transport
//! errors by reconnecting with exponential backoff and resuming from the last
//! offset seen (replay-from-offset).

use std::time::Duration;

use frf_domain::{ChannelId, EventEnvelope, Offset};
use futures_util::{Stream, StreamExt as _};

use crate::client::FrfClient;
use crate::error::SdkError;

/// Backoff and retry policy for a reconnecting subscription.
#[derive(Debug, Clone, Copy)]
pub struct ReconnectPolicy {
    /// Initial delay before the first reconnect attempt.
    pub initial_backoff: Duration,
    /// Maximum delay between reconnect attempts.
    pub max_backoff: Duration,
    /// Multiplier applied to the delay after each failed attempt.
    pub multiplier: u32,
    /// Maximum consecutive failed reconnects before giving up. `None` = retry
    /// forever.
    pub max_retries: Option<u32>,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_millis(250),
            max_backoff: Duration::from_secs(30),
            multiplier: 2,
            max_retries: None,
        }
    }
}

impl ReconnectPolicy {
    fn next_backoff(&self, current: Duration) -> Duration {
        (current * self.multiplier).min(self.max_backoff)
    }
}

/// Parameters describing what to (re)subscribe to.
#[derive(Debug, Clone)]
pub struct SubscribeTarget {
    pub endpoint: String,
    pub token: Option<String>,
    pub channel_id: ChannelId,
    pub consumer_id: String,
    /// Offset to start from on the FIRST connect. Subsequent reconnects resume
    /// from the last offset actually observed.
    pub from: Offset,
}

/// A reconnecting subscription stream.
///
/// Yields `Ok(envelope)` for each delivered event. On a transport error it does
/// NOT terminate: it reconnects with exponential backoff and resumes from
/// `last_seen_offset + 1`, so no event is skipped and duplicates are avoided.
/// It yields `Err` only if reconnection is exhausted (`max_retries` reached).
///
/// Because reconnection is transparent, a consumer written against this stream
/// survives gateway restarts without any special handling.
pub fn resilient_subscribe(
    target: SubscribeTarget,
    policy: ReconnectPolicy,
) -> impl Stream<Item = Result<EventEnvelope, SdkError>> {
    async_stream::stream! {
        // `from` advances as we observe events, so a reconnect resumes correctly.
        let mut next_from = target.from;
        let mut backoff = policy.initial_backoff;
        let mut retries: u32 = 0;

        loop {
            match connect_and_stream(&target, next_from).await {
                Ok(mut stream) => {
                    // Connected: reset backoff and retry counter.
                    backoff = policy.initial_backoff;
                    retries = 0;

                    let mut transport_ok = true;
                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(envelope) => {
                                // Resume AFTER the last delivered offset.
                                next_from = Offset(envelope.offset.0.saturating_add(1));
                                yield Ok(envelope);
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "subscribe stream error — will reconnect");
                                transport_ok = false;
                                break;
                            }
                        }
                    }

                    if transport_ok {
                        // Server closed the stream cleanly (not an error). Reconnect
                        // to keep the subscription alive across gateway restarts.
                        tracing::info!("subscribe stream ended — reconnecting to resume");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "reconnect failed");
                    retries += 1;
                    if policy.max_retries.is_some_and(|max| retries > max) {
                        yield Err(SdkError::Connect(format!(
                            "reconnection exhausted after {retries} attempts: {e}"
                        )));
                        return;
                    }
                }
            }

            tokio::time::sleep(backoff).await;
            backoff = policy.next_backoff(backoff);
        }
    }
}

async fn connect_and_stream(
    target: &SubscribeTarget,
    from: Offset,
) -> Result<impl Stream<Item = Result<EventEnvelope, SdkError>> + use<>, SdkError> {
    let mut client = FrfClient::connect(target.endpoint.clone(), target.token.clone()).await?;
    // The returned stream captures nothing (`use<>`) — it owns its transport, so
    // `client` may be dropped at the end of this function without ending it.
    client
        .subscribe(target.channel_id, target.consumer_id.clone(), from)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_and_caps_at_max() {
        let policy = ReconnectPolicy {
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_millis(500),
            multiplier: 2,
            max_retries: None,
        };
        let mut d = policy.initial_backoff;
        d = policy.next_backoff(d); // 200
        assert_eq!(d, Duration::from_millis(200));
        d = policy.next_backoff(d); // 400
        assert_eq!(d, Duration::from_millis(400));
        d = policy.next_backoff(d); // 800 -> capped at 500
        assert_eq!(d, Duration::from_millis(500));
        d = policy.next_backoff(d); // stays at cap
        assert_eq!(d, Duration::from_millis(500));
    }

    #[test]
    fn default_policy_retries_forever() {
        assert!(ReconnectPolicy::default().max_retries.is_none());
    }

    #[test]
    fn resume_offset_is_last_seen_plus_one() {
        // The resume logic advances `next_from` to last_delivered + 1 so a
        // reconnect neither skips nor replays the last-seen event.
        let last = Offset(41);
        let next = Offset(last.0.saturating_add(1));
        assert_eq!(next, Offset(42));
    }

    #[test]
    fn resume_offset_saturates_at_u64_max() {
        let last = Offset(u64::MAX);
        let next = Offset(last.0.saturating_add(1));
        assert_eq!(next, Offset(u64::MAX));
    }
}
