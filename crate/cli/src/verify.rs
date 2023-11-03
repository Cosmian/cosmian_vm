use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::Args;
use cosmian_vm_agent::core::{parse_ima_ascii, ImaEntry};
use cosmian_vm_client::client::CosmianVmClient;
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

const IMA_MEASURE_BIN_PATH: &str = "/home/seb/tmp/ima-tests/ima_measure";

impl VerifyArgs {
    pub async fn run(&self) -> Result<()> {
        println!("Fetching the collaterals...");

        let client = CosmianVmClient::instantiate(&self.url, false)?;
        let ima_ascii = client.ima_ascii().await?;
        let ima_binary = client.ima_binary().await?;

        let ima_binary_path = Path::new("ima.binary");
        fs::write(ima_binary_path, ima_binary)?;

        let ima = parse_ima_ascii(&ima_ascii)?;
        let pcr_id = ima[0].pcr;
        let pcr_value = client.pcr_value(pcr_id).await?;

        let snapshot = fs::read_to_string(&self.snapshot)?;
        let snapshot: Vec<(&str, &str)> = snapshot
            .lines()
            .map(|line| {
                let mut s = line.split(r"\f");
                (s.next().unwrap(), s.next().unwrap())
            })
            .collect();

        let mut data = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut data);

        let quote = client.tee_quote(&data).await?;

        println!("Verifying the VM integrity...");
        verify_ima(&ima, &snapshot);
        verify_pcr_value(ima_binary_path, &pcr_value)?; // TODO: dev our own implemnt for compute the pcr_value

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

pub fn verify_ima(ima: &[ImaEntry], snapshot: &[(&str, &str)]) -> bool {
    let mut ret = true;

    for entry in ima {
        if entry.filename_hint == "boot_aggrate" {
            continue;
        }

        let found = snapshot
            .iter()
            .any(|item| *item == (&entry.filedata_hash, &entry.filename_hint));

        if !found {
            println!(
                "Entry ({},{}) can't be found in the snapshot!",
                entry.filename_hint, entry.filedata_hash
            );
            ret = false;
        }
    }

    ret
}

pub fn verify_pcr_value(ima_binary_path: &Path, expected_pcr_value: &str) -> Result<()> {
    let output = Command::new(IMA_MEASURE_BIN_PATH)
        .arg(ima_binary_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Command returns an error (code: {}): , {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let output = String::from_utf8_lossy(&output.stdout);
    let output = output.trim_end();
    let output = &output[(output.len() - 59)..].replace(' ', "");

    if output == expected_pcr_value {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "Bad PCR value ({output} == {expected_pcr_value})"
    ))
}
