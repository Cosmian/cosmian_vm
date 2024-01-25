use std::sync::Mutex;

use crate::{
    conf::{EncryptedAppConf, EncryptedAppConfAlgorithm},
    error::{Error, ResponseWithError},
    snapshot::{self, order_snapshot, reset_snapshot, Snapshot},
    CosmianVmAgent, DEFAULT_TPM_HASH_METHOD,
};
use actix_web::{
    delete, get, post,
    web::{Data, Json, Query},
    HttpResponse,
};
use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore as _, Aes256Gcm, KeyInit as _, Nonce,
};
use cosmian_vm_client::{
    client::{AppConf, QuoteParam, RestartParam, TpmQuoteResponse},
    snapshot::CosmianVmSnapshot,
};
use ima::ima::{read_ima_ascii, read_ima_ascii_first_line, read_ima_binary, Ima};
use sha1::digest::generic_array::GenericArray;
use tee_attestation::{forge_report_data_with_nonce, get_quote as tee_get_quote};
use tpm_quote::{error::Error as TpmError, get_quote as tpm_get_quote};

use tss_esapi::Context;

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

/// Get a system snapshot.
///
/// If the snapshot is ready, return it with a HTTP status code `200 OK`
/// If not, start a snapshot and return a HTTP status code `202 Accepted`
///
/// Note: require root privileges
#[get("/snapshot")]
pub async fn get_snapshot(snapshot_worker: Data<Snapshot>) -> ResponseWithError<HttpResponse> {
    match snapshot::get_snapshot(&snapshot_worker) {
        Ok(snapshot) => {
            if let Some(snapshot) = snapshot {
                Ok(HttpResponse::Ok().json(Some(snapshot)))
            } else {
                order_snapshot(&snapshot_worker)?;
                Ok(HttpResponse::Accepted().json(None::<CosmianVmSnapshot>))
            }
        }
        Err(Error::SnapshotIsProcessing) => {
            Ok(HttpResponse::Accepted().json(None::<CosmianVmSnapshot>))
        }
        Err(e) => Err(e),
    }
}

/// Remove the previously computed snapshot.
#[delete("/snapshot")]
pub async fn delete_snapshot(snapshot_worker: Data<Snapshot>) -> ResponseWithError<Json<()>> {
    reset_snapshot(&snapshot_worker)?;
    Ok(Json(()))
}

/// Return the TEE quote
#[get("/quote/tee")]
pub async fn get_tee_quote(
    data: Query<QuoteParam>,
    certificate: Data<Vec<u8>>,
) -> ResponseWithError<Json<Vec<u8>>> {
    let data = data.into_inner();
    let report_data = forge_report_data_with_nonce(
        &data.nonce.try_into().map_err(|_| {
            Error::BadRequest("Nonce should be a 32 bytes string (hex encoded)".to_string())
        })?,
        &certificate,
    )?;
    let quote = tee_get_quote(&report_data)?;
    Ok(Json(quote))
}

/// Return the TPM quote
#[get("/quote/tpm")]
pub async fn get_tpm_quote(
    quote_param: Query<QuoteParam>,
    tpm_context: Data<Mutex<Option<Context>>>,
) -> ResponseWithError<Json<TpmQuoteResponse>> {
    let mut tpm_context = tpm_context
        .lock()
        .map_err(|_| Error::Unexpected("TPM already in use".to_owned()))?;

    let tpm_context = tpm_context.as_mut().ok_or_else(|| {
        Error::Unexpected("The agent is not configured to support TPM".to_string())
    })?;

    let quote_param = quote_param.into_inner();

    if quote_param.nonce.len() > 64 {
        return Err(Error::Tpm(TpmError::AttestationError(
            "Nonce too long (> 64 bytes)".to_owned(),
        )));
    }

    let pcr_slot = Ima::try_from(read_ima_ascii_first_line()?.as_str())?.pcr_id();

    let (quote, signature, public_key) = tpm_get_quote(
        tpm_context,
        &[pcr_slot as u8],
        Some(&quote_param.nonce),
        DEFAULT_TPM_HASH_METHOD,
    )?;

    Ok(Json(TpmQuoteResponse {
        quote,
        signature,
        public_key,
        pcr_value_hash_method: DEFAULT_TPM_HASH_METHOD,
    }))
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
    app_conf_agent
        .service_type
        .start(&app_conf_agent.service_app_name)?;

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
    app_conf_agent
        .service_type
        .stop(&app_conf_agent.service_app_name)?;

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
    app_conf_agent
        .service_type
        .start(&app_conf_agent.service_app_name)?;

    Ok(Json(()))
}
