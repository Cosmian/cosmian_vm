use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Args;
use cosmian_vm_client::client::CosmianVmClient;
use ima::snapshot::Snapshot;
use rand::RngCore;
use tee_attestation::{verify_quote, TeeMeasurement};
use tokio::task::spawn_blocking;

/// Verify a Cosmian VM
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// The URL of the Cosmian VM
    #[arg(long, action)]
    url: String,

    /// Path of the Cosmian VM snapshot
    #[arg(short, long)]
    snapshot: PathBuf,
}

impl VerifyArgs {
    pub async fn run(&self) -> Result<()> {
        println!("Fetching the collaterals...");

        let client = CosmianVmClient::instantiate(&self.url, false)?;
        let ima_binary = client.ima_binary().await?;
        let ima_binary: &[u8] = ima_binary.as_ref();
        let ima = ima::ima::Ima::try_from(ima_binary)?;

        // let ima_binary_path = Path::new("ima.binary");
        // fs::write(ima_binary_path, ima_binary)?;

        let expecting_pcr_value = client.pcr_value(ima.entries[0].pcr).await?;

        let snapshot = fs::read_to_string(&self.snapshot)?;
        let snapshot = Snapshot::try_from(snapshot.as_ref())?;

        let mut data = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut data);

        let quote = client.tee_quote(&data).await?;

        println!("Verifying the VM integrity...");
        let failures = ima.compare(&snapshot);
        if !failures.entries.is_empty() {
            let _ = failures.entries.iter().map(|entry| {
                println!(
                    "Entry ({},{}) can't be found in the snapshot!",
                    entry.path,
                    hex::encode(&entry.hash)
                );
            });
        }

        let pcr_value = ima.pcr_value()?;
        if pcr_value != hex::decode(&expecting_pcr_value)? {
            return Err(anyhow::anyhow!(
                "Bad PCR value ({} == {})",
                hex::encode(pcr_value),
                expecting_pcr_value
            ));
        }

        println!("Verifying the TPM integrity...");
        // TODO

        println!("Verifying the TEE integrity...");
        spawn_blocking(move || {
            verify_quote(
                &quote,
                &data,
                TeeMeasurement {
                    sgx: None,
                    sev: None,
                },
            )
        })
        .await??;

        Ok(())
    }
}
