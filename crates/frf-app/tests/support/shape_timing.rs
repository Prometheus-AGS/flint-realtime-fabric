use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use frf_ports::{PortError, ShapeBodyStream};
use futures_util::Stream;
use tokio::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EventKind {
    AuthorityCheck(usize),
    UpstreamFrame(usize),
    UpstreamPending,
    UpstreamComplete,
    UpstreamDrop,
    ConsumerFrame(usize),
    CancellationObserved,
    ConsumerDrop,
    NormalComplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimedEvent {
    kind: EventKind,
    elapsed: Duration,
}

pub(super) struct Timeline {
    origin: Instant,
    events: Mutex<Vec<TimedEvent>>,
}

impl Timeline {
    pub(super) fn new() -> Self {
        Self {
            origin: Instant::now(),
            events: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn record(&self, kind: EventKind) {
        self.lock().push(TimedEvent {
            kind,
            elapsed: Instant::now().duration_since(self.origin),
        });
    }

    pub(super) fn at(&self, kind: EventKind) -> Option<Duration> {
        self.lock()
            .iter()
            .find(|event| event.kind == kind)
            .map(|event| event.elapsed)
    }

    pub(super) fn report(&self, scenario: &str) {
        eprintln!("{scenario}: {:?}", self.lock().clone());
    }

    fn lock(&self) -> MutexGuard<'_, Vec<TimedEvent>> {
        match self.events.lock() {
            Ok(events) => events,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

struct TimedBody {
    frames: VecDeque<Bytes>,
    next_frame: usize,
    stall_when_empty: bool,
    pending_recorded: bool,
    timeline: Arc<Timeline>,
}

impl Stream for TimedBody {
    type Item = Result<Bytes, PortError>;

    fn poll_next(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(frame) = self.frames.pop_front() {
            self.next_frame += 1;
            self.timeline
                .record(EventKind::UpstreamFrame(self.next_frame));
            return Poll::Ready(Some(Ok(frame)));
        }
        if self.stall_when_empty {
            if !self.pending_recorded {
                self.pending_recorded = true;
                self.timeline.record(EventKind::UpstreamPending);
            }
            Poll::Pending
        } else {
            self.timeline.record(EventKind::UpstreamComplete);
            Poll::Ready(None)
        }
    }
}

impl Drop for TimedBody {
    fn drop(&mut self) {
        self.timeline.record(EventKind::UpstreamDrop);
    }
}

pub(super) fn timed_body(
    timeline: &Arc<Timeline>,
    frames: &[&'static [u8]],
    stall_when_empty: bool,
) -> ShapeBodyStream {
    Box::pin(TimedBody {
        frames: frames
            .iter()
            .map(|frame| Bytes::from_static(frame))
            .collect(),
        next_frame: 0,
        stall_when_empty,
        pending_recorded: false,
        timeline: Arc::clone(timeline),
    })
}
