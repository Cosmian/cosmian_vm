use anyhow::Result;

use clap::{Parser, Subcommand};

pub mod app;
pub mod snapshot;
pub mod verify;

use app::AppConfArgs;
use snapshot::SnapshotArgs;
use verify::VerifyArgs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
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

    match opts.command {
        CliCommands::Snapshot(args) => args.run().await,
        CliCommands::Verify(args) => args.run().await,
        CliCommands::App(args) => match args {
            AppConfArgs::Init(args) => args.run().await,
            AppConfArgs::Restart(args) => args.run().await,
        },
    }?;

    Ok(())
}
