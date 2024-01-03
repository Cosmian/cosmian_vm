use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Args;
use cosmian_vm_client::client::CosmianVmClient;

/// Snapshot a cosmian VM
#[derive(Args, Debug)]
pub struct SnapshotArgs {
    /// Path of the fetched snapshot
    #[arg(short, long, default_value = PathBuf::from("./cosmian_vm.snapshot").into_os_string())]
    output: PathBuf,
}

impl SnapshotArgs {
    pub async fn run(&self, client: &CosmianVmClient) -> Result<()> {
        println!("Proceeding the snapshot...");

        let snapshot = client.snapshot().await?;
        fs::write(&self.output, serde_json::to_string(&snapshot)?)?;

        println!(
            "The snapshot has been saved at: {}",
            self.output.to_string_lossy()
        );

        Ok(())
    }
}
