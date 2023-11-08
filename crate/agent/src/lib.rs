use std::fs;

use actix_cors::Cors;
use actix_http::Method;
use actix_web::{
    dev::Service as _,
    web::{scope, Data, PayloadConfig, ServiceConfig},
};

pub struct CosmianVmAgent {
    pem_certificate: String,
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
    let agent = CosmianVmAgent {
        pem_certificate: fs::read_to_string(
            std::env::var("COSMIAN_VM_AGENT_CERTIFICATE")
                .expect("Please set the `COSMIAN_VM_AGENT_CERTIFICATE` environment variable."),
        )
        .expect("Can't read the Cosmian VM Agent certificate file"),
    };

    move |cfg: &mut ServiceConfig| {
        cfg.app_data(PayloadConfig::new(10_000_000_000))
            .app_data(Data::new(agent))
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
