use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};
use cosmian_vm_client::client::CosmianVmClient;

#[derive(Subcommand)]
pub enum AppConfArgs {
    Init(InitArgs),
    Restart(RestartArgs),
}

/// Init the deployed application by providing the conf
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Path of the app configuration to upload
    #[arg(short, long)]
    configuration: PathBuf,
}

impl InitArgs {
    pub async fn run(&self, client: &CosmianVmClient) -> Result<()> {
        println!("Processing the init of the deployed app...");

        let cfg_content = std::fs::read(&self.configuration)?;

        if let Some(key) = client.init_app(&cfg_content).await? {
            println!("Save the key: `{}`", hex::encode(key));
        }

        println!("The app has been configurated");

        Ok(())
    }
}

/// Restart the deployed application
#[derive(Args, Debug)]
pub struct RestartArgs {}

impl RestartArgs {
    pub async fn run(&self, client: &CosmianVmClient) -> Result<()> {
        println!("Processing the restart of the deployed app...");

        client.restart_app().await?;

        println!("The app has been restarted");

        Ok(())
    }
}
