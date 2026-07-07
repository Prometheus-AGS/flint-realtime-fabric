//! `frf cdc` — inspect config and manage the Postgres CDC replication slot.
//!
//! `status` reports the resolved config and replication URL so an operator can verify
//! slot/publication naming before bring-up. `slot create|drop` manages the logical
//! replication slot explicitly (the gateway's `PostgresCdcConsumer` also creates it at
//! startup) — useful for pre-provisioning or for releasing pinned WAL during recovery.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use frf_domain::TenantId;
use frf_postgres_cdc::CdcConfig;
use tokio_postgres::NoTls;

#[derive(Subcommand)]
pub enum CdcCommand {
    /// Show the resolved CDC slot/publication configuration and replication URL.
    Status(StatusArgs),

    /// Manage the logical replication slot (create / drop).
    #[command(subcommand)]
    Slot(SlotCommand),
}

#[derive(Subcommand)]
pub enum SlotCommand {
    /// Create the logical replication slot (idempotent-friendly: reports if it exists).
    Create(SlotArgs),
    /// Drop the logical replication slot.
    Drop(SlotArgs),
}

#[derive(Args)]
pub struct SlotArgs {
    /// Postgres database URL (a normal connection, not `replication=database`).
    /// Env: `CDC_REPLICATION_URL`.
    #[arg(
        long,
        env = "CDC_REPLICATION_URL",
        default_value = "postgres://frf:frf@localhost:5432/frf"
    )]
    database_url: String,
    /// Replication slot name. Env: `CDC_SLOT_NAME`.
    #[arg(long, env = "CDC_SLOT_NAME", default_value = "frf_slot")]
    slot: String,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Postgres database URL. Env: `CDC_REPLICATION_URL`.
    #[arg(
        long,
        env = "CDC_REPLICATION_URL",
        default_value = "postgres://frf:frf@localhost:5432/frf"
    )]
    database_url: String,
    /// Replication slot name. Env: `CDC_SLOT_NAME`.
    #[arg(long, env = "CDC_SLOT_NAME", default_value = "frf_slot")]
    slot: String,
    /// Publication name. Env: `CDC_PUBLICATION_NAME`.
    #[arg(long, env = "CDC_PUBLICATION_NAME", default_value = "frf_pub")]
    publication: String,
    /// Tenant UUID that ingested changes are stamped with. Env: `CDC_TENANT_ID`.
    #[arg(
        long,
        env = "CDC_TENANT_ID",
        default_value = "00000000-0000-0000-0000-000000000001"
    )]
    tenant: String,
    /// Channel path on the spine. Env: `CDC_CHANNEL_PATH`.
    #[arg(long, env = "CDC_CHANNEL_PATH", default_value = "entities")]
    channel_path: String,
}

pub async fn run(cmd: CdcCommand) -> Result<()> {
    match cmd {
        CdcCommand::Slot(slot_cmd) => run_slot(slot_cmd).await,
        CdcCommand::Status(args) => {
            let tenant_uuid = uuid::Uuid::parse_str(&args.tenant)
                .map_err(|_| anyhow::anyhow!("--tenant must be a UUID, got {}", args.tenant))?;

            let config = CdcConfig::new(
                &args.database_url,
                &args.slot,
                &args.publication,
                TenantId::from_uuid(tenant_uuid),
                &args.channel_path,
            );

            println!("CDC configuration:");
            println!("  slot name:         {}", config.slot_name);
            println!("  publication:       {}", config.publication_name);
            println!("  tenant id:         {}", args.tenant);
            println!("  channel path:      {}", config.channel_path);
            println!("  replication url:   {}", config.replication_url());
            println!();
            println!(
                "The slot is created/managed by the gateway's PostgresCdcConsumer at \
                 startup, or explicitly via `frf cdc slot create`. Ensure Postgres runs \
                 with wal_level=logical and that the above names match the gateway's env."
            );
            Ok(())
        }
    }
}

/// Connect to Postgres over a normal (non-replication) connection and run the slot
/// admin function. Returns the connected client; the background connection task is
/// spawned and detached for the short lifetime of a single admin command.
async fn connect(database_url: &str) -> Result<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls)
        .await
        .context("failed to connect to Postgres")?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error = %e, "postgres connection error");
        }
    });
    Ok(client)
}

async fn run_slot(cmd: SlotCommand) -> Result<()> {
    match cmd {
        SlotCommand::Create(args) => {
            let client = connect(&args.database_url).await?;
            // Slot name is a bound parameter to the admin function (not string-interpolated
            // into SQL), so a hostile name cannot inject. `pgoutput` is the standard
            // logical-decoding plugin used by the gateway's replication stream.
            let existing = client
                .query_opt(
                    "SELECT 1 FROM pg_replication_slots WHERE slot_name = $1",
                    &[&args.slot],
                )
                .await
                .context("failed to query existing slots")?;
            if existing.is_some() {
                println!(
                    "replication slot '{}' already exists — nothing to do",
                    args.slot
                );
                return Ok(());
            }
            client
                .execute(
                    "SELECT pg_create_logical_replication_slot($1, 'pgoutput')",
                    &[&args.slot],
                )
                .await
                .context("failed to create replication slot")?;
            println!("created logical replication slot '{}'", args.slot);
            Ok(())
        }
        SlotCommand::Drop(args) => {
            let client = connect(&args.database_url).await?;
            let existing = client
                .query_opt(
                    "SELECT 1 FROM pg_replication_slots WHERE slot_name = $1",
                    &[&args.slot],
                )
                .await
                .context("failed to query existing slots")?;
            if existing.is_none() {
                println!(
                    "replication slot '{}' does not exist — nothing to do",
                    args.slot
                );
                return Ok(());
            }
            client
                .execute("SELECT pg_drop_replication_slot($1)", &[&args.slot])
                .await
                .context("failed to drop replication slot")?;
            println!("dropped logical replication slot '{}'", args.slot);
            Ok(())
        }
    }
}
