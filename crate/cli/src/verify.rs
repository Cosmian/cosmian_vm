use anyhow::Result;
use clap::Args;
use cosmian_vm_client::{client::CosmianVmClient, snapshot::CosmianVmSnapshot};
use rand::RngCore;
use std::{fs, path::PathBuf};
use tee_attestation::{forge_report_data_with_nonce, verify_quote as tee_verify_quote};
use tokio::task::spawn_blocking;
use tpm_quote::{verify_pcr_value, verify_quote as tpm_verify_quote};

/// Verify a Cosmian VM
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Path of the Cosmian VM snapshot
    #[arg(short, long)]
    snapshot: PathBuf,
}

impl VerifyArgs {
    pub async fn run(&self, client: &CosmianVmClient) -> Result<()> {
        println!("Reading the snapshot...");

        let snapshot = fs::read_to_string(&self.snapshot)?;
        let snapshot: CosmianVmSnapshot = serde_json::from_str(&snapshot)?;

        println!("Fetching the collaterals...");

        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);

        let quote = client.tee_quote(&nonce).await?;

        if snapshot.filehashes.0.is_empty() {
            println!("[ WARNING ] No files hash in the snapshot");
            println!("[ SKIP ] Verifying VM integrity");
            println!("[ SKIP ] Verifying TPM attestation");
        } else {
            let ima_binary = client.ima_binary().await?;

            if ima_binary.is_empty() {
                anyhow::bail!("No IMA list recovered");
            }

            let ima_binary: &[u8] = ima_binary.as_ref();
            let ima_entries = ima::ima::Ima::try_from(ima_binary)?;

            let failures = ima_entries.compare(&snapshot.filehashes.0);
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

            let tpm_quote_reponse = client.tpm_quote(&nonce).await?;
            let quote_info = tpm_verify_quote(
                &tpm_quote_reponse.quote,
                &tpm_quote_reponse.signature,
                &tpm_quote_reponse.public_key,
                Some(&nonce),
            )?;

            verify_pcr_value(&quote_info, &ima_entries.pcr_value()?)?;
            println!("[ OK ] Verifying TPM attestation");
        }

        let report_data = forge_report_data_with_nonce(&nonce, &client.certificate.0)?;

        let mut policy = snapshot.policy;
        policy.set_report_data(&report_data)?;
        spawn_blocking(move || tee_verify_quote(&quote, Some(&policy))).await??;

        println!("[ OK ] Verifying TEE attestation");

        Ok(())
    }
}
