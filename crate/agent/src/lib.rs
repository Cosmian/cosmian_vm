use std::{fs::File, path::Path, sync::Mutex};

use actix_cors::Cors;
use actix_http::Method;
use actix_web::{
    dev::Service as _,
    web::{scope, Data, PayloadConfig, ServiceConfig},
};
use actix_web_lab::middleware::from_fn;

use conf::CosmianVmAgent;
use error::Error;
use rustls::ServerConfig;
use user_agent::check_user_agent_middleware;
use utils::create_tpm_context;

pub mod conf;
pub mod endpoints;
pub mod error;
pub mod service;
pub mod user_agent;
pub mod utils;

pub fn endpoints(cfg: &mut ServiceConfig) {
    cfg.service(endpoints::get_ima_ascii);
    cfg.service(endpoints::get_ima_binary);
    cfg.service(endpoints::get_snapshot);
    cfg.service(endpoints::get_tee_quote);
    cfg.service(endpoints::get_tpm_quote);
    cfg.service(endpoints::init_app);
    cfg.service(endpoints::restart_app);
}

pub fn config(conf: CosmianVmAgent) -> impl FnOnce(&mut ServiceConfig) {
    let certificate = conf
        .read_leaf_certificate()
        .expect("TLS certificate malformed (PEM expecting)");

    let tpm_context =
        Mutex::new(conf.agent.tpm_device.as_ref().map(|tpm_device| {
            create_tpm_context(tpm_device).expect("Fail to build the TPM context")
        }));

    move |cfg: &mut ServiceConfig| {
        cfg.app_data(PayloadConfig::new(10_000_000_000))
            .app_data(Data::new(conf))
            .app_data(Data::new(certificate))
            .app_data(Data::new(tpm_context))
            .service({
                // cannot call `.wrap()` on the `ServiceConfig` directly, so an empty scope is created for the entire app
                scope("")
                    .wrap(Cors::permissive())
                    .wrap_fn(|mut req, srv| {
                        if req.method() == Method::POST {
                            if let Some(value) = req.headers().get("x-http-method-override") {
                                match Method::from_bytes(value.as_bytes()) {
                                    Ok(method) => req.head_mut().method = method,
                                    Err(err) => {
                                        tracing::warn!(
                                            "Invalid method inside x-http-method-override {err}"
                                        )
                                    }
                                }
                            }
                        }

                        srv.call(req)
                    })
                    .wrap(from_fn(check_user_agent_middleware))
                    .configure(endpoints)
            });
    }
}

/// Create a TLS config builder
pub fn get_tls_config(certificate: &Path, private_key: &Path) -> Result<ServerConfig, Error> {
    let mut cert_reader = std::io::BufReader::new(File::open(certificate)?);
    let mut sk_reader = std::io::BufReader::new(File::open(private_key)?);

    let certificate = rustls_pemfile::certs(&mut cert_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    let key = rustls_pemfile::pkcs8_private_keys(&mut sk_reader)?;

    Ok(ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            certificate,
            rustls::PrivateKey(
                key.first()
                    .ok_or_else(|| Error::Certificate("TLS private key not found!".to_owned()))?
                    .clone(),
            ),
        )?)
}
