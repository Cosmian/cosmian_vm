use anyhow::Result;
use gethostname::gethostname;

use actix_web::{App, HttpServer};
use cosmian_vm_agent::{
    conf::CosmianVmAgent,
    get_tls_config,
    utils::{generate_self_signed_cert, generate_tpm_keys},
};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{
    filter::{filter_fn, EnvFilter, FilterExt, LevelFilter},
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

const AGENT_CONF: &str = "/etc/cosmian_vm/agent.toml";
const TLS_DAYS_BEFORE_EXPIRATION: u64 = 365 * 10;

#[actix_web::main]
async fn main() -> Result<()> {
    init_logging();

    tracing::info!(
        "Cosmain VM Agent version {} (TEE detected: {})",
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
    let ssl_private_key = conf.agent.ssl_private_key.clone();
    let ssl_certificate = conf.agent.ssl_certificate.clone();

    // First startup: initialize the agent
    initialize_agent(&conf)?;

    // Start REST server thread
    tracing::info!("Starting server on {host}:{port}...");
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(cosmian_vm_agent::config(conf.clone()))
    })
    .bind_rustls(
        format!("{host}:{port}"),
        get_tls_config(&ssl_certificate, &ssl_private_key)?,
    )?
    .run()
    .await?;

    Ok(())
}

fn init_logging() {
    let stdout_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy()
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap())
        .add_directive("tokio=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap());

    // filters elements from `tracing_actix_web` (wanted only for telemetry export)
    let filter = filter_fn(|metadata| !metadata.target().starts_with("tracing_actix_web"))
        .and(LevelFilter::DEBUG);

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter))
        .with(stdout_layer)
        .init();
}

fn initialize_agent(conf: &CosmianVmAgent) -> Result<()> {
    let ssl_private_key = &conf.agent.ssl_private_key;
    let ssl_certificate = &conf.agent.ssl_certificate;

    // Generate the certificate if not present
    match (ssl_private_key.exists(), ssl_certificate.exists()) {
        (false, false) => {
            tracing::info!("Generating certificates...");
            let hostname = gethostname();
            let hostname = hostname.to_string_lossy();
            let subject = format!("CN={hostname},O=Cosmian Tech,C=FR,L=Paris,ST=Ile-de-France");
            let (sk, cert) = generate_self_signed_cert(
                &subject,
                &[&conf.agent.host],
                TLS_DAYS_BEFORE_EXPIRATION,
            )?;

            std::fs::write(ssl_certificate, cert)?;
            std::fs::write(ssl_private_key, sk)?;

            tracing::info!("The certificate has been generated for CN='{hostname}' (days before expiration: {TLS_DAYS_BEFORE_EXPIRATION}) at: {ssl_certificate:?}")
        }
        (true, true) => tracing::info!("The certificate has been read from {ssl_certificate:?}"),
        (false, true) => {
            anyhow::bail!("The private key file doesn't exist whereas the certificat exists");
        }
        (true, false) => {
            anyhow::bail!("The certificate file doesn't exist whereas the private key exists");
        }
    };

    // Generate TPM keys if not already done
    if let Some(tpm_device) = &conf.agent.tpm_device {
        generate_tpm_keys(tpm_device)?
    } else {
        tracing::info!("No TPM configuration found: TPM generation keys skipped!");
    }

    Ok(())
}
