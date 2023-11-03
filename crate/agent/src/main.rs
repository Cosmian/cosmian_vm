use actix_web::{App, HttpServer};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{
    filter::{filter_fn, EnvFilter, FilterExt, LevelFilter},
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Init logging
    init_logging();

    tracing::info!("Starting server...");
    // Start REST server thread
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(cosmian_vm_agent::config())
    })
    .bind((
        "0.0.0.0",
        std::env::var("BACKEND_PORT").map_or_else(
            |_| 8080,
            |p| p.parse::<u16>().expect("bad BACKEND_PORT value"),
        ),
    ))?
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
