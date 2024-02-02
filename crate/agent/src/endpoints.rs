use std::sync::Mutex;

use crate::{
    error::{Error, ResponseWithError},
    worker::snapshot::{self, order_snapshot, reset_snapshot, Snapshot},
    CosmianVmAgent, DEFAULT_TPM_HASH_METHOD,
};
use actix_web::{
    delete, get, post,
    web::{Data, Json, Query},
    HttpResponse,
};

use cosmian_vm_client::{
    client::{AppConf, QuoteParam, TpmQuoteResponse},
    snapshot::CosmianVmSnapshot,
};
use ima::ima::{read_ima_ascii, read_ima_ascii_first_line, read_ima_binary, Ima};
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
        Ok(Some(snapshot)) => Ok(HttpResponse::Ok().json(Some(snapshot))),
        Ok(None) => {
            order_snapshot(&snapshot_worker)?;
            Ok(HttpResponse::Accepted().json(None::<CosmianVmSnapshot>))
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
) -> ResponseWithError<Json<()>> {
    let app_conf_param = data.into_inner();

    let Some(app_conf_agent) = &conf.app else {
        return Err(Error::BadRequest(
            "No app section provided in Cosmian VM Agent configuration file".to_string(),
        ));
    };

    let Some(app_storage) = &conf.app_storage() else {
        return Err(Error::BadRequest(
            "No app section provided in Cosmian VM Agent configuration file".to_string(),
        ));
    };

    // write app conf
    std::fs::write(app_storage.join(APP_CONF_FILENAME), app_conf_param.content)?;

    // start app service
    app_conf_agent
        .service_type
        .start(&app_conf_agent.service_name)?;

    Ok(Json(()))
}

/// Restart a configured app (after a reboot for example).
///
/// Stop the service, decrypt and copy app conf, start the service.
#[post("/app/restart")]
pub async fn restart_app(conf: Data<CosmianVmAgent>) -> ResponseWithError<Json<()>> {
    let Some(app_conf_agent) = &conf.app else {
        // No app configuration provided
        return Ok(Json(()));
    };

    // ensure app service is stopped
    app_conf_agent
        .service_type
        .stop(&app_conf_agent.service_name)?;

    // start app service
    app_conf_agent
        .service_type
        .start(&app_conf_agent.service_name)?;

    Ok(Json(()))
}
