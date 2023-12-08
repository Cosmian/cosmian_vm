use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use http::{HeaderMap, HeaderValue, StatusCode};

use crate::certificate_verifier::{LeafCertificateVerifier, NoVerifier};
use crate::error::Error;
use crate::snapshot::CosmianVmSnapshot;
use ratls::verify::get_server_certificate;
use reqwest::{Client, ClientBuilder, Response, Url};
use rustls::{client::WebPkiVerifier, Certificate};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CosmianVmClient {
    pub agent_url: String,
    client: Client,
    pub certificate: Certificate,
}

#[derive(Serialize, Deserialize)]
pub struct QuoteParam {
    pub nonce: String,
}

impl CosmianVmClient {
    /// Proceed a snapshot of the VM
    pub async fn snapshot(&self) -> Result<CosmianVmSnapshot, Error> {
        self.get("/snapshot", None::<&()>).await
    }

    /// Get the IMA list as an ascii string
    pub async fn ima_ascii(&self) -> Result<String, Error> {
        self.get("/ima/ascii", None::<&()>).await
    }

    /// Get the IMA list as a binary blob
    pub async fn ima_binary(&self) -> Result<Vec<u8>, Error> {
        self.get("/ima/binary", None::<&()>).await
    }

    /// Get the PCR value for register number `id`
    pub async fn pcr_value(&self, id: u32) -> Result<String, Error> {
        self.get(&format!("/tmp_endpoint/pcr/{id}"), None::<&()>)
            .await
    }

    /// Get the quote of the tee
    pub async fn tee_quote(&self, nonce: &[u8]) -> Result<Vec<u8>, Error> {
        self.get(
            "/quote/tee",
            Some(&QuoteParam {
                nonce: hex::encode(nonce),
            }),
        )
        .await
    }

    /// Get the quote of the tpm
    pub async fn tpm_quote(&self, nonce: &[u8]) -> Result<Vec<u8>, Error> {
        self.get(
            "/quote/tpm",
            Some(&QuoteParam {
                nonce: hex::encode(nonce),
            }),
        )
        .await
    }

    /// Instantiate a new cosmian VM client
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub fn instantiate(agent_url: &str, accept_invalid_certs: bool) -> Result<Self, Error> {
        let agent_url = agent_url.strip_suffix('/').unwrap_or(agent_url).to_string();

        let mut headers = HeaderMap::new();
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));

        // Get the agent certificate
        let agent_url_parsed: Url = Url::parse(&agent_url)?;
        let certificate = Certificate(
            get_server_certificate(
                agent_url_parsed
                    .host_str()
                    .ok_or_else(|| Error::Default("Host not found in agent url".to_string()))?,
                agent_url_parsed.port().unwrap_or(443).into(),
            )
            .map_err(|e| {
                Error::Default(format!("Can't get the Cosmian VM Agent certificate: {e}"))
            })?,
        );

        let builder = build_tls_client_tee(&certificate, accept_invalid_certs)?;

        // Build the client
        Ok(Self {
            client: builder
                .connect_timeout(Duration::from_secs(5))
                .tcp_keepalive(Duration::from_secs(30))
                .default_headers(headers)
                .build()?,
            agent_url,
            certificate,
        })
    }

    pub async fn get<R, O>(&self, endpoint: &str, data: Option<&O>) -> Result<R, Error>
    where
        R: serde::de::DeserializeOwned + Sized + 'static,
        O: Serialize,
    {
        let agent_url = format!("{}{endpoint}", self.agent_url);
        let response = match data {
            Some(d) => self.client.get(agent_url).query(d).send().await?,
            None => self.client.get(agent_url).send().await?,
        };

        let status_code = response.status();
        if status_code.is_success() {
            return Ok(response.json::<R>().await?);
        }

        // process error
        let p = handle_error(response).await?;
        Err(Error::RequestFailed(p))
    }

    pub async fn post<O, R>(&self, endpoint: &str, data: Option<&O>) -> Result<R, Error>
    where
        O: Serialize,
        R: serde::de::DeserializeOwned + Sized + 'static,
    {
        let agent_url = format!("{}{endpoint}", self.agent_url);
        let response = match data {
            Some(d) => self.client.post(agent_url).json(d).send().await?,
            None => self.client.post(agent_url).send().await?,
        };

        let status_code = response.status();
        if status_code.is_success() {
            return Ok(response.json::<R>().await?);
        }

        // process error
        let p = handle_error(response).await?;
        Err(Error::RequestFailed(p))
    }
}

/// Configuration of the deployed application.
///
/// This configuration depends on the app developer.
#[derive(Serialize, Deserialize)]
pub struct AppConf {
    /// Location of the deployed app configuration file.
    ///
    /// This location is usually a path within
    /// the encrypted part of disk (ie: tmpfs).
    ///
    /// This is the expected location by the app
    /// when running onto the VM.
    pub deployed_filepath: PathBuf,

    /// Raw content of the configuration.
    ///
    /// Note: fully depends on the app, so
    /// we can't guess better than bytes.
    pub content: Vec<u8>,

    /// Key/password used to encrypt the app configuration.
    ///
    /// If `None` is provided, a new random key
    /// is generated when calling `/init` endpoint.
    pub wrap_key: Option<Vec<u8>>,
}

/// Configuration to restart a deployed app (ie: after a reboot)
#[derive(Serialize, Deserialize)]
pub struct RestartParam {
    /// Location of the deployed app configuration file.
    ///
    /// This location is usually a path within
    /// the encrypted part of disk (ie: tmpfs).
    ///
    /// This is the expected location by the app
    /// when running onto the VM.
    pub deployed_filepath: PathBuf,

    /// Key/password used to decrypt the app configuration.
    pub wrap_key: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ErrorPayload {
    pub error: String,
    pub messages: Option<Vec<String>>,
}

/// Some errors are returned by the Middleware without going through our own error manager.
/// In that case, we make the error clearer here for the client.
async fn handle_error(response: Response) -> Result<String, Error> {
    let status = response.status();
    let text = response.text().await?;

    if !text.is_empty() {
        Ok(text)
    } else {
        Ok(match status {
            StatusCode::NOT_FOUND => "Endpoint does not exist".to_string(),
            StatusCode::UNAUTHORIZED => "Bad authorization token".to_string(),
            _ => format!("{status} {text}"),
        })
    }
}

/// Build a `TLSClient` to use with a Cosmian VM Agent running inside a tee
/// The TLS verification is the basic one but also include the verification of the leaf certificate
/// The TLS socket is mounted since the leaf certificate is exactly the same than the expected one.
pub fn build_tls_client_tee(
    leaf_cert: &Certificate,
    accept_invalid_certs: bool,
) -> Result<ClientBuilder, Error> {
    let mut root_cert_store = rustls::RootCertStore::empty();

    let trust_anchors = webpki_roots::TLS_SERVER_ROOTS.iter().map(|trust_anchor| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            trust_anchor.subject,
            trust_anchor.spki,
            trust_anchor.name_constraints,
        )
    });
    root_cert_store.add_trust_anchors(trust_anchors);

    let verifier = if !accept_invalid_certs {
        LeafCertificateVerifier::new(
            leaf_cert,
            Arc::new(WebPkiVerifier::new(root_cert_store, None)),
        )
    } else {
        LeafCertificateVerifier::new(leaf_cert, Arc::new(NoVerifier))
    };

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();

    // Create a client builder
    Ok(Client::builder().use_preconfigured_tls(config))
}
