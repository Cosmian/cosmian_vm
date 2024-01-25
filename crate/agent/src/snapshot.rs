use futures::{future, StreamExt};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use actix_web::rt::task::JoinHandle;
use cosmian_vm_client::snapshot::{CosmianVmSnapshot, SnapshotFiles};
use tee_attestation::{get_quote as tee_get_quote, TeePolicy};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tpm_quote::{get_quote as tpm_get_quote, policy::TpmPolicy};

use crate::{error::Error, utils::create_tpm_context, DEFAULT_TPM_HASH_METHOD};

use ima::ima::{read_ima_binary, Ima, ImaHashMethod};

use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

use std::path::Path;
use walkdir::{DirEntry, WalkDir};

const ROOT_PATH: &str = "/";

#[derive(Debug, Default)]
pub struct SnapshotJob {
    // Trigger a new snapshot
    pub trigger: bool,
    // The last snapshot result or None if no snapshot has been process
    pub result: Option<Result<CosmianVmSnapshot, Error>>,
}

pub type Snapshot = Mutex<SnapshotJob>;

/// Create the worker dedicated to the snapshotting of the Cosmian VM
pub fn init_snapshot_worker(
    tpm_device: Option<PathBuf>,
) -> (Arc<Snapshot>, JoinHandle<()>, CancellationToken) {
    // construct empty snapshot
    let snapshot = Arc::new(Snapshot::default());

    // stop signal for snapshot worker
    let snapshot_cancel = CancellationToken::new();

    // spawn snapshot worker
    (
        Arc::clone(&snapshot),
        actix_web::rt::spawn(process_snapshot_orders(
            Arc::clone(&snapshot),
            snapshot_cancel.clone(),
            tpm_device.clone(),
        )),
        snapshot_cancel,
    )
}

/// Get the snapshot if it exists or None otherwise
///
/// Return `Error::SnapshotIsProcessing` if the snapshot is processing
pub(crate) fn get_snapshot(snapshot: &Snapshot) -> Result<Option<CosmianVmSnapshot>, Error> {
    if let Ok(snapshot) = snapshot.try_lock() {
        return match &snapshot.result {
            Some(Ok(result)) => Ok(Some(result.clone())),
            Some(Err(e)) => Err(Error::Unexpected(e.to_string())),
            None => Ok(None),
        };
    }

    Err(Error::SnapshotIsProcessing)
}

/// Clear the snapshot if it exists
///
/// Return `Error::SnapshotIsProcessing` if the snapshot is processing
pub(crate) fn reset_snapshot(snapshot: &Snapshot) -> Result<(), Error> {
    if let Ok(mut snapshot) = snapshot.try_lock() {
        if snapshot.trigger {
            return Err(Error::SnapshotIsProcessing);
        }
        snapshot.result = None;
        return Ok(());
    }

    Err(Error::SnapshotIsProcessing)
}

/// Order a snapshot
///
/// Return `Error::SnapshotIsProcessing` if the snapshot is processing
pub(crate) fn order_snapshot(snapshot: &Snapshot) -> Result<(), Error> {
    if let Ok(mut snapshot) = snapshot.try_lock() {
        snapshot.trigger = true;
        return Ok(());
    }

    Err(Error::SnapshotIsProcessing)
}

/// Wait for order to snapshot the Cosmian VM until `stop_signal` cancels the worker
async fn process_snapshot_orders(
    snapshot: Arc<Snapshot>,
    stop_signal: CancellationToken,
    tpm_device: Option<PathBuf>,
) {
    let mut interval = actix_web::rt::time::interval(Duration::from_secs(10));

    tokio::select! {
        _ = async move {
            loop {
                interval.tick().await;

                // only _try_ to lock so reads and writes from route handlers do not get blocked
                if let Ok(mut snapshot) = snapshot.try_lock() {
                    if snapshot.trigger {
                        tracing::info!("Processing a snapshot...");
                        let start = Instant::now();
                        snapshot.trigger = false;
                        snapshot.result = Some(do_snapshot(tpm_device.clone()).await);
                        let duration = start.elapsed();
                        tracing::info!("Snapshot proceed in {duration:?}");
                    }
                }
            }
        } => {}

        _ = stop_signal.cancelled() => {
            tracing::info!("Gracefully shutting down snapshot worker");
        }
    }
}

/// Snapshot the Cosmian VM
async fn do_snapshot(tpm_device: Option<PathBuf>) -> Result<CosmianVmSnapshot, Error> {
    // Get the measurements of the tee (the report data does not matter)
    let tee_quote = tee_get_quote(&[])?;
    let tee_policy = TeePolicy::try_from(tee_quote.as_ref())?;

    let (filehashes, tpm_policy) = match tpm_device {
        None => (None, None),
        Some(tpm_device) => {
            let mut tpm_context = create_tpm_context(&tpm_device)?;

            // Get the policy of the tpm (the nonce and the pcr_list don't matter)
            let (tpm_quote, _, _) =
                tpm_get_quote(&mut tpm_context, &[], None, DEFAULT_TPM_HASH_METHOD)?;
            let tpm_policy = TpmPolicy::try_from(tpm_quote.as_ref())?;

            // Get the IMA hashes
            let ima = read_ima_binary()?;
            let ima: &[u8] = ima.as_ref();
            let ima = Ima::try_from(ima)?;

            // We use the same hash method as the one IMA used
            let hash_method = ima.hash_file_method();

            // Create the snapshot files with files contained in the IMA list
            let mut filehashes = SnapshotFiles(
                ima.entries
                    .iter()
                    .map(|item| (item.filename_hint.clone(), item.filedata_hash.clone()))
                    .collect(),
            );

            // Add to the snapshotfiles all the file on the system
            filehashes.0.extend(hash_filesystem(&hash_method).await?);

            (Some(filehashes), Some(tpm_policy))
        }
    };

    Ok(CosmianVmSnapshot {
        tee_policy,
        tpm_policy,
        filehashes,
    })
}

#[inline(always)]
pub(crate) async fn hash_file(path: &Path, hash_method: &ImaHashMethod) -> Result<Vec<u8>, Error> {
    match hash_method {
        ImaHashMethod::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(tokio::fs::read(path).await?);
            Ok(hasher.finalize().to_vec())
        }
        ImaHashMethod::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(tokio::fs::read(path).await?);
            Ok(hasher.finalize().to_vec())
        }
        ImaHashMethod::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(tokio::fs::read(path).await?);
            Ok(hasher.finalize().to_vec())
        }
    }
}

#[inline(always)]
pub async fn hash_filesystem(hash_method: &ImaHashMethod) -> Result<Vec<(String, Vec<u8>)>, Error> {
    // Collect the files first
    // We store all the files in memory. It's a tradeoff to then quickly hash in parallel all the files
    // Listing the files is pretty quick: negligeable against hashing the files
    let files: Vec<_> = WalkDir::new(ROOT_PATH)
        .into_iter()
        .filter_entry(filter_whilelist)
        .filter_map(std::result::Result::ok)
        // Only keeps files
        .filter(|file| file.file_type().is_file())
        .map(|file| file.into_path())
        .collect();

    // Create threads to compute the hash in parallel
    // Note: processing like that doesn't block the main thread when stopping
    Ok(futures::stream::iter(files.into_iter())
        .map(|file| {
            let hash_method2 = hash_method.clone();
            async move {
                hash_file(&file, &hash_method2)
                    .await
                    .ok() // We ignore file if the hashing fails
                    .map(|hash| (file.display().to_string(), hash))
            }
        })
        .buffer_unordered(num_cpus::get()) // Run up to X concurrently
        .filter_map(future::ready)
        .collect::<Vec<_>>()
        .await)
}

const BASE_EXCLUDE_DIRS: [&str; 8] = [
    "/sys/",
    "/run/",
    "/proc/",
    "/lost+found/",
    "/dev/",
    "/media/",
    "/var/",
    "/tmp/",
];

#[must_use]
pub fn filter_whilelist(entry: &DirEntry) -> bool {
    // Do not keep files in some folders
    !BASE_EXCLUDE_DIRS
        .iter()
        .any(|exclude_dir| entry.path().starts_with(exclude_dir))
}
