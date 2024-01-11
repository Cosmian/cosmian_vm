use anyhow::Result;
use clap::Args;
use openssl::ec::EcKey;
use std::{fs, path::PathBuf};

use crate::common::{decrypt, unique_filename};

/// Decrypt a file from a trusted environment
#[derive(Args, Debug)]
pub struct DecryptArgs {
    /// Path of the file to decrypt
    #[arg(short, long, required = true)]
    file: PathBuf,

    /// Directory to store the decrypted file
    #[arg(short, short, long, default_value = PathBuf::from(".").into_os_string())]
    output: PathBuf,

    /// Path of the client private key (PEM format)
    #[arg(long, required = true)]
    private_key: PathBuf,
}

impl DecryptArgs {
    pub fn run(&self) -> Result<()> {
        // Step 1: Load the bi-key for the ECDH
        let private_key = EcKey::private_key_from_pem(&fs::read(&self.private_key)?)?;

        // Step 2: Decrypt the file
        let content = fs::read(&self.file)?;
        let output_bytes = decrypt(&content, private_key)?;

        // Step 3: Write the decrypted file
        let decrypted_filepath: PathBuf = self.output.join(unique_filename()?);
        fs::create_dir_all(&self.output)?;
        fs::write(&decrypted_filepath, output_bytes)?;

        println!("Plain file: {decrypted_filepath:?}");

        Ok(())
    }
}
