use crate::{error::Error, utils::call};

use rand::{distributions::Alphanumeric, Rng};

use std::path::Path;

const FSTOOL_PATH: &str = "/usr/sbin/cosmian_fstool";
const FSTOOL_DEFAULT_SIZE: &str = "500MB";
const FSTOOL_DEFAULT_CONTAINER_FILE: &str = "container";
const FSTOOL_DEFAULT_CONTAINER_MOUNTPOINT: &str = "data";
const FSTOOL_DEFAULT_CONTAINER_NAME: &str = "cosmian_vm_container";
const FSTOOL_DEFAULT_PASSWORD_LENGTH: usize = 32;

/// Generate a luks container
///
/// If the container already exists: just return `Ok`
///
/// Note: the password of the generated container will be prompted in the log
pub fn generate_encrypted_fs(encrypted_fs_path: &Path) -> Result<(), Error> {
    let container_path = encrypted_fs_path.join(FSTOOL_DEFAULT_CONTAINER_FILE);
    let mountpoint_path = encrypted_fs_path.join(FSTOOL_DEFAULT_CONTAINER_MOUNTPOINT);

    if container_path.exists() {
        // Already done: don't proceed further
        return Ok(());
    }

    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(FSTOOL_DEFAULT_PASSWORD_LENGTH)
        .map(char::from)
        .collect();

    tracing::info!("Generating a default encrypted container (password={password})...");
    call(
        &std::env::var("COSMIAN_VM_FSTOOL").unwrap_or(FSTOOL_PATH.to_string()),
        &[
            "--size",
            FSTOOL_DEFAULT_SIZE,
            "--location",
            &container_path.to_string_lossy(),
            "--name",
            FSTOOL_DEFAULT_CONTAINER_NAME,
            "--password",
            &password,
            "--mountpoint",
            &mountpoint_path.to_string_lossy(),
        ],
        false,
    )?;

    tracing::info!("The container has been generated at: {container_path:?} and is mounted at: {mountpoint_path:?}");

    Ok(())
}
