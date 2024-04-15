use std::{fs::File, path::Path};

use self::{
    certificate::generate_self_signed_cert, luks::generate_encrypted_fs, tpm::generate_tpm_keys,
};
use crate::{conf::CosmianVmAgent, error::Error, VAR_PATH};
use const_format::formatcp;

mod certificate;
mod luks;
mod tpm;

/// A file we store to remember the agent has already been configured once
const AGENT_INITIALIZED_CACHE_PATH: &str = formatcp!("{VAR_PATH}/cache.init");

/// Test if the agent has already been configured once
fn is_initialized() -> bool {
    Path::new(AGENT_INITIALIZED_CACHE_PATH).exists()
}

/// Store the fact that the agent has been configured once
fn initialized() -> Result<(), Error> {
    File::create(AGENT_INITIALIZED_CACHE_PATH)?;
    Ok(())
}

/// Initialize the agent if it has never be done before:
/// 1. Create a default luks container
/// 2. Create a default SSL certificate
/// 3. Create TPM enforcement keys
pub fn initialize_agent(conf: &CosmianVmAgent) -> Result<(), Error> {
    // If already done once, just ignore this function
    if is_initialized() {
        return Ok(());
    }

    // Generate TPM keys if tpm is enabled in the config file
    if let Some(tpm_device) = &conf.agent.tpm_device {
        generate_tpm_keys(tpm_device)?;
    } else {
        tracing::warn!("No TPM configuration found: TPM generation keys skipped!");
        tracing::warn!(
            "The agent is not configured to support TPM and files integrity verification"
        );
    }

    // Generate the default encrypted fs
    generate_encrypted_fs()?;

    // Generate the default self signed certificate
    generate_self_signed_cert(
        &conf.agent.ssl_private_key(),
        &conf.agent.ssl_certificate(),
        &conf.agent.host,
    )?;

    // Assure we don't pass in that function anymore
    initialized()
}
