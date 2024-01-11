use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use pem_rfc7468::LineEnding;
use ratls::verify::get_server_certificate;

/// Fetch an RATLS certificate from a domain name
#[derive(Args, Debug)]
pub struct FetchArgs {
    /// The server name to fetch
    #[arg(long, action)]
    hostname: String,

    /// The port to fetch
    #[arg(long, short, action, default_value_t = 443)]
    port: u32,

    /// Path of the fetched certificate
    #[arg(short, long, default_value = PathBuf::from(".").into_os_string())]
    output: PathBuf,
}

impl FetchArgs {
    pub fn run(&self) -> Result<()> {
        let cert = get_server_certificate(&self.hostname, self.port)?;
        let cert = pem_rfc7468::encode_string("CERTIFICATE", LineEnding::default(), &cert)
            .map_err(|e| anyhow!(e))?;

        let cert_path = self.output.join(PathBuf::from("cert.ratls.pem"));

        fs::create_dir_all(&self.output)?;
        fs::write(&cert_path, cert)?;

        println!("RATLS certificate: {cert_path:?}");

        Ok(())
    }
}
