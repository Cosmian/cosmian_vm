use anyhow::Result;
use cosmian_vm_agent::snapshot;
use cosmian_vm_agent::utils::generate_encrypted_fs;
use gethostname::gethostname;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use cosmian_vm_agent::{
    conf::CosmianVmAgent,
    get_tls_config,
    utils::{generate_self_signed_cert, generate_tpm_keys},
};

const AGENT_CONF: &str = "/etc/cosmian_vm/agent.toml";
const TLS_DAYS_BEFORE_EXPIRATION: u64 = 365 * 10;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    tracing::info!(
        "Cosmian VM Agent version {} (TEE detected: {})",
        option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        tee_attestation::guess_tee()?.to_string()
    );

    // Read the configuration of the Cosmian VM Agent
    let conf: CosmianVmAgent = toml::from_str(
        &std::fs::read_to_string(
            std::env::var("COSMIAN_VM_AGENT_CONF").unwrap_or(AGENT_CONF.to_string()),
        )
        .map_err(|e| anyhow::anyhow!("Cannot read agent conf at: `{AGENT_CONF:?}: {e:?}`"))?,
    )
    .map_err(|e| {
        anyhow::anyhow!("Failed to parse agent configuration as a valid toml file: {e:?}`")
    })?;

    let host = conf.agent.host.clone();
    let port = conf.agent.port;
    let ssl_private_key = conf.agent.ssl_private_key();
    let ssl_certificate = conf.agent.ssl_certificate();

    // First startup: initialize the agent
    initialize_agent(&conf)?;

    // Background worker relating to the snapshot processing
    tracing::info!("Starting the snapshot worker...");
    let (snapshot_worker, snapshot_worker_handle, snapshot_worker_cancel) =
        snapshot::init_snapshot_worker(conf.agent.tpm_device.clone());

    // Start REST server thread
    tracing::info!("Starting Cosmian VM Agent on {host}:{port}...");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(cosmian_vm_agent::config(
                conf.clone(),
                snapshot_worker.clone(),
            ))
    })
    .bind_rustls(
        format!("{host}:{port}"),
        get_tls_config(&ssl_certificate, &ssl_private_key)?,
    )?
    .run()
    .await?;

    tracing::info!("Stopping the snapshot worker...");
    // signal snapshot worker to stop running
    snapshot_worker_cancel.cancel();

    // wait for the snapshot worker to exit its loop gracefully
    snapshot_worker_handle.await?;

    tracing::info!("Cosmian VM Agent successfully shutdown gracefully");
    Ok(())
}

fn initialize_agent(conf: &CosmianVmAgent) -> Result<()> {
    // Generate the encrypted fs
    generate_encrypted_fs(&conf.agent.data_storage)?;

    // Generate the certificate if not present
    let (ssl_private_key, ssl_certificate) =
        (conf.agent.ssl_private_key(), conf.agent.ssl_certificate());

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
            anyhow::bail!("The private key file doesn't exist whereas the certificate exists");
        }
        (true, false) => {
            anyhow::bail!("The certificate file doesn't exist whereas the private key exists");
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
