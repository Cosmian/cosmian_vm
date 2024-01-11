use anyhow::Result;
use clap::Args;
use hex::decode;
use ratls::verify::verify_ratls;
use std::path::PathBuf;
use std::{fs, ops::Deref};
use tee_attestation::{SevQuoteVerificationPolicy, SgxQuoteVerificationPolicy, TeePolicy};

/// Verify a RATLS certificate
#[derive(Args, Debug)]
#[clap(verbatim_doc_comment)]
pub struct VerifyArgs {
    /// Path of the certificate to verify
    #[arg(short, long)]
    cert: PathBuf,

    /// Expected value of the SEV measurement
    #[arg(long, required = false)]
    measurement: Option<String>,

    /// Expected value of the SGX mrenclave
    #[arg(long, required = false)]
    mrenclave: Option<String>,

    /// Path of the SGX enclave signer key (to compute the SGX mrsigner)
    #[arg(long)]
    public_signer_key: Option<PathBuf>,
}

impl VerifyArgs {
    pub fn run(&self) -> Result<()> {
        let public_signer_key = if let Some(path) = &self.public_signer_key {
            Some(fs::read_to_string(path)?)
        } else {
            None
        };

        let mrenclave: Option<[u8; 32]> = if let Some(v) = &self.mrenclave {
            Some(decode(v)?.as_slice().try_into()?)
        } else {
            None
        };

        let sev_measurement = if let Some(v) = &self.measurement {
            Some(decode(v)?.as_slice().try_into()?)
        } else {
            None
        };

        let mut policy = match (public_signer_key, mrenclave, sev_measurement) {
            (None, None, None) => None,
            (Some(s), Some(e), None) => Some(TeePolicy::Sgx(SgxQuoteVerificationPolicy::new(e, s.deref())?)),
            (None, None, Some(m)) => Some(TeePolicy::Sev(SevQuoteVerificationPolicy::new(m))),
            _ => anyhow::bail!("Bad measurements combination. It should be [None | (--mrenclave & --signer_key) | measurement]")
        };

        verify_ratls(fs::read_to_string(&self.cert)?.as_bytes(), policy.as_mut())?;

        println!("Verification succeed!");

        Ok(())
    }
}
