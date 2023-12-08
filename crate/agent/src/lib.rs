use actix_cors::Cors;
use actix_http::Method;
use actix_web::{
    dev::Service as _,
    web::{scope, Data, PayloadConfig, ServiceConfig},
};
use conf::CosmianVmAgent;

static AGENT_CONF: &str = "/etc/cosmian_vm/agent.toml";

pub mod conf;
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
    cfg.service(endpoints::init_app);
    cfg.service(endpoints::restart_app);
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
