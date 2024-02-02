use crate::error::Error;
use crate::utils::call;

use std::path::Path;
use std::process::Command;

/// Generate the TPM keys during the first startup of the agent
/// - Ignore generation if already done previously
/// - Raise an error if no TPM detected
///
/// Note: this function should be replace in a near feature (waiting for a patch in the tpm lib)
pub(crate) fn generate_tpm_keys(tpm_device_path: &Path) -> Result<(), Error> {
    if !tpm_device_path.exists() {
        return Err(Error::Configuration(format!(
            "TPM device path unknown: {tpm_device_path:?} "
        )));
    }

    // Verify the keys has not been already generated
    match Command::new("tpm2_readpublic")
        .args(["-c", "0x81000000"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                return Ok(());
            }
        }
        Err(e) => return Err(Error::Command(e.to_string())),
    }

    // Otherwise generate them
    //
    // # create EK and make it persistent
    // sudo tpm2_createek --ek-context=ek.ctx --key-algorithm=ecc --public=ek.pub --format=pem
    // sudo tpm2_evictcontrol --hierarchy=o --object-context=ek.ctx --output=ek.handle
    // # create AK and make it persistent
    // sudo tpm2_createak --ek-context=ek.handle --ak-context=ak.ctx --key-algorithm=ecc --hash-algorithm=sha256 --public=ak.pub --format pem --ak-name=ak.name
    // sudo tpm2_evictcontrol --hierarchy=o --object-context=ak.ctx --output=ak.handle

    // Create EK and make it persistent
    tracing::info!("Generating TPM EK & AK...");
    call(
        "tpm2_createek",
        &[
            "--ek-context=/tmp/ek.ctx",
            "--key-algorithm=ecc",
            "--public=/tmp/ek.pub",
            "--format=pem",
        ],
        false,
    )?;

    call(
        "tpm2_evictcontrol",
        &[
            "--hierarchy=o",
            "--object-context=/tmp/ek.ctx",
            "--output=/tmp/ek.handle",
        ],
        false,
    )?;

    // Create AK and make it persistent
    call(
        "tpm2_createak",
        &[
            "--ek-context=/tmp/ek.handle",
            "--ak-context=/tmp/ak.ctx",
            "--key-algorithm=ecc",
            "--hash-algorithm=sha256",
            "--public=/tmp/ak.pub",
            "--format=pem",
            "--ak-name=/tmp/ak.name",
        ],
        false,
    )?;

    call(
        "tpm2_evictcontrol",
        &[
            "--hierarchy=o",
            "--object-context=/tmp/ak.ctx",
            "--output=/tmp/ak.handle",
        ],
        false,
    )?;

    Ok(())
}
