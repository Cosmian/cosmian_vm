use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Args;
use cosmian_vm_client::client::CosmianVmClient;

/// Snapshot a cosmian VM
#[derive(Args, Debug)]
pub struct SnapshotArgs {
    /// The URL of the cosmian VM
    #[arg(long, action)]
    url: String,

    /// Path of the fetched snapshot
    #[arg(short, long, default_value = PathBuf::from("./cosmian_vm.snapshot").into_os_string())]
    output: PathBuf,
}

impl SnapshotArgs {
    pub async fn run(&self) -> Result<()> {
        println!("Proceeding the snapshot...");

        let client = CosmianVmClient::instantiate(&self.url, false)?;
        let snapshot = client.snapshot().await?;

        fs::write(&self.output, snapshot)?;

        println!(
            "The snapshot has been saved at {}",
            self.output.to_string_lossy()
        );

        Ok(())
    }
}
