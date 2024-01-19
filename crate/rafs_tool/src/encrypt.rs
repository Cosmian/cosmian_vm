use anyhow::{anyhow, Result};
use clap::Args;
use hex::decode;
use openssl::{
    bn::BigNumContext,
    ec::{EcGroup, EcKey, PointConversionForm},
};
use sev_quote::{
    policy::SevQuoteVerificationPolicy,
    quote::{parse_quote, verify_quote},
};
use std::{fs, path::PathBuf};

use crate::common::{
    derive_shared_key, encrypt, sha256, unique_filename, CURVE_NAME, QUOTE_FINGERPRINT_SIZE,
};

/// Encrypt a file for a trusted environment
#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// Path of the file to encrypt
    #[arg(short, long, required = true)]
    file: PathBuf,

    /// Directory to store the encrypted file
    #[arg(short, short, long, default_value = PathBuf::from(".").into_os_string())]
    output: PathBuf,

    /// Path of the quote
    #[arg(short, long, required = true)]
    quote: PathBuf,

    /// Path of the trusted environment public key (PEM format)
    #[arg(short, long, required = true)]
    key: PathBuf,

    /// Expected value of the SEV measurement
    #[arg(long, required = false)]
    measurement: Option<String>,
}

impl EncryptArgs {
    pub fn run(&self) -> Result<()> {
        // Step 1: Generate the bi-key for the ECDH
        let group = EcGroup::from_curve_name(CURVE_NAME)?;
        let private_key = EcKey::generate(&group)?;
        let peer_public_key = EcKey::public_key_from_pem(&fs::read(&self.key)?)?;

        // Step 2: Verify the quote
        let sev_measurement = if let Some(v) = &self.measurement {
            Some(decode(v)?.as_slice().try_into()?)
        } else {
            None
        };

        let quote = parse_quote(&fs::read(&self.quote)?)?;
        verify_quote(
            &quote,
            &SevQuoteVerificationPolicy {
                measurement: sev_measurement,
                ..Default::default()
            },
        )?;

        if quote.report.report_data[0..QUOTE_FINGERPRINT_SIZE]
            != sha256(&peer_public_key.public_key_to_der()?)
        {
            return Err(anyhow!("The trusted environment public key does not match the value in the quote report data"));
        }

        // Step 3: Compute the shared_key
        let shared_key = derive_shared_key(private_key.clone(), peer_public_key)?;

        // Step 4: Encrypt the file
        let content = fs::read(&self.file)?;
        let mut ctx = BigNumContext::new()?;

        let output_bytes = encrypt(
            &content,
            shared_key,
            &private_key.public_key().to_bytes(
                &group,
                PointConversionForm::COMPRESSED,
                &mut ctx,
            )?,
        )?;

        // Step 5: Write the encrypted file
        let encrypted_filepath: PathBuf = self.output.join(unique_filename()?);
        fs::create_dir_all(&self.output)?;
        fs::write(&encrypted_filepath, output_bytes)?;

        println!("Plain file: {encrypted_filepath:?}");

        Ok(())
    }
}
