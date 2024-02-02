use crate::{conf::CosmianVmAgent, error::Error, init::certificate::generate_self_signed_cert};

use self::{luks::generate_encrypted_fs, tpm::generate_tpm_keys};
use gethostname::gethostname;

mod certificate;
mod luks;
mod tpm;

const TLS_DAYS_BEFORE_EXPIRATION: u64 = 365 * 10;

pub fn initialize_agent(conf: &CosmianVmAgent) -> Result<(), Error> {
    // Generate the encrypted fs
    generate_encrypted_fs(&conf.agent.data_storage)?;

    // Generate the certificate if not present
    let (ssl_private_key, ssl_certificate) = (conf.ssl_private_key(), conf.ssl_certificate());

    match (ssl_private_key.exists(), ssl_certificate.exists()) {
        (false, false) => {
            tracing::info!("Generating default certificates...");
            let hostname = gethostname();
            let hostname = hostname.to_string_lossy();
            let subject = format!("CN={hostname},O=Cosmian Tech,C=FR,L=Paris,ST=Ile-de-France");
            let (sk, cert) = generate_self_signed_cert(
                &subject,
                &[&conf.agent.host],
                TLS_DAYS_BEFORE_EXPIRATION,
            )?;

            std::fs::write(&ssl_certificate, cert)?;
            std::fs::write(&ssl_private_key, sk)?;

            tracing::info!("The certificate has been generated for CN='{hostname}' (days before expiration: {TLS_DAYS_BEFORE_EXPIRATION}) at: {ssl_certificate:?}");
        }
        (true, true) => tracing::info!("The certificate has been read from {ssl_certificate:?}"),
        (false, true) => {
            return Err(Error::Certificate(
                "The private key file doesn't exist whereas the certificate exists".to_owned(),
            ));
        }
        (true, false) => {
            return Err(Error::Certificate(
                "The certificate file doesn't exist whereas the private key exists".to_owned(),
            ));
        }
    };

    // Generate TPM keys if not already done
    if let Some(tpm_device) = &conf.agent.tpm_device {
        generate_tpm_keys(tpm_device)?;
    } else {
        tracing::warn!("No TPM configuration found: TPM generation keys skipped!");
        tracing::warn!(
            "The agent is not configured to support TPM and files integrity verification"
        );
    }

    Ok(())
}
