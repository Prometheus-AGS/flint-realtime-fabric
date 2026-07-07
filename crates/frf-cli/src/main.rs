//! `frf` — operator and dev CLI for Flint Realtime Fabric.
//!
//! Binary edge: `anyhow` is used for error handling (per project rules). Each
//! subcommand drives an existing adapter (Keto authz, Iggy broker, Postgres CDC).

use anyhow::Result;
use clap::{Parser, Subcommand};

mod broker;
mod cdc;
mod keto;

#[derive(Parser)]
#[command(name = "frf", version, about = "Flint Realtime Fabric operator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage Ory Keto relation tuples (authorization).
    #[command(subcommand)]
    Keto(keto::KetoCommand),
    /// Inspect and manage the Iggy event broker.
    #[command(subcommand)]
    Broker(broker::BrokerCommand),
    /// Inspect config and manage the Postgres CDC replication slot.
    #[command(subcommand)]
    Cdc(cdc::CdcCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Keto(cmd) => keto::run(cmd).await,
        Command::Broker(cmd) => broker::run(cmd).await,
        Command::Cdc(cmd) => cdc::run(cmd).await,
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory as _;

    #[test]
    fn cli_definition_is_valid() {
        // clap's own structural verification: catches conflicting args, bad defaults,
        // and duplicate subcommands across the whole (Keto/Broker/Cdc + Slot) tree.
        Cli::command().debug_assert();
    }
}
