use actix_web::{
    get, post,
    web::{Data, Json, Path, Query},
};
use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore as _, Aes256Gcm, KeyInit as _,
};
use cosmian_vm_client::{
    client::{ConfParam, QuoteParam},
    snapshot::{CosmianVmSnapshot, SnapshotFiles, SnapshotFilesEntry},
};
use ima::ima::{read_ima_ascii, read_ima_binary, Ima};
use sha1::digest::generic_array::GenericArray;
use std::process::Command;
use tee_attestation::{forge_report_data_with_nonce, get_measurement, get_quote};
use walkdir::WalkDir;

use crate::{
    error::{Error, ResponseWithError},
    utils::{filter_whilelist, hash_file},
    CosmianVmAgent,
};

const ROOT_PATH: &str = "/";

/// Get the IMA hashes list (ASCII format)
///
/// Note: require root privileges
#[get("/ima/ascii")]
pub async fn get_ima_ascii() -> ResponseWithError<Json<String>> {
    Ok(Json(read_ima_ascii()?))
}

/// Get the IMA hashes list (Binary format)
///
/// Note: require root privileges
#[get("/ima/binary")]
pub async fn get_ima_binary() -> ResponseWithError<Json<Vec<u8>>> {
    Ok(Json(read_ima_binary()?))
}

/// Snapshot the system.
///
/// Return the list of all files hashes and all IMA hashes
///
/// Remark: suboptimal => the connection holds during the hashing process
///
/// Note: require root privileges
#[get("/snapshot")]
pub async fn get_snapshot() -> ResponseWithError<Json<CosmianVmSnapshot>> {
    let ima_ascii = read_ima_ascii()?;
    let ima_ascii: &str = ima_ascii.as_ref();
    let ima = Ima::try_from(ima_ascii)?;

    // We use the same hash method than the IMA used
    let hash_method = ima.hash_file_method();

    // Create the snapshotfiles with files contains in the IMA list
    let mut filehashes = SnapshotFiles(
        ima.entries
            .iter()
            .map(|item| SnapshotFilesEntry {
                hash: item.filedata_hash.clone(),
                path: item.filename_hint.clone(),
            })
            .collect(),
    );

    // Add to the snapshotfiles all the file on the system
    for file in WalkDir::new(ROOT_PATH)
        .into_iter()
        .filter_entry(filter_whilelist)
        .filter_map(|file| file.ok())
    {
        // Only keeps files
        if !file.file_type().is_file() {
            continue;
        }

        filehashes.0.insert(SnapshotFilesEntry {
            hash: hash_file(file.path(), &hash_method)?,
            path: file.path().display().to_string(),
        });
    }

    // Get the measurement of the tee (the report data does not matter)
    let quote = get_quote(&[])?;
    let measurement = get_measurement(&quote)?;

    Ok(Json(CosmianVmSnapshot {
        filehashes,
        measurement,
    }))
}

/// Return the #id PCR value
///
/// TODO: remove that endpoint when the `/quote/tpm` will be implemented`
///
/// Note: require root privileges
#[get("/tmp_endpoint/pcr/{id}")]
pub async fn get_pcr_value(path: Path<u32>) -> ResponseWithError<Json<String>> {
    let pcr_id = path.into_inner();

    let output = Command::new("/root/go/bin/gotpm")
        .arg("read")
        .arg("pcr")
        .arg("--pcrs")
        .arg(pcr_id.to_string())
        .arg("--hash-algo")
        .arg("sha1")
        .output()?;

    if !output.status.success() {
        return Err(Error::CommandError(
            format!(
                "Command returns an error (code: {}): , {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )
            .to_string(),
        ));
    }

    // Example of output:
    //SHA1:
    //  10: 0x3CF23C475157764A6CD0B17EDA92F75C5C3F9FBB
    let output = String::from_utf8_lossy(&output.stdout);
    let output = output.lines().last();

    if let Some(output) = output {
        return Ok(Json(output[(output.len() - 40)..].to_owned()));
    }

    Err(Error::CommandError("Can't parse GOTPM output".to_string()))
}

/// Return the TEE quote
#[get("/quote/tee")]
pub async fn get_tee_quote(
    data: Query<QuoteParam>,
    certificate: Data<CosmianVmAgent>,
) -> ResponseWithError<Json<Vec<u8>>> {
    let nonce = hex::decode(&data.nonce)?;
    let report_data = forge_report_data_with_nonce(
        &nonce.try_into().map_err(|_| {
            Error::BadRequest("Nonce should be a 32 bytes string (hex encoded)".to_string())
        })?,
        certificate.pem_certificate.as_bytes(),
    )?;
    let quote = get_quote(&report_data)?;
    Ok(Json(quote))
}

/// Return the TPM quote
#[get("/quote/tpm")]
pub async fn get_tpm_quote(_data: Query<QuoteParam>) -> ResponseWithError<Json<Vec<u8>>> {
    // TODO
    Ok(Json(vec![]))
}

/// Provision the agent configuration
#[post("/init")]
pub async fn init_agent(data: Query<ConfParam>) -> ResponseWithError<Json<Option<Vec<u8>>>> {
    let cfg = data.into_inner();

    // encrypt app conf (if some) and write it to disk
    let key = if let Some(app_cfg) = &cfg.application {
        let (cipher, key) = if let Some(wrap_key) = &app_cfg.wrap_key {
            // key is provided, no need to return the key to the user
            (Aes256Gcm::new(GenericArray::from_slice(wrap_key)), None)
        } else {
            // key generation is needed, the new key is then returned to the user
            let key = Aes256Gcm::generate_key(OsRng);
            (Aes256Gcm::new(&key), Some(key.to_vec()))
        };

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, app_cfg.content.as_ref())
            .map_err(|e| Error::CommandError(format!("cannot encrypt app configuration: {e}")))?;

        // write nonce
        let mut nonce_path = cfg.agent.tmpfs.clone();
        nonce_path.push("nonce");
        std::fs::write(&nonce_path, nonce)?;

        // write encrypted conf
        let mut enc_app_path = cfg.agent.tmpfs.clone();
        enc_app_path.push("app.cfg.enc");
        std::fs::write(enc_app_path, ciphertext)?;

        // copy conf to encrypted tmpfs
        std::fs::write(&app_cfg.deployed_filepath, &app_cfg.content)?;

        key
    } else {
        None
    };

    // start app binary
    Command::new(&cfg.agent.app_binary).spawn()?;

    Ok(Json(key))
}

/// Restart a configured agent (after a reboot for example)
#[post("/restart")]
pub async fn restart_agent(data: Query<ConfParam>) -> ResponseWithError<Json<()>> {
    let cfg = data.into_inner();

    // TODO check it is not already start?

    if let Some(app_cfg) = &cfg.application {
        // decrypt app conf and copy it to tmpfs

        let key = &app_cfg
            .wrap_key
            .as_deref()
            .map(GenericArray::from_slice)
            .ok_or_else(|| {
                Error::BadRequest("No key provided to decrypt app configuration".to_string())
            })?;

        // read nonce
        let mut nonce_path = cfg.agent.tmpfs.clone();
        nonce_path.push("nonce");
        let nonce = std::fs::read(nonce_path)?;
        let nonce = GenericArray::from_slice(&nonce);

        // read encrypted conf
        let mut enc_app_path = cfg.agent.tmpfs.clone();
        enc_app_path.push("app.cfg.enc");
        let ciphertext = std::fs::read(enc_app_path)?;

        // decrypt conf
        let cipher = Aes256Gcm::new(key);
        let app_cfg_content = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| Error::CommandError(format!("cannot decrypt app configuration: {e}")))?;

        // copy conf to encrypted tmpfs
        std::fs::write(&app_cfg.deployed_filepath, app_cfg_content)?;
    }

    // start app binary
    Command::new(cfg.agent.app_binary).spawn()?;

    Ok(Json(()))
}
