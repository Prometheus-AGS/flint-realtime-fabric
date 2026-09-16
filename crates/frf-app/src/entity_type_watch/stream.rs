use std::sync::Arc;
use std::time::Duration;

use frf_ports::{AuthzProvider, IdentityVerifier, PortError, RelationTuple};
use futures_util::{StreamExt as _, stream};
use tokio::sync::{mpsc, watch};
use tokio::time::Instant;

use super::{
    EntityTypeWatchFrame, EntityTypeWatchRequest, EntityTypeWatchStream, ResnapshotReason,
    object_view_tuple,
};
use crate::watch_checkpoint::{CheckpointCodec, CheckpointScope};

pub(super) struct WatchProducer<A, I> {
    authz: Arc<A>,
    identity: Arc<I>,
    checkpoints: CheckpointCodec,
    request: EntityTypeWatchRequest,
    subject: String,
    subscribe_tuple: RelationTuple,
    source_epoch: String,
    deadline: Instant,
    recheck: Duration,
}

impl<A, I> WatchProducer<A, I> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        authz: Arc<A>,
        identity: Arc<I>,
        checkpoints: CheckpointCodec,
        request: EntityTypeWatchRequest,
        subject: String,
        subscribe_tuple: RelationTuple,
        source_epoch: String,
        deadline: Instant,
        recheck: Duration,
    ) -> Self {
        Self {
            authz,
            identity,
            checkpoints,
            request,
            subject,
            subscribe_tuple,
            source_epoch,
            deadline,
            recheck,
        }
    }
}

#[derive(Clone)]
enum Terminal {
    Frame(Box<EntityTypeWatchFrame>),
    Permission(String),
    Source(String),
    Serialization(String),
}

impl Terminal {
    fn preempts_buffered(&self) -> bool {
        matches!(self, Self::Frame(_) | Self::Permission(_))
    }
}

pub(super) fn spawn_bounded_stream<A, I>(
    raw: frf_ports::EntityTypeDeliveryStream,
    producer: WatchProducer<A, I>,
    initial: Vec<EntityTypeWatchFrame>,
    capacity: usize,
) -> EntityTypeWatchStream
where
    A: AuthzProvider,
    I: IdentityVerifier,
{
    let (tx, rx) = mpsc::channel(capacity);
    let (terminal_tx, terminal_rx) = watch::channel(None);
    let task = tokio::spawn(run_producer(raw, tx, terminal_tx, initial, producer));
    let state = OutputState {
        rx,
        terminal_rx,
        task: Some(task),
        terminated: false,
    };
    Box::pin(stream::unfold(state, next_output))
}

async fn run_producer<A, I>(
    mut raw: frf_ports::EntityTypeDeliveryStream,
    tx: mpsc::Sender<EntityTypeWatchFrame>,
    terminal_tx: watch::Sender<Option<Terminal>>,
    initial: Vec<EntityTypeWatchFrame>,
    producer: WatchProducer<A, I>,
) where
    A: AuthzProvider,
    I: IdentityVerifier,
{
    if !enqueue_initial(&tx, &terminal_tx, initial, &producer).await {
        return;
    }
    let mut interval = tokio::time::interval(producer.recheck);
    interval.tick().await;
    loop {
        let item = tokio::select! {
            biased;
            () = tokio::time::sleep_until(producer.deadline) => {
                terminal_tx.send_replace(Some(Terminal::Permission(
                    "entity watch token expired".to_owned(),
                )));
                return;
            }
            _ = interval.tick() => {
                let authorized = tokio::select! {
                    biased;
                    () = tokio::time::sleep_until(producer.deadline) => {
                        expire(&terminal_tx);
                        return;
                    }
                    result = subscription_authorized(&producer) => result,
                };
                if !authorized {
                    terminal_tx.send_replace(Some(Terminal::Permission(
                        "entity watch authority was revoked".to_owned(),
                    )));
                    return;
                }
                continue;
            }
            item = raw.next() => item,
        };
        let Some(item) = item else {
            terminal_tx.send_replace(Some(Terminal::Source(
                "entity watch source ended".to_owned(),
            )));
            return;
        };
        let delivery = match item {
            Ok(delivery) => delivery,
            Err(error) => {
                terminal_tx.send_replace(Some(Terminal::Source(error.to_string())));
                return;
            }
        };
        let frame = match tokio::select! {
            biased;
            () = tokio::time::sleep_until(producer.deadline) => {
                expire(&terminal_tx);
                return;
            }
            frame = delivery_frame(&producer, delivery) => frame,
        } {
            Ok(frame) => frame,
            Err(terminal) => {
                terminal_tx.send_replace(Some(terminal));
                return;
            }
        };
        let authorized = tokio::select! {
            biased;
            () = tokio::time::sleep_until(producer.deadline) => {
                expire(&terminal_tx);
                return;
            }
            result = subscription_authorized(&producer) => result,
        };
        if !authorized {
            terminal_tx.send_replace(Some(Terminal::Permission(
                "entity watch authority was revoked".to_owned(),
            )));
            return;
        }
        if Instant::now() >= producer.deadline {
            expire(&terminal_tx);
            return;
        }
        match tx.try_send(frame) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Closed(_)) => return,
            Err(mpsc::error::TrySendError::Full(_)) => {
                terminal_tx.send_replace(Some(Terminal::Frame(Box::new(
                    EntityTypeWatchFrame::Lagged {
                        checkpoint_resumable: false,
                        last_safe_checkpoint: None,
                    },
                ))));
                return;
            }
        }
    }
}

async fn subscription_authorized<A, I>(producer: &WatchProducer<A, I>) -> bool
where
    A: AuthzProvider,
    I: IdentityVerifier,
{
    producer
        .identity
        .verify(&producer.request.bearer_token)
        .await
        .is_ok()
        && producer
            .authz
            .check(&producer.subscribe_tuple)
            .await
            .unwrap_or(false)
}

async fn delivery_frame<A, I>(
    producer: &WatchProducer<A, I>,
    delivery: frf_domain::EntityTypeDelivery,
) -> Result<EntityTypeWatchFrame, Terminal>
where
    A: AuthzProvider,
{
    if delivery.mutation.source.epoch != producer.source_epoch {
        return Err(Terminal::Frame(Box::new(
            EntityTypeWatchFrame::ResnapshotRequired {
                reason: ResnapshotReason::SourceEpochChanged,
                message: "entity watch source epoch changed".to_owned(),
            },
        )));
    }
    match producer.authz.check(&producer.subscribe_tuple).await {
        Ok(true) => {}
        Ok(false) => {
            return Err(Terminal::Permission(
                "entity watch authority was revoked".to_owned(),
            ));
        }
        Err(error) => return Err(Terminal::Permission(error.to_string())),
    }
    let scope = CheckpointScope {
        subject: &producer.subject,
        tenant_id: producer.request.tenant_id,
        entity_type: &producer.request.entity_type,
        source_epoch: &producer.source_epoch,
    };
    let checkpoint = producer
        .checkpoints
        .issue(&scope, Some(delivery.broker_offset))
        .map_err(Terminal::Serialization)?;
    if delivery.mutation.tenant_id != producer.request.tenant_id
        || !producer.request.entity_type.matches(&delivery.mutation)
    {
        return Ok(EntityTypeWatchFrame::CheckpointAdvanced(checkpoint));
    }
    let tuple = object_view_tuple(
        producer.request.tenant_id,
        &producer.subject,
        &delivery.mutation.key,
    );
    match producer.authz.check(&tuple).await {
        Ok(true) => Ok(EntityTypeWatchFrame::Mutation {
            delivery,
            checkpoint,
        }),
        Ok(false) => Ok(EntityTypeWatchFrame::CheckpointAdvanced(checkpoint)),
        Err(error) => Err(Terminal::Permission(error.to_string())),
    }
}

async fn enqueue_initial<A, I>(
    tx: &mpsc::Sender<EntityTypeWatchFrame>,
    terminal_tx: &watch::Sender<Option<Terminal>>,
    initial: Vec<EntityTypeWatchFrame>,
    producer: &WatchProducer<A, I>,
) -> bool
where
    A: AuthzProvider,
    I: IdentityVerifier,
{
    for frame in initial {
        let authorized = tokio::select! {
            biased;
            () = tokio::time::sleep_until(producer.deadline) => {
                expire(terminal_tx);
                return false;
            }
            result = tokio::time::timeout(producer.recheck, async {
                producer
                    .identity
                    .verify(&producer.request.bearer_token)
                    .await
                    .is_ok()
                    && producer
                        .authz
                        .check(&producer.subscribe_tuple)
                        .await
                        .unwrap_or(false)
            }) => result,
        };
        if !matches!(authorized, Ok(true)) {
            terminal_tx.send_replace(Some(Terminal::Permission(
                "entity watch authority was revoked".to_owned(),
            )));
            return false;
        }
        let sent = tokio::select! {
            biased;
            () = tokio::time::sleep_until(producer.deadline) => {
                expire(terminal_tx);
                return false;
            }
            result = tokio::time::timeout(producer.recheck, tx.send(frame)) => result,
        };
        match sent {
            Ok(Ok(())) => {}
            Ok(Err(_)) => return false,
            Err(_) => {
                terminal_tx.send_replace(Some(Terminal::Frame(Box::new(
                    EntityTypeWatchFrame::Lagged {
                        checkpoint_resumable: false,
                        last_safe_checkpoint: None,
                    },
                ))));
                return false;
            }
        }
    }
    true
}

fn expire(terminal_tx: &watch::Sender<Option<Terminal>>) {
    terminal_tx.send_replace(Some(Terminal::Permission(
        "entity watch token expired".to_owned(),
    )));
}

struct OutputState {
    rx: mpsc::Receiver<EntityTypeWatchFrame>,
    terminal_rx: watch::Receiver<Option<Terminal>>,
    task: Option<tokio::task::JoinHandle<()>>,
    terminated: bool,
}

impl Drop for OutputState {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

async fn next_output(
    mut state: OutputState,
) -> Option<(Result<EntityTypeWatchFrame, PortError>, OutputState)> {
    if state.terminated {
        return None;
    }
    let immediate_terminal = state.terminal_rx.borrow().clone();
    if immediate_terminal
        .as_ref()
        .is_some_and(Terminal::preempts_buffered)
    {
        state.terminated = true;
        return Some((terminal_result(immediate_terminal?), state));
    }
    if let Ok(frame) = state.rx.try_recv() {
        let terminal = state.terminal_rx.borrow().clone();
        if terminal.as_ref().is_some_and(Terminal::preempts_buffered) {
            state.terminated = true;
            return Some((terminal_result(terminal?), state));
        }
        return Some((Ok(frame), state));
    }
    if let Some(terminal) = immediate_terminal {
        state.terminated = true;
        return Some((terminal_result(terminal), state));
    }
    let output = tokio::select! {
        biased;
        changed = state.terminal_rx.changed() => {
            if changed.is_err() {
                return None;
            }
            let terminal = state.terminal_rx.borrow().clone()?;
            if terminal.preempts_buffered() {
                Some(terminal_result(terminal))
            } else if let Ok(frame) = state.rx.try_recv() {
                Some(Ok(frame))
            } else {
                Some(terminal_result(terminal))
            }
        },
        frame = state.rx.recv() => match frame {
            Some(frame) => {
                let terminal = state.terminal_rx.borrow().clone();
                if terminal.as_ref().is_some_and(Terminal::preempts_buffered) {
                    Some(terminal_result(terminal?))
                } else {
                    Some(Ok(frame))
                }
            }
            None => state.terminal_rx.borrow().clone().map(terminal_result),
        },
    }?;
    if output.is_err()
        || matches!(
            state.terminal_rx.borrow().as_ref(),
            Some(Terminal::Frame(_))
        )
    {
        state.terminated = true;
    }
    Some((output, state))
}

fn terminal_result(terminal: Terminal) -> Result<EntityTypeWatchFrame, PortError> {
    match terminal {
        Terminal::Frame(frame) => Ok(*frame),
        Terminal::Permission(message) => Err(PortError::PermissionDenied(message)),
        Terminal::Source(message) => Err(PortError::Transport(message)),
        Terminal::Serialization(message) => Err(PortError::Serialization(message)),
    }
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
