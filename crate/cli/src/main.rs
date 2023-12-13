use anyhow::Result;

use clap::{Parser, Subcommand};

pub mod app;
pub mod snapshot;
pub mod verify;

use app::AppConfArgs;
use cosmian_vm_client::client::CosmianVmClient;
use snapshot::SnapshotArgs;
use verify::VerifyArgs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,

    /// The URL of the Cosmian VM
    #[arg(long, action)]
    url: String,

    /// Allow to connect using a self signed cert
    #[arg(long)]
    allow_self_signed: bool,
}

#[derive(Subcommand)]
enum CliCommands {
    Snapshot(SnapshotArgs),
    Verify(VerifyArgs),
    #[command(subcommand)]
    App(AppConfArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Cli::parse();

    let client = CosmianVmClient::instantiate(&opts.url, opts.allow_self_signed)?;

    match opts.command {
        CliCommands::Snapshot(args) => args.run(&client).await,
        CliCommands::Verify(args) => args.run(&client).await,
        CliCommands::App(args) => match args {
            AppConfArgs::Init(args) => args.run(&client).await,
            AppConfArgs::Restart(args) => args.run(&client).await,
        },
    }?;

    Ok(())
}
