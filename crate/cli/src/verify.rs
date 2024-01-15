use anyhow::Result;
use clap::Args;
use cosmian_vm_client::{client::CosmianVmClient, snapshot::CosmianVmSnapshot};
use rand::RngCore;
use std::{fs, path::PathBuf};
use tee_attestation::{forge_report_data_with_nonce, verify_quote as tee_verify_quote};
use tokio::task::spawn_blocking;
use tpm_quote::verify_quote as tpm_verify_quote;

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

        match snapshot.filehashes {
            None => {
                println!("[ WARNING ] No files hash in the snapshot");
                println!("[ SKIP ] Verifying VM integrity");
                println!("[ SKIP ] Verifying TPM attestation");
            }
            Some(filehashes) => {
                let ima_binary = client.ima_binary().await?;

                if ima_binary.is_empty() {
                    anyhow::bail!("No IMA list recovered");
                }

                let ima_binary: &[u8] = ima_binary.as_ref();
                let ima_entries = ima::ima::Ima::try_from(ima_binary)?;

                let tpm_quote_reponse = client.tpm_quote(&nonce).await?;
                tpm_verify_quote(
                    &tpm_quote_reponse.quote,
                    &tpm_quote_reponse.signature,
                    &tpm_quote_reponse.public_key,
                    Some(&nonce),
                    &ima_entries.pcr_value(tpm_quote_reponse.pcr_value_hash_method)?,
                    &snapshot
                        .tpm_policy
                        .ok_or_else(|| anyhow::anyhow!("TPM policy is missing in the snapshot"))?,
                )?;

                println!("[ OK ] Verifying TPM attestation");

                let failures = ima_entries.compare(&filehashes.0);
                if !failures.entries.is_empty() {
                    failures.entries.iter().for_each(|entry| {
                        println!(
                            "Entry ({},{}) can't be found in the snapshot!",
                            entry.filename_hint,
                            hex::encode(&entry.filedata_hash)
                        );
                    });
                    println!("[ FAIL ] Verifying VM integrity");
                    anyhow::bail!("Unexpected binaries found!");
                } else {
                    println!(
                        "[ OK ] Verifying VM integrity (against {} files)",
                        filehashes.0.len()
                    );
                }
            }
        };

        let mut policy = snapshot.tee_policy;
        policy.set_report_data(&forge_report_data_with_nonce(
            &nonce,
            &client.certificate.0,
        )?)?;
        spawn_blocking(move || tee_verify_quote(&quote, Some(&policy))).await??;

        println!("[ OK ] Verifying TEE attestation");

        Ok(())
    }
}
