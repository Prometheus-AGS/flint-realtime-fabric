use std::collections::HashMap;

use frf_domain::AgentEvent;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use tokio::sync::mpsc;

/// Message type for the `PublisherActor`.
#[allow(clippy::module_name_repetitions)]
pub enum PublisherMessage {
    /// Publish an event to all subscribers of the event's tenant.
    Publish(AgentEvent),
    /// Register a subscriber channel for a tenant.
    Subscribe {
        tenant_id: String,
        reply: RpcReplyPort<mpsc::Receiver<AgentEvent>>,
    },
}

/// Per-tenant list of subscriber senders.
type SubscriberMap = HashMap<String, Vec<mpsc::Sender<AgentEvent>>>;

/// Actor that holds subscriber map and fans out published events.
pub struct PublisherActor;

#[ractor::async_trait]
impl Actor for PublisherActor {
    type Msg = PublisherMessage;
    type State = SubscriberMap;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(HashMap::new())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PublisherMessage::Publish(event) => {
                let tenant = event.tenant_id.to_string();
                if let Some(senders) = state.get_mut(&tenant) {
                    senders.retain(|tx| tx.try_send(event.clone()).is_ok());
                }
            }
            PublisherMessage::Subscribe { tenant_id, reply } => {
                let (tx, rx) = mpsc::channel(256);
                state.entry(tenant_id).or_default().push(tx);
                let _ = reply.send(rx);
            }
        }
        Ok(())
    }
}
