use crate::{error::Error, utils::call, BIN_PATH, VAR_PATH};

use const_format::formatcp;
use rand::{distributions::Alphanumeric, Rng};

use std::path::Path;

const FSTOOL_PATH: &str = formatcp!("{BIN_PATH}/cosmian_fstool");
const FSTOOL_DEFAULT_SIZE: &str = "500MB";
const FSTOOL_DEFAULT_CONTAINER_FILE: &str = formatcp!("{VAR_PATH}/container");
const FSTOOL_DEFAULT_CONTAINER_MOUNTPOINT: &str = formatcp!("{VAR_PATH}/data");
const FSTOOL_DEFAULT_PASSWORD_LENGTH: usize = 32;

/// Generate a luks container
///
/// If the container already exists: just return `Ok`
///
/// Note: the password of the generated container will be written in the log
pub(crate) fn generate_encrypted_fs() -> Result<(), Error> {
    if Path::new(&FSTOOL_DEFAULT_CONTAINER_FILE).exists() {
        // Already done: don't proceed further
        return Ok(());
    }

    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(FSTOOL_DEFAULT_PASSWORD_LENGTH)
        .map(char::from)
        .collect();

    call(
        &std::env::var("COSMIAN_VM_FSTOOL").unwrap_or(FSTOOL_PATH.to_string()),
        &[
            "--size",
            FSTOOL_DEFAULT_SIZE,
            "--location",
            FSTOOL_DEFAULT_CONTAINER_FILE,
            "--password",
            &password,
        ],
        false,
    )?;

    tracing::info!("The container has been generated at: {FSTOOL_DEFAULT_CONTAINER_FILE:?} and is mounted at: {FSTOOL_DEFAULT_CONTAINER_MOUNTPOINT:?}");

    // write LUKS password into the LUKS container, so one admin could save it later on
    let password_filepath = Path::new(FSTOOL_DEFAULT_CONTAINER_MOUNTPOINT).join("luks_password");
    std::fs::write(&password_filepath, password.as_bytes()).map_err(|e| {
        Error::Unexpected(format!(
            "unable to save LUKS password in {password_filepath:?}: {e}"
        ))
    })?;

    Ok(())
}
