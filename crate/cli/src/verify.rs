use anyhow::Result;
use clap::Args;
use cosmian_vm_client::{
    client::{get_server_certificate_from_url, CosmianVmClient},
    cloud_provider::CloudProvider,
    snapshot::CosmianVmSnapshot,
};
use rand::RngCore;
use std::{fs, path::PathBuf};
use tee_attestation::{
    az_verify_quote as az_tee_verify_quote, forge_report_data_with_nonce,
    verify_quote as tee_verify_quote,
};
use tokio::task::spawn_blocking;
use tpm_quote::verify_quote as tpm_verify_quote;

/// Verify a Cosmian VM
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Path of the Cosmian VM snapshot
    #[arg(short, long)]
    snapshot: PathBuf,

    /// Application urls (`domain_name:port`) to verify against Cosmian VM TLS certificate
    #[arg(short, long)]
    application: Option<Vec<String>>,
}

impl VerifyArgs {
    pub async fn run(&self, client: &CosmianVmClient) -> Result<()> {
        println!("Reading the snapshot...");

        let snapshot = fs::read_to_string(&self.snapshot)?;
        let snapshot: CosmianVmSnapshot = serde_json::from_str(&snapshot)?;

        println!(
            "Fetching the collaterals... (cloud_type: {:?})",
            snapshot.cloud_type
        );

        let mut nonce: [u8; 32] = [0u8; 32];

        if let Some(cloud_type) = snapshot.cloud_type {
            if cloud_type != CloudProvider::Azure && cloud_type != CloudProvider::AWS {
                // Random nonce for all cloud provider except Microsoft Azure
                // because REPORT_DATA can't be set in quote
                rand::thread_rng().fill_bytes(&mut nonce);
            }
        }

        let quote = client.tee_quote(&nonce).await?;

        match snapshot.filehashes {
            None => {
                println!("[ WARNING ] No files hash in the snapshot");
                println!("[ SKIP ] Verifying VM integrity");
                println!("[ SKIP ] Verifying TPM attestation");
            }
            Some(filehashes) => {
                let ima_binary = client.ima_binary().await?;
                let tpm_quote_response = client.tpm_quote(&nonce).await?;

                tracing::debug!(
                    "Cosmian VM CLI: verify: tpm_quote_response: {tpm_quote_response:?}"
                );

                if ima_binary.is_empty() {
                    anyhow::bail!("No IMA list recovered");
                }

                let ima_binary: &[u8] = ima_binary.as_ref();
                tracing::debug!("Cosmian VM CLI: verify: ima_binary: {ima_binary:?}");
                let ima_entries = ima::ima::Ima::try_from(ima_binary)?;

                tpm_verify_quote(
                    &tpm_quote_response.quote,
                    &tpm_quote_response.signature,
                    &tpm_quote_response.public_key,
                    Some(&nonce),
                    &ima_entries.pcr_value(tpm_quote_response.pcr_value_hash_method)?,
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

        match snapshot.cloud_type {
            Some(CloudProvider::GCP | CloudProvider::AWS) | None => {
                policy.set_report_data(&forge_report_data_with_nonce(
                    &nonce,
                    &client.certificate.0,
                )?)?;
                spawn_blocking(move || tee_verify_quote(&quote, Some(&policy))).await??;
            }
            Some(CloudProvider::Azure) => {
                spawn_blocking(move || az_tee_verify_quote(&quote, &policy)).await??;
            }
        }

        println!("[ OK ] Verifying TEE attestation");

        if let Some(application_urls) = &self.application {
            for application_url in application_urls {
                let mut application_url = application_url.clone();
                if !application_url.starts_with("http://")
                    && !application_url.starts_with("https://")
                {
                    application_url.insert_str(0, "https://");
                }

                let app_certificate =
                    get_server_certificate_from_url(&application_url).map_err(|e| {
                        anyhow::anyhow!(format!(
                            "Can't get the application certificate for {application_url}: {e}"
                        ))
                    })?;

                if app_certificate != client.certificate.0 {
                    println!("[ FAIL ] TLS certificate for application {application_url} differs from Cosmian VM Agent TLS certificate");
                } else {
                    println!("[ OK ] Verifying TLS application for {application_url}");
                }
            }
        }

        Ok(())
    }
}
