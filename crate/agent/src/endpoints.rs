use crate::{
    conf::{EncryptedAppConf, EncryptedAppConfAlgorithm},
    error::{Error, ResponseWithError},
    service::{Supervisor, UnixService as _},
    utils::{filter_whilelist, hash_file},
    CosmianVmAgent,
};

use actix_web::{
    get, post,
    web::{Data, Json, Path, Query},
};
use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore as _, Aes256Gcm, KeyInit as _, Nonce,
};
use cosmian_vm_client::{
    client::{AppConf, QuoteParam, RestartParam},
    snapshot::{CosmianVmSnapshot, SnapshotFiles},
};
use ima::ima::{read_ima_ascii, read_ima_binary, Ima};
use sha1::digest::generic_array::GenericArray;
use std::process::Command;
use tee_attestation::{forge_report_data_with_nonce, get_quote, TeePolicy};
use walkdir::WalkDir;

const ROOT_PATH: &str = "/";
const APP_CONF_FILENAME: &str = "app.conf";

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
            .map(|item| (item.filename_hint.clone(), item.filedata_hash.clone()))
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

        filehashes.0.insert((
            file.path().display().to_string(),
            hash_file(file.path(), &hash_method)?,
        ));
    }

    // Get the measurement of the tee (the report data does not matter)
    let quote = get_quote(&[])?;
    let policy = TeePolicy::try_from(quote.as_ref())?;

    Ok(Json(CosmianVmSnapshot { filehashes, policy }))
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
        return Err(Error::Command(
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

    Err(Error::Command("Can't parse GOTPM output".to_string()))
}

/// Return the TEE quote
#[get("/quote/tee")]
pub async fn get_tee_quote(
    data: Query<QuoteParam>,
    conf: Data<CosmianVmAgent>,
) -> ResponseWithError<Json<Vec<u8>>> {
    let data = data.into_inner();
    let report_data = forge_report_data_with_nonce(
        &data.nonce.try_into().map_err(|_| {
            Error::BadRequest("Nonce should be a 32 bytes string (hex encoded)".to_string())
        })?,
        std::fs::read_to_string(&conf.agent.ssl_certificate)?.as_bytes(),
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

/// Initialize the application configuration
#[post("/app/init")]
pub async fn init_app(
    data: Json<AppConf>,
    conf: Data<CosmianVmAgent>,
) -> ResponseWithError<Json<Option<Vec<u8>>>> {
    let app_conf_param = data.into_inner();

    let Some(app_conf_agent) = &conf.app else {
        return Err(Error::BadRequest(
            "no app configuration provided".to_string(),
        ));
    };

    let (cipher, key) = if let Some(key) = &app_conf_param.key {
        // key is provided, no need to return the key to the user
        (Aes256Gcm::new(GenericArray::from_slice(key)), None)
    } else {
        // key generation is needed, the new key is then returned to the user
        let key = Aes256Gcm::generate_key(OsRng);
        (Aes256Gcm::new(&key), Some(key.to_vec()))
    };

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, app_conf_param.content.as_ref())
        .map_err(|e| Error::Cryptography(format!("cannot encrypt app conf: {e}")))?;

    let eac = EncryptedAppConf {
        version: "1.0".to_string(),
        algorithm: EncryptedAppConfAlgorithm::Aes256Gcm,
        nonce: nonce.to_vec(),
        data: ciphertext,
    };
    let json = serde_json::to_string(&eac).map_err(Error::Serialization)?;

    // write encrypted app conf to non-encrypted fs
    std::fs::write(&app_conf_agent.encrypted_secret_app_conf, json.as_bytes())?;

    // write plaintext conf to encrypted tmpfs
    std::fs::write(
        app_conf_agent.decrypted_folder.join(APP_CONF_FILENAME),
        app_conf_param.content,
    )?;

    // start app service
    Supervisor::start(&app_conf_agent.service_app_name)?;

    Ok(Json(key))
}

/// Restart a configured app (after a reboot for example).
///
/// Stop the service, decrypt and copy app conf, start the service.
#[post("/app/restart")]
pub async fn restart_app(
    data: Json<RestartParam>,
    conf: Data<CosmianVmAgent>,
) -> ResponseWithError<Json<()>> {
    let cfg = data.into_inner();

    let Some(app_conf_agent) = &conf.app else {
        // No app configuration provided
        return Ok(Json(()));
    };

    // ensure app service is stopped
    Supervisor::stop(&app_conf_agent.service_app_name)?;

    // read app json conf
    let raw_json = std::fs::read_to_string(&app_conf_agent.encrypted_secret_app_conf)?;
    let eac: EncryptedAppConf = serde_json::from_str(&raw_json).map_err(Error::Serialization)?;

    // decrypt conf
    let key = GenericArray::from_slice(&cfg.key);
    let nonce = Nonce::from_slice(&eac.nonce);
    let cipher = Aes256Gcm::new(key);
    let app_cfg_content = cipher
        .decrypt(nonce, eac.data.as_ref())
        .map_err(|e| Error::Cryptography(format!("cannot decrypt app conf: {e}")))?;

    // write decrypted app conf to encrypted tmpfs
    std::fs::write(
        app_conf_agent.decrypted_folder.join(APP_CONF_FILENAME),
        app_cfg_content,
    )?;

    // start app service
    Supervisor::start(&app_conf_agent.service_app_name)?;

    Ok(Json(()))
}
