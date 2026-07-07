//! `frf broker` — inspect and checkpoint the Iggy event broker.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use frf_broker_iggy::IggyBroker;
use frf_domain::{ChannelId, Cursor, Offset};
use frf_ports::LogBroker;

#[derive(Subcommand)]
pub enum BrokerCommand {
    /// Force a consumer checkpoint by seeking its cursor to an explicit offset.
    ///
    /// Use this to reset or advance a consumer's position (e.g. to replay from
    /// an earlier offset, or skip past a poison message).
    Checkpoint(CheckpointArgs),

    /// Inspect the stored offset for a channel/consumer.
    ///
    /// Reports the last offset the consumer has checkpointed, so an operator can
    /// see how far a consumer has progressed (or verify a `checkpoint` took effect).
    Offsets(OffsetsArgs),
}

#[derive(Args)]
pub struct OffsetsArgs {
    /// Iggy connection string. Env: `IGGY_CONNECTION_STRING`.
    #[arg(
        long,
        env = "IGGY_CONNECTION_STRING",
        default_value = "iggy://iggy:iggy@localhost:8090"
    )]
    connection: String,
    /// Channel UUID.
    #[arg(long)]
    channel: String,
    /// Consumer id whose stored offset is read.
    #[arg(long)]
    consumer: String,
}

#[derive(Args)]
pub struct CheckpointArgs {
    /// Iggy connection string. Env: `IGGY_CONNECTION_STRING`.
    #[arg(
        long,
        env = "IGGY_CONNECTION_STRING",
        default_value = "iggy://iggy:iggy@localhost:8090"
    )]
    connection: String,
    /// Channel UUID.
    #[arg(long)]
    channel: String,
    /// Consumer id whose offset is being set.
    #[arg(long)]
    consumer: String,
    /// Offset to seek the consumer cursor to.
    #[arg(long)]
    offset: u64,
}

pub async fn run(cmd: BrokerCommand) -> Result<()> {
    match cmd {
        BrokerCommand::Checkpoint(args) => {
            let channel_uuid = uuid::Uuid::parse_str(&args.channel)
                .with_context(|| format!("--channel must be a UUID, got {}", args.channel))?;

            let broker = IggyBroker::new(&args.connection)
                .await
                .context("failed to connect to Iggy broker")?;

            let cursor = Cursor {
                channel_id: ChannelId::from_uuid(channel_uuid),
                consumer_id: args.consumer.clone(),
                offset: Offset(args.offset),
                updated_at: chrono::Utc::now(),
            };

            broker
                .seek(cursor)
                .await
                .context("failed to seek consumer cursor")?;

            println!(
                "checkpoint set: channel {} consumer {} -> offset {}",
                args.channel, args.consumer, args.offset
            );
            Ok(())
        }
        BrokerCommand::Offsets(args) => {
            let channel_uuid = uuid::Uuid::parse_str(&args.channel)
                .with_context(|| format!("--channel must be a UUID, got {}", args.channel))?;

            let broker = IggyBroker::new(&args.connection)
                .await
                .context("failed to connect to Iggy broker")?;

            let stored = broker
                .get_consumer_offset(ChannelId::from_uuid(channel_uuid), &args.consumer)
                .await
                .context("failed to read consumer offset")?;

            match stored {
                Some(offset) => println!(
                    "channel {} consumer {} -> stored offset {}",
                    args.channel, args.consumer, offset.0
                ),
                None => println!(
                    "channel {} consumer {} -> no stored offset yet",
                    args.channel, args.consumer
                ),
            }
            Ok(())
        }
    }
}
