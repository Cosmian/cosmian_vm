use anyhow::{anyhow, Result};
use clap::Args;
use cosmian_vm_client::{client::CosmianVmClient, snapshot::CosmianVmSnapshot};
use pem_rfc7468::{encode_string, LineEnding};
use rand::RngCore;
use std::{fs, path::PathBuf};
use tee_attestation::{forge_report_data_with_nonce, verify_quote};
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

        if ima_binary.is_empty() {
            anyhow::bail!("No IMA list recovered");
        }

        let ima_binary: &[u8] = ima_binary.as_ref();
        let ima = ima::ima::Ima::try_from(ima_binary)?;
        let expecting_pcr_value = client.pcr_value(ima.pcr_id()).await?;

        let snapshot = fs::read_to_string(&self.snapshot)?;
        let snapshot: CosmianVmSnapshot = serde_json::from_str(&snapshot)?;

        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);

        let quote = client.tee_quote(&nonce).await?;

        let failures = ima.compare(&snapshot.filehashes.0);
        if !failures.entries.is_empty() {
            failures.entries.iter().for_each(|entry| {
                println!(
                    "Entry ({},{}) can't be found in the snapshot!",
                    entry.filename_hint,
                    hex::encode(&entry.filedata_hash)
                );
            });
            println!("[ FAIL ] Verifying VM integrity");
            anyhow::bail!("Unexpected binaries found");
        } else {
            println!("[ OK ] Verifying VM integrity");
        }

        let pcr_value = ima.pcr_value()?;
        if pcr_value != hex::decode(&expecting_pcr_value)? {
            println!("[ FAIL ] Verifying TPM attestation");
            anyhow::bail!(
                "Bad PCR value '{}' (Expecting: '{}')",
                hex::encode(pcr_value),
                expecting_pcr_value.to_lowercase()
            );
        }

        // TODO
        println!("[ OK ] Verifying TPM attestation");

        let report_data = forge_report_data_with_nonce(
            &nonce,
            encode_string("CERTIFICATE", LineEnding::default(), &client.certificate.0)
                .map_err(|e| anyhow!(e))?
                .as_bytes(),
        )?;

        let mut policy = snapshot.policy;
        policy.set_report_data(&report_data)?;
        spawn_blocking(move || verify_quote(&quote, Some(&policy))).await??;

        println!("[ OK ] Verifying TEE attestation");

        Ok(())
    }
}
