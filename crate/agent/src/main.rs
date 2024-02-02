use anyhow::Result;
use cosmian_vm_agent::init::initialize_agent;
use cosmian_vm_agent::worker::snapshot;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use cosmian_vm_agent::{conf::CosmianVmAgent, get_tls_config, CONF_PATH};
use env_logger::{Builder, Target};

#[actix_web::main]
async fn main() -> Result<()> {
    let mut builder = Builder::from_env(env_logger::Env::new().default_filter_or("info"));
    builder.target(Target::Stdout);
    builder.try_init()?;

    tracing::info!(
        "Cosmian VM Agent version {} (TEE detected: {})",
        option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
        tee_attestation::guess_tee()
            .ok()
            .map(|tee| tee.to_string())
            .unwrap_or("unknown".to_string())
    );

    // Read the configuration of the Cosmian VM Agent
    let conf: CosmianVmAgent = toml::from_str(
        &std::fs::read_to_string(
            std::env::var("COSMIAN_VM_AGENT_CONF").unwrap_or(CONF_PATH.to_string()),
        )
        .map_err(|e| anyhow::anyhow!("Cannot read agent conf at: `{CONF_PATH:?}: {e:?}`"))?,
    )
    .map_err(|e| {
        anyhow::anyhow!("Failed to parse agent configuration as a valid toml file: {e:?}`")
    })?;

    let host = conf.agent.host.clone();
    let port = conf.agent.port;
    let ssl_private_key = conf.agent.ssl_private_key();
    let ssl_certificate = conf.agent.ssl_certificate();

    // First startup: initialize the agent
    // This can be disabled by setting COSMIAN_VM_PREINIT=0 when starting `cosmian_vm_agent`
    if std::env::var("COSMIAN_VM_PREINIT").unwrap_or("1".to_string()) == "1" {
        initialize_agent(&conf)?;
    }

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
