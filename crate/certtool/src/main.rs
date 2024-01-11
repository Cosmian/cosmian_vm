use anyhow::Result;

use clap::{Args, Parser, Subcommand};

pub mod command;

use command::acme::generate::GenerateArgs as GenerateAcmeArgs;
#[cfg(target_os = "linux")]
use command::key::generate::KeyArgs;
use command::ratls::fetch::FetchArgs;
#[cfg(target_os = "linux")]
use command::ratls::generate::GenerateArgs as GenerateRatlsArgs;
use command::ratls::verify::VerifyArgs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand)]
enum CliCommands {
    Acme(GenerateAcmeArgs),
    Ratls(RatlsCommands),
    #[cfg(target_os = "linux")]
    Key(KeyArgs),
}

/// Generate a RATLS certificate [TEE required]
#[derive(Args, Debug)]
struct RatlsCommands {
    #[clap(subcommand)]
    subcommand: RatlsSubSubcommand,
}

#[derive(Subcommand, Debug)]
enum RatlsSubSubcommand {
    #[cfg(target_os = "linux")]
    Generate(GenerateRatlsArgs),
    Fetch(FetchArgs),
    Verify(VerifyArgs),
}

#[actix_web::main]
async fn main() -> Result<()> {
    let opts = Cli::parse();

    match opts.command {
        #[cfg(target_os = "linux")]
        CliCommands::Key(args) => args.run(),

        CliCommands::Acme(args) => args.run().await,

        CliCommands::Ratls(args) => match args.subcommand {
            #[cfg(target_os = "linux")]
            RatlsSubSubcommand::Generate(args) => args.run(),
            RatlsSubSubcommand::Fetch(args) => args.run(),
            RatlsSubSubcommand::Verify(args) => args.run(),
        },
    }?;

    Ok(())
}
