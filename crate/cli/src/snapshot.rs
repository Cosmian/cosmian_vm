use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Args;
use cosmian_vm_client::client::CosmianVmClient;

/// Snapshot a cosmian VM
#[derive(Args, Debug)]
pub struct SnapshotArgs {
    /// Path to save the snapshot
    #[arg(short, long, default_value = PathBuf::from("./cosmian_vm.snapshot").into_os_string())]
    output: PathBuf,
}

impl SnapshotArgs {
    pub async fn run(&self, client: &CosmianVmClient) -> Result<()> {
        println!("Processing the snapshot...");

        // Reset the previous snapshot (or fail is the snapshotting process is still running)
        client.reset_snapshot().await?;

        let snapshot = client.get_snapshot().await?;
        fs::write(&self.output, serde_json::to_string(&snapshot)?)?;

        println!(
            "The snapshot has been saved at: {}",
            self.output.to_string_lossy()
        );

        Ok(())
    }
}
