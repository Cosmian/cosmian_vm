use anyhow::Result;
use clap::Args;
use openssl::ec::{EcGroup, EcKey};
use std::{fs, path::PathBuf};

use crate::common::CURVE_NAME;

/// Generate a SECP384R1 bi-key
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Directory to store the keys
    #[arg(short, short, long, default_value = PathBuf::from(".").into_os_string())]
    output: PathBuf,
}

impl InitArgs {
    pub fn run(&self) -> Result<()> {
        let group = EcGroup::from_curve_name(CURVE_NAME)?;
        let private_key = EcKey::generate(&group)?;

        fs::create_dir_all(&self.output)?;

        let public_key_path = self.output.join(PathBuf::from("key.pub"));
        let private_key_path = self.output.join(PathBuf::from("key.pem"));

        fs::write(&public_key_path, private_key.public_key_to_pem()?)?;
        fs::write(&private_key_path, private_key.private_key_to_pem()?)?;

        println!("Public key: {public_key_path:?}");
        println!("Private key: {private_key_path:?}");

        Ok(())
    }
}
