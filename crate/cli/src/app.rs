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
    /// The URL of the cosmian VM
    #[arg(long, action)]
    url: String,

    /// Path of the app configuration to upload
    #[arg(short, long)]
    configuration: PathBuf,

    /// Optional key to encrypt the uploaded configuration on the VM.
    ///
    /// The key must be provided hex encoded.
    ///
    /// If no key is provided, a random one will be generated
    #[arg(short, long)]
    key: Option<String>,
}

impl InitArgs {
    pub async fn run(&self) -> Result<()> {
        println!("Proceeding the init of the deployed app...");

        let cfg_content = std::fs::read(&self.configuration)?;
        let key = if let Some(key) = &self.key {
            Some(hex::decode(key)?)
        } else {
            None
        };

        let client = CosmianVmClient::instantiate(&self.url, false)?;
        if let Some(key) = client.init_app(&cfg_content, key.as_deref()).await? {
            println!("Save the key: `{}`", hex::encode(key));
        }

        println!("The app has been configurated");

        Ok(())
    }
}

/// Restart the deployed application
#[derive(Args, Debug)]
pub struct RestartArgs {
    /// The URL of the cosmian VM
    #[arg(long, action)]
    url: String,

    /// Optional key/password used to decrypt the app configuration
    ///
    /// The key must be provided hex encoded.
    #[arg(short, long)]
    key: String,
}

impl RestartArgs {
    pub async fn run(&self) -> Result<()> {
        println!("Proceeding the restart of the deployed app...");

        let client = CosmianVmClient::instantiate(&self.url, false)?;
        client.restart_app(&hex::decode(&self.key)?).await?;

        println!("The app has been restarted");

        Ok(())
    }
}
