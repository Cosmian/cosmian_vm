use std::{fs::File, path::Path, sync::Arc};

use actix_cors::Cors;
use actix_http::Method;
use actix_web::{
    dev::Service as _,
    web::{scope, Data, PayloadConfig, ServiceConfig},
};
use actix_web_lab::middleware::from_fn;
use std::sync::Mutex;

use conf::CosmianVmAgent;
use const_format::formatcp;
use error::Error;
use rustls::ServerConfig;
use user_agent::check_user_agent_middleware;
use utils::create_tpm_context;
use worker::snapshot::Snapshot;

/// Related to the applications running inside the Cosmian VM
pub mod app;
/// Try to detect cloud provider
pub mod cloud_detection;
pub mod conf;
pub mod endpoints;
pub mod error;
/// Related to tasks to process at the first Cosmian VM start
pub mod init;
pub mod user_agent;
pub mod utils;
/// Workers processing async tasks
pub mod worker;

const DEFAULT_TPM_HASH_METHOD: tpm_quote::PcrHashMethod = tpm_quote::PcrHashMethod::Sha256;

pub const BIN_PATH: &str = "/usr/sbin/";
pub const VAR_PATH: &str = "/var/lib/cosmian_vm";
pub const ETC_PATH: &str = "/etc/cosmian_vm";
pub const CONF_PATH: &str = formatcp!("{ETC_PATH}/agent.toml");

pub fn endpoints(cfg: &mut ServiceConfig) {
    cfg.service(endpoints::delete_snapshot);
    cfg.service(endpoints::get_ima_ascii);
    cfg.service(endpoints::get_ima_binary);
    cfg.service(endpoints::get_snapshot);
    cfg.service(endpoints::get_tee_quote);
    cfg.service(endpoints::get_tpm_quote);
    cfg.service(endpoints::init_app);
    cfg.service(endpoints::restart_app);
}

pub fn config(
    conf: CosmianVmAgent,
    snapshot_worker: Arc<Snapshot>,
) -> impl FnOnce(&mut ServiceConfig) {
    let certificate = conf
        .read_leaf_certificate()
        .expect("TLS certificate malformed (PEM expecting)");

    let tpm_context =
        Mutex::new(conf.agent.tpm_device.as_ref().map(|tpm_device| {
            create_tpm_context(tpm_device).expect("Fail to build the TPM context")
        }));

    move |cfg: &mut ServiceConfig| {
        cfg.app_data(PayloadConfig::new(10_000_000_000))
            .app_data(Data::from(Arc::clone(&snapshot_worker)))
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
                                        );
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
    let mut cert_reader = std::io::BufReader::new(File::open(certificate).map_err(|e| {
        Error::Certificate(format!("Unable to read cert file {certificate:?}: {e}"))
    })?);
    let mut sk_reader = std::io::BufReader::new(File::open(private_key).map_err(|e| {
        Error::Certificate(format!(
            "Unable to read private key of cert file {private_key:?}: {e}"
        ))
    })?);

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

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use crate::get_tls_config;

    #[test]
    fn test_cert_key_path_error() {
        let invalid_cert = Path::new("/some/invalid/path/cert.pem");
        let invalid_private_key = Path::new("/some/invalid/path/key.pem");

        // both invalid
        let e = get_tls_config(invalid_cert, invalid_private_key).unwrap_err();
        assert_eq!(e.to_string(), "Unable to read cert file \"/some/invalid/path/cert.pem\": No such file or directory (os error 2)");

        let tmp_dir = env::temp_dir();
        std::fs::File::create(tmp_dir.join("cert.pem")).unwrap();

        // only key invalid
        let e = get_tls_config(&tmp_dir.join("cert.pem"), invalid_private_key).unwrap_err();
        assert_eq!(e.to_string(), "Unable to read private key of cert file \"/some/invalid/path/key.pem\": No such file or directory (os error 2)");

        std::fs::File::create(tmp_dir.join("key.pem")).unwrap();

        // all good (but files are invalid, hence `TLS private key not found!` within the key file)
        let e = get_tls_config(&tmp_dir.join("cert.pem"), &tmp_dir.join("key.pem")).unwrap_err();
        assert_eq!(e.to_string(), "TLS private key not found!");
    }
}
