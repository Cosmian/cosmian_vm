use crate::error::Error;
use der::{asn1::Ia5String, pem::LineEnding, EncodePem};
use p256::{ecdsa::DerSignature, ecdsa::SigningKey, pkcs8::EncodePrivateKey, SecretKey};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

use spki::{EncodePublicKey, SubjectPublicKeyInfoOwned};
use std::{
    convert::TryFrom,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    time::Duration,
};
use x509_cert::{
    builder::{Builder, CertificateBuilder, Profile},
    ext::pkix::{name::GeneralName, BasicConstraints, SubjectAltName},
    name::Name,
    serial_number::SerialNumber,
    time::Validity,
};

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
                    Ia5String::try_from((*san).to_string())
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
