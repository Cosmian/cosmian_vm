use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Result};

use clap::Parser;
use cosmian_vm_certtool::generate;

use actix_web::{rt::spawn, App, HttpServer};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Domain name to include
    #[arg(short, long, action)]
    domain: String,

    /// Alternative domain names (not IP) to include
    #[arg(short, long, action)]
    san: Vec<String>,

    /// The email used for registration and recovery contact
    #[arg(short, long, action)]
    email: String,

    /// Location to store temporary work files
    #[arg(short, long, action)]
    workspace: PathBuf,

    /// Location to store the generated certificate and private key
    #[arg(short, long, action)]
    output: PathBuf,

    /// Use staging ACME (less API limitation but insecure certificates: for dev only)
    #[arg(long)]
    staging: bool,

    /// Generate the SSL private key using the TEE. Provide the salt (in hexadecimal) to derive the key
    #[arg(long, required = false)]
    derived_from_tee: Option<String>,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let opts = Cli::parse();

    let (domain, san, email, workspace, output, staging) = (
        opts.domain.clone(),
        opts.san.clone(),
        opts.email.clone(),
        opts.workspace.clone(),
        opts.output.clone(),
        opts.staging,
    );

    let derived_from_tee = if let Some(derived_from_tee) = opts.derived_from_tee {
        Some(hex::decode(derived_from_tee).map_err(|_| {
            anyhow!("Error when decoding hexadecimal value for salt (used in TEE key derivation)")
        })?)
    } else {
        None
    };

    // Start a HTTP server, to negotiate a certificate
    let server = HttpServer::new(move || {
        App::new().service(actix_files::Files::new("/", &opts.workspace).use_hidden_files())
    })
    .workers(1)
    .bind(("0.0.0.0", 80))?
    .run(); // The server is not started yet here!

    let succeed = Arc::new(AtomicBool::new(false));
    let succeed_me = Arc::clone(&succeed);
    let srv = server.handle();

    spawn(async move {
        // Generate the certificate in another thread
        println!("Requesting acme...");
        let san = san.iter().map(String::as_ref).collect::<Vec<&str>>();
        match generate(
            &domain,
            &san,
            &email,
            &workspace,
            &output,
            derived_from_tee.as_deref(),
            staging,
        ) {
            Ok(_) => succeed_me.store(true, Ordering::Relaxed),
            Err(error) => {
                println!("Error when generating the certificate: {error:?}");
                succeed_me.store(false, Ordering::Relaxed);
            }
        }

        // Stop the HTTP server. We don't need it anymore
        srv.stop(true).await;
    });

    // Run server until stopped (either by ctrl-c or stopped by the previous thread)
    println!("Starting the HTTP standalone server...");
    server.await?;

    println!("Standalone HTTP server stopped!");
    if !succeed.load(Ordering::Relaxed) {
        return Err(anyhow::anyhow!(
            "Abort program, failed to get a valid certificate"
        ));
    }
    println!("Certificate has been generated!");

    Ok(())
}
