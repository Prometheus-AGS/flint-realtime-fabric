use frf_domain::Offset;
use frf_ports::PortError;
use iggy::client::{MessageClient, TopicClient};
use iggy::clients::client::IggyClient;
use iggy::consumer::Consumer;
use iggy::error::IggyError;
use iggy::messages::poll_messages::PollingStrategy;

const POSITION_SCAN_BATCH: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PartitionSnapshot {
    pub(crate) current_offset: u64,
    pub(crate) messages_count: u64,
}

impl PartitionSnapshot {
    pub(crate) const fn retained_floor(self) -> Offset {
        Offset(
            self.current_offset
                .saturating_add(1)
                .saturating_sub(self.messages_count),
        )
    }
}

pub(crate) async fn partition_snapshot(
    client: &IggyClient,
    stream: &str,
    topic: &str,
    partition: u32,
) -> Result<PartitionSnapshot, PortError> {
    let stream_id = stream
        .try_into()
        .map_err(|error: IggyError| PortError::Transport(error.to_string()))?;
    let topic_id = topic
        .try_into()
        .map_err(|error: IggyError| PortError::Transport(error.to_string()))?;
    let details = client
        .get_topic(&stream_id, &topic_id)
        .await
        .map_err(|error| PortError::Transport(error.to_string()))?
        .ok_or_else(|| PortError::NotFound(format!("topic {stream}/{topic}")))?;
    let details = details
        .partitions
        .iter()
        .find(|candidate| candidate.id == partition)
        .ok_or_else(|| {
            PortError::NotFound(format!("partition {partition} for topic {stream}/{topic}"))
        })?;

    Ok(PartitionSnapshot {
        current_offset: details.current_offset,
        messages_count: details.messages_count,
    })
}

pub(crate) fn new_message_range(
    before: PartitionSnapshot,
    after: PartitionSnapshot,
) -> Option<(Offset, Offset)> {
    if after.messages_count == 0 {
        return None;
    }
    if before.messages_count == 0 && after.messages_count > 0 {
        return Some((after.retained_floor(), Offset(after.current_offset)));
    }
    (after.current_offset > before.current_offset).then(|| {
        let first_new = Offset(before.current_offset.saturating_add(1));
        (
            std::cmp::max(first_new, after.retained_floor()),
            Offset(after.current_offset),
        )
    })
}

pub(crate) fn validate_first_delivery(
    requested: Offset,
    delivered: Offset,
) -> Result<(), PortError> {
    if requested != Offset::BEGINNING && delivered != requested {
        return Err(PortError::NotFound(format!(
            "resnapshot_required: requested broker offset {} but first retained delivery was {}",
            requested.0, delivered.0
        )));
    }
    Ok(())
}

pub(crate) async fn locate_message(
    client: &IggyClient,
    stream: &str,
    topic: &str,
    partition: u32,
    message_id: u128,
    range: (Offset, Offset),
) -> Result<Option<Offset>, PortError> {
    let stream_id = stream
        .try_into()
        .map_err(|error: IggyError| PortError::Transport(error.to_string()))?;
    let topic_id = topic
        .try_into()
        .map_err(|error: IggyError| PortError::Transport(error.to_string()))?;
    let consumer = Consumer::default();
    let (mut cursor, end) = range;

    while cursor <= end {
        let polled = client
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(partition),
                &consumer,
                &PollingStrategy::offset(cursor.0),
                POSITION_SCAN_BATCH,
                false,
            )
            .await
            .map_err(|error| PortError::Transport(error.to_string()))?;

        if let Some(found) = polled
            .messages
            .iter()
            .find(|message| message.offset <= end.0 && message.id == message_id)
        {
            return Ok(Some(Offset(found.offset)));
        }
        let Some(last) = polled.messages.last() else {
            return Ok(None);
        };
        if last.offset >= end.0 {
            return Ok(None);
        }
        let next = last.offset.saturating_add(1);
        if next <= cursor.0 {
            return Ok(None);
        }
        cursor = Offset(next);
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_floor_advances_past_an_empty_expired_partition() {
        assert_eq!(
            PartitionSnapshot {
                current_offset: 41,
                messages_count: 0,
            }
            .retained_floor(),
            Offset(42)
        );
    }

    #[test]
    fn new_range_handles_the_first_append_and_later_appends() {
        let empty = PartitionSnapshot {
            current_offset: 0,
            messages_count: 0,
        };
        let first = PartitionSnapshot {
            current_offset: 0,
            messages_count: 1,
        };
        assert_eq!(
            new_message_range(empty, first),
            Some((Offset(0), Offset(0)))
        );

        let later = PartitionSnapshot {
            current_offset: 3,
            messages_count: 4,
        };
        assert_eq!(
            new_message_range(first, later),
            Some((Offset(1), Offset(3)))
        );
    }

    #[test]
    fn explicit_first_delivery_must_match_after_the_preflight_check() {
        assert!(validate_first_delivery(Offset::BEGINNING, Offset(41)).is_ok());
        assert!(validate_first_delivery(Offset(41), Offset(41)).is_ok());
        let error = validate_first_delivery(Offset(41), Offset(42))
            .expect_err("advanced retention must resnapshot");
        assert!(matches!(
            error,
            PortError::NotFound(message) if message.starts_with("resnapshot_required:")
        ));
    }
}
