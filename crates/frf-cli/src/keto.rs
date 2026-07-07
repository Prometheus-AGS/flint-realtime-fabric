//! `frf keto` — seed and revoke Ory Keto relation tuples.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use frf_authz_keto::KetoAuthzProvider;
use frf_domain::TenantId;
use frf_ports::{AuthzProvider, RelationTuple};

#[derive(Subcommand)]
pub enum KetoCommand {
    /// Write (seed) a relation tuple.
    Seed(TupleArgs),
    /// Delete (revoke) a relation tuple.
    Revoke(TupleArgs),
}

#[derive(Args)]
pub struct TupleArgs {
    /// Keto base URL. Env: `KETO_BASE_URL`.
    #[arg(long, env = "KETO_BASE_URL", default_value = "http://localhost:4467")]
    keto_url: String,
    /// Keto namespace. Env: `KETO_NAMESPACE`.
    #[arg(long, env = "KETO_NAMESPACE", default_value = "default")]
    namespace: String,
    /// Tenant UUID.
    #[arg(long)]
    tenant: String,
    /// Subject (e.g. a user or session id).
    #[arg(long)]
    subject: String,
    /// Relation (e.g. `view`, `subscribe`, `publish`).
    #[arg(long)]
    relation: String,
    /// Object id the relation applies to.
    #[arg(long)]
    object: String,
}

fn build_tuple(args: &TupleArgs) -> Result<RelationTuple> {
    let tenant_uuid = uuid::Uuid::parse_str(&args.tenant)
        .with_context(|| format!("--tenant must be a UUID, got {}", args.tenant))?;
    Ok(RelationTuple {
        tenant_id: TenantId::from_uuid(tenant_uuid),
        subject: args.subject.clone(),
        relation: args.relation.clone(),
        object: args.object.clone(),
    })
}

pub async fn run(cmd: KetoCommand) -> Result<()> {
    match cmd {
        KetoCommand::Seed(args) => {
            let provider = KetoAuthzProvider::new(&args.keto_url, &args.namespace);
            let tuple = build_tuple(&args)?;
            provider
                .write(tuple)
                .await
                .context("failed to write Keto relation tuple")?;
            println!(
                "seeded tuple: {}#{}@{} (tenant {})",
                args.object, args.relation, args.subject, args.tenant
            );
            Ok(())
        }
        KetoCommand::Revoke(args) => {
            let provider = KetoAuthzProvider::new(&args.keto_url, &args.namespace);
            let tuple = build_tuple(&args)?;
            provider
                .delete(tuple)
                .await
                .context("failed to delete Keto relation tuple")?;
            println!(
                "revoked tuple: {}#{}@{} (tenant {})",
                args.object, args.relation, args.subject, args.tenant
            );
            Ok(())
        }
    }
}
