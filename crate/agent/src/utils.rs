use crate::error::Error;

use std::process::Command;
use std::{path::Path, str::FromStr};
use tss_esapi::{Context, TctiNameConf};

pub(crate) fn call(exe: &str, args: &[&str], background: bool) -> Result<Option<String>, Error> {
    if background {
        let _ = Command::new(exe).args(args).spawn()?;
        return Ok(None);
    }

    match Command::new(exe).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
            } else {
                Err(Error::Command(format!(
                    "Output: {} - error: {}",
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim(),
                )))
            }
        }
        Err(e) => Err(Error::Command(e.to_string())),
    }
}

pub(crate) fn create_tpm_context(tpm_device: &Path) -> Result<Context, Error> {
    let tcti = TctiNameConf::from_str(&format!("device:{}", &tpm_device.to_string_lossy()))
        .map_err(|e| Error::Unexpected(format!("Incorrect TCTI (TPM device): {e}")))?;

    let tpm_context = Context::new(tcti).map_err(|e| {
        Error::Unexpected(format!("Can't build context from TCTI (TPM device): {e}"))
    })?;

    Ok(tpm_context)
}
