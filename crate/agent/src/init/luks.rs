use crate::{error::Error, utils::call, BIN_PATH, VAR_PATH};

use const_format::formatcp;
use rand::{distributions::Alphanumeric, Rng};

use std::path::Path;

const FSTOOL_PATH: &str = formatcp!("{BIN_PATH}/cosmian_fstool");
const FSTOOL_DEFAULT_SIZE: &str = "512MB";
const FSTOOL_DEFAULT_CONTAINER_FILE: &str = formatcp!("{VAR_PATH}/container");
const FSTOOL_DEFAULT_HEADER_FILE: &str = formatcp!("{VAR_PATH}/header");
const FSTOOL_DEFAULT_CONTAINER_MOUNT_POINT: &str = formatcp!("{VAR_PATH}/data");
const FSTOOL_DEFAULT_PASSWORD_LENGTH: usize = 32;

/// Generate a luks container
///
/// Only called before the agent is initialized: any container or header found on disk
/// comes from an interrupted first initialization (crash, reboot, `cosmian_fstool` failure)
/// and holds no user data. It is discarded and regenerated, since it may be unmounted,
/// not enrolled in the TPM, or its password may never have been saved.
///
/// Note: the password of the generated container will be written in the log
pub(crate) fn generate_encrypted_fs() -> Result<(), Error> {
    // `cosmian_fstool` closes and unmounts a previously opened container by itself
    for leftover in [FSTOOL_DEFAULT_CONTAINER_FILE, FSTOOL_DEFAULT_HEADER_FILE] {
        if Path::new(leftover).exists() {
            tracing::warn!("Removing {leftover:?} left by an interrupted initialization");
            std::fs::remove_file(leftover)?;
        }
    }

    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(FSTOOL_DEFAULT_PASSWORD_LENGTH)
        .map(char::from)
        .collect();

    let output = call(
        &std::env::var("COSMIAN_VM_FSTOOL").unwrap_or(FSTOOL_PATH.to_owned()),
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

    if let Some(output) = output {
        tracing::info!("The cosmian_fstool output is: {}", output);
    }

    tracing::info!("The container has been generated at: {FSTOOL_DEFAULT_CONTAINER_FILE:?} and is mounted at: {FSTOOL_DEFAULT_CONTAINER_MOUNT_POINT:?}");

    // write LUKS password into the LUKS container, so one admin could save it later on
    let password_filepath = Path::new(FSTOOL_DEFAULT_CONTAINER_MOUNT_POINT).join("luks_password");
    std::fs::write(&password_filepath, password.as_bytes()).map_err(|e| {
        Error::Unexpected(format!(
            "unable to save LUKS password in {password_filepath:?}: {e}"
        ))
    })?;

    Ok(())
}
