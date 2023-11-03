use crate::{
    core::{filter_whilelist, hash_file, parse_ima_ascii, read_ima_ascii, read_ima_binary},
    errors::{Error, ResponseWithError},
};
use actix_web::{
    get,
    web::{Json, Path, Query},
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tee_attestation::get_quote;
use walkdir::WalkDir;

const ROOT_PATH: &str = "/";

/// Get the IMA hashes list (ASCII format)
///
/// Note: required root privileges
#[get("/ima/ascii")]
pub async fn get_ima_ascii() -> ResponseWithError<Json<String>> {
    Ok(Json(read_ima_ascii()?))
}

/// Get the IMA hashes list (Binary format)
///
/// Note: required root privileges
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
/// Note: required root privileges
#[get("/snapshot")]
pub async fn get_snapshot() -> ResponseWithError<Json<String>> {
    let ima_ascii = read_ima_ascii()?;
    let ima = parse_ima_ascii(&ima_ascii)?;

    // TODO: create a whitelist structure

    let mut filehashes: Vec<String> = ima
        .iter()
        .map(|item| format!(r"{}\f{}", item.filedata_hash, item.filename_hint))
        .collect();

    for file in WalkDir::new(ROOT_PATH)
        .into_iter()
        .filter_entry(filter_whilelist)
        .filter_map(|file| file.ok())
    {
        // Only keeps files
        if !file.file_type().is_file() {
            continue;
        }

        filehashes.push(format!(
            r"{}\f{}",
            hex::encode(hash_file(file.path())?),
            file.path().display()
        ));
    }

    Ok(Json(filehashes.join("\n")))
}

/// Return the #id PCR value
///
/// TODO: remove that endpoint when the `/quote/tpm` will be implemented`
///
/// Note: required root privileges
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

#[derive(Serialize, Deserialize)]
pub struct QuoteParam {
    pub nonce: String,
}

/// Return the TEE quote
#[get("/quote/tee")]
pub async fn get_tee_quote(data: Query<QuoteParam>) -> ResponseWithError<Json<Vec<u8>>> {
    let nonce = hex::decode(&data.nonce)?;
    let quote = get_quote(&nonce)?;
    Ok(Json(quote))
}

/// Return the TPM quote
#[get("/quote/tpm")]
pub async fn get_tpm_quote(_data: Query<QuoteParam>) -> ResponseWithError<Json<Vec<u8>>> {
    // TODO
    Ok(Json(vec![]))
}
