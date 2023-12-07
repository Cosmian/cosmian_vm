use std::path::PathBuf;

use actix_cors::Cors;
use actix_http::Method;
use actix_web::{
    dev::Service as _,
    web::{scope, Data, PayloadConfig, ServiceConfig},
};
use serde::Deserialize;

static AGENT_CONF: &str = "/etc/cosmian_vm/agent.toml";

#[derive(Deserialize)]
pub struct CosmianVmAgent {
    agent: Agent,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Agent {
    /// Name of the Linux service (ie: nginx)
    pub service: String,
    /// Certificate of the VM in PEM format
    #[serde(with = "pem_reader")]
    pub pem_certificate: String,
    /// Encrypted data storage (ie: tmpfs)
    pub encrypted_store: PathBuf,
    /// Where the app conf is stored encrypted
    pub secret_app_conf: PathBuf,
}

mod pem_reader {
    use serde::{Deserialize as _, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let pem_content = std::fs::read_to_string(s).map_err(serde::de::Error::custom)?;
        Ok(pem_content)
    }
}

pub mod endpoints;
pub mod error;
pub mod utils;

pub fn endpoints(cfg: &mut ServiceConfig) {
    cfg.service(endpoints::get_ima_ascii);
    cfg.service(endpoints::get_ima_binary);
    cfg.service(endpoints::get_pcr_value);
    cfg.service(endpoints::get_snapshot);
    cfg.service(endpoints::get_tee_quote);
    cfg.service(endpoints::get_tpm_quote);
}

pub fn config() -> impl FnOnce(&mut ServiceConfig) {
    let conf: CosmianVmAgent = toml::from_str(
        &std::fs::read_to_string(AGENT_CONF)
            .unwrap_or_else(|_| panic!("cannot read agent conf at: `{AGENT_CONF:?}`")),
    )
    .expect("failed to parse agent configuration as a valid toml file");

    move |cfg: &mut ServiceConfig| {
        cfg.app_data(PayloadConfig::new(10_000_000_000))
            .app_data(Data::new(conf))
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
                    .configure(endpoints)
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::CosmianVmAgent;

    #[test]
    fn test_agent_toml() {
        let cfg_str = r#"
        [agent]
        service = "cosmian_kms"
        pem_certificate = "../../tests/data/cert.pem"
        encrypted_store = "/mnt/cosmian_vm/data"
        secret_app_conf = "/mnt/cosmian_vm/app.json"
        "#;

        let config: CosmianVmAgent = toml::from_str(cfg_str).unwrap();
        // test that the content of PEM cert is read
        assert!(config
            .agent
            .pem_certificate
            .starts_with("-----BEGIN PRIVATE KEY"));
    }
}
