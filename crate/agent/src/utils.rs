use crate::error::Error;
use der::{asn1::Ia5String, pem::LineEnding, EncodePem};
use ima::ima::ImaHashMethod;
use p256::{ecdsa::DerSignature, ecdsa::SigningKey, pkcs8::EncodePrivateKey, SecretKey};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};
use spki::{EncodePublicKey, SubjectPublicKeyInfoOwned};
use std::process::Command;
use std::{
    convert::TryFrom,
    fs, io,
    net::{IpAddr, Ipv4Addr},
    path::Path,
    str::FromStr,
    time::Duration,
};
use tss_esapi::{Context, TctiNameConf};
use walkdir::DirEntry;
use x509_cert::{
    builder::{Builder, CertificateBuilder, Profile},
    ext::pkix::{name::GeneralName, BasicConstraints, SubjectAltName},
    name::Name,
    serial_number::SerialNumber,
    time::Validity,
};

#[inline(always)]
pub(crate) fn hash_file(path: &Path, hash_method: &ImaHashMethod) -> Result<Vec<u8>, Error> {
    let mut file = fs::File::open(path)?;

    match hash_method {
        ImaHashMethod::Sha1 => {
            let mut hasher = Sha1::new();
            let _ = io::copy(&mut file, &mut hasher)?;
            Ok(hasher.finalize().to_vec())
        }
        ImaHashMethod::Sha256 => {
            let mut hasher = Sha256::new();
            let _ = io::copy(&mut file, &mut hasher)?;
            Ok(hasher.finalize().to_vec())
        }
        ImaHashMethod::Sha512 => {
            let mut hasher = Sha512::new();
            let _ = io::copy(&mut file, &mut hasher)?;
            Ok(hasher.finalize().to_vec())
        }
    }
}

pub fn filter_whilelist(entry: &DirEntry) -> bool {
    _filter_whilelist(entry).unwrap_or(false)
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

pub fn _filter_whilelist(entry: &DirEntry) -> Result<bool, Error> {
    // Do not keep files in some folders
    if BASE_EXCLUDE_DIRS
        .iter()
        .any(|exclude_dir| entry.path().starts_with(exclude_dir))
    {
        return Ok(false);
    }

    Ok(true)
}

/// Generate a self-signed certificate
pub fn generate_self_signed_cert(
    subject: &str,
    subject_alternative_names: &[&str],
    days_before_expiration: u64,
) -> Result<(String, String), Error> {
    let mut rng = ChaCha20Rng::from_entropy();

    let serial_number = SerialNumber::from(rng.next_u32());
    let validity = Validity::from_now(Duration::new(days_before_expiration * 24 * 60 * 60, 0))
        .map_err(|_| Error::Certificate("Unexpected expiration validity".to_owned()))?;

    let subject = Name::from_str(subject)
        .map_err(|_| Error::Certificate("Can't parse subject".to_owned()))?;

    let secret_key = SecretKey::random(&mut rng);

    let pem_sk = secret_key
        .clone()
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|_| Error::Certificate("Can't convert secret key to PEM".to_owned()))?
        .to_string();

    let signer = SigningKey::from(secret_key);
    let pk_der = signer
        .verifying_key()
        .to_public_key_der()
        .map_err(|e| Error::Cryptography(e.to_string()))?;
    let spki = SubjectPublicKeyInfoOwned::try_from(pk_der.as_bytes()).map_err(|e| {
        Error::Certificate(format!(
            "Can't create SubjectPublicKeyInfo from public key: {e:?}"
        ))
    })?;
    let mut builder = CertificateBuilder::new(
        Profile::Manual { issuer: None },
        serial_number,
        validity,
        subject,
        spki,
        &signer,
    )
    .map_err(|_| Error::Certificate("Failed to create certificate builder".to_owned()))?;

    if !subject_alternative_names.is_empty() {
        let subject_alternative_names = subject_alternative_names
            .iter()
            .map(|san| match san.parse::<Ipv4Addr>() {
                Ok(ip) => GeneralName::from(IpAddr::V4(ip)),
                Err(_) => GeneralName::DnsName(
                    Ia5String::try_from(san.to_string())
                        .expect("SAN contains non-ascii characters"),
                ),
            })
            .collect::<Vec<GeneralName>>();

        builder
            .add_extension(&SubjectAltName(subject_alternative_names))
            .map_err(|_| Error::Certificate("Can't create SAN extension".to_owned()))?;
    }

    builder
        .add_extension(&BasicConstraints {
            ca: true,
            path_len_constraint: None,
        })
        .map_err(|_| Error::Certificate("Failed to add basic constraint CA:true".to_owned()))?;

    let certificate = builder
        .build::<DerSignature>()
        .map_err(|_| Error::Certificate("Can't build certificate".to_owned()))?;
    let pem_cert = certificate
        .to_pem(LineEnding::LF)
        .map_err(|_| Error::Certificate("Failed to convert certificate to PEM".to_owned()))?;

    Ok((pem_sk, pem_cert))
}

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

/// Generate the TPM keys during the first startup of the agent
/// - Ignore generation if already done previously
/// - Raise an error if no TPM detected
///
/// Note: this function should be replace in a near feature (waiting for a patch in the tpm lib)
pub fn generate_tpm_keys(tpm_device_path: &Path) -> Result<(), Error> {
    if !tpm_device_path.exists() {
        return Err(Error::Configuration(format!(
            "TPM device path unknown: {tpm_device_path:?} "
        )));
    }

    // Verify the keys has not been already generated
    match Command::new("tpm2_readpublic")
        .args(["-c", "0x81000000"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                return Ok(());
            }
        }
        Err(e) => return Err(Error::Command(e.to_string())),
    }

    // Otherwise generated them
    //
    // # create EK and make it persistent
    // sudo tpm2_createek --ek-context=ek.ctx --key-algorithm=ecc --public=ek.pub --format=pem
    // sudo tpm2_evictcontrol --hierarchy=o --object-context=ek.ctx --output=ek.handle
    // # create AK and make it persistent
    // sudo tpm2_createak --ek-context=ek.handle --ak-context=ak.ctx --key-algorithm=ecc --hash-algorithm=sha256 --public=ak.pub --format pem --ak-name=ak.name
    // sudo tpm2_evictcontrol --hierarchy=o --object-context=ak.ctx --output=ak.handle

    // Create EK and make it persistent
    tracing::info!("Generating TPM EK & AK...");
    call(
        "tpm2_createek",
        &[
            "--ek-context=/tmp/ek.ctx",
            "--key-algorithm=ecc",
            "--public=/tmp/ek.pub",
            "--format=pem",
        ],
        false,
    )?;

    call(
        "tpm2_evictcontrol",
        &[
            "--hierarchy=o",
            "--object-context=/tmp/ek.ctx",
            "--output=/tmp/ek.handle",
        ],
        false,
    )?;

    // Create AK and make it persistent
    call(
        "tpm2_createak",
        &[
            "--ek-context=/tmp/ek.handle",
            "--ak-context=/tmp/ak.ctx",
            "--key-algorithm=ecc",
            "--hash-algorithm=sha256",
            "--public=/tmp/ak.pub",
            "--format=pem",
            "--ak-name=/tmp/ak.name",
        ],
        false,
    )?;

    call(
        "tpm2_evictcontrol",
        &[
            "--hierarchy=o",
            "--object-context=/tmp/ak.ctx",
            "--output=/tmp/ak.handle",
        ],
        false,
    )?;

    Ok(())
}

pub(crate) fn create_tpm_context(tpm_device: &Path) -> Result<Context, Error> {
    let tcti = TctiNameConf::from_str(&format!("device:{}", &tpm_device.to_string_lossy()))
        .map_err(|e| Error::Unexpected(format!("Incorrect TCTI (TPM device): {e}")))?;

    let tpm_context = Context::new(tcti).map_err(|e| {
        Error::Unexpected(format!("Can't build context from TCTI (TPM device): {e}"))
    })?;

    Ok(tpm_context)
}
