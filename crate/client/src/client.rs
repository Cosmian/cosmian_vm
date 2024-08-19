use std::{io::Write, sync::Arc, thread::sleep, time::Duration};

use http::{HeaderMap, HeaderValue, StatusCode};
use reqwest::{Client, ClientBuilder, Response, Url};
use rustls::{client::WebPkiVerifier, Certificate};
use serde::{Deserialize, Serialize};
use tpm_quote::PcrHashMethod;

use crate::{
    certificate_verifier::{LeafCertificateVerifier, NoVerifier},
    error::Error,
    ser_de::base64_serde,
    snapshot::CosmianVmSnapshot,
};

#[derive(Clone)]
pub struct CosmianVmClient {
    pub agent_url: String,
    client: Client,
    pub certificate: Certificate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpmQuoteResponse {
    pub pcr_value_hash_method: PcrHashMethod,
    #[serde(with = "base64_serde")]
    pub quote: Vec<u8>,
    #[serde(with = "base64_serde")]
    pub signature: Vec<u8>,
    #[serde(with = "base64_serde")]
    pub public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct QuoteParam {
    #[serde(with = "base64_serde")]
    pub nonce: Vec<u8>,
}

pub const USER_AGENT_ATTRIBUTE: &str = "cli-version";

impl CosmianVmClient {
    /// Proceed a snapshot of the VM
    pub async fn get_snapshot(&self) -> Result<CosmianVmSnapshot, Error> {
        loop {
            let snapshot: Option<CosmianVmSnapshot> = self.get("/snapshot", None::<&()>).await?;

            if let Some(snapshot) = snapshot {
                return Ok(snapshot);
            } else {
                // Not ready
                sleep(Duration::from_secs(10));
            }
        }
    }

    /// Proceed a snapshot of the VM
    pub async fn reset_snapshot(&self) -> Result<(), Error> {
        self.delete("/snapshot", None::<&()>).await
    }

    /// Get the IMA list as an ascii string
    pub async fn ima_ascii(&self) -> Result<String, Error> {
        self.get("/ima/ascii", None::<&()>).await
    }

    /// Get the IMA list as a binary blob
    pub async fn ima_binary(&self) -> Result<Vec<u8>, Error> {
        self.get("/ima/binary", None::<&()>).await
    }

    /// Get the quote of the tee
    pub async fn tee_quote(&self, nonce: &[u8]) -> Result<Vec<u8>, Error> {
        self.get(
            "/quote/tee",
            Some(&QuoteParam {
                nonce: nonce.to_vec(),
            }),
        )
        .await
    }

    /// Get the quote of the tpm
    pub async fn tpm_quote(&self, nonce: &[u8]) -> Result<TpmQuoteResponse, Error> {
        self.get(
            "/quote/tpm",
            Some(&QuoteParam {
                nonce: nonce.to_vec(),
            }),
        )
        .await
    }

    /// Initialize the deployed app
    pub async fn init_app(&self, content: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.post(
            "/app/init",
            Some(&AppConf {
                content: content.to_vec(),
            }),
        )
        .await
    }

    /// Restart the deployed app
    pub async fn restart_app(&self) -> Result<(), Error> {
        self.post("/app/restart", None::<&()>).await
    }

    /// Instantiate a new cosmian VM client
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub fn instantiate(
        agent_url: &str,
        cli_version: &str,
        accept_invalid_certs: bool,
    ) -> Result<Self, Error> {
        let agent_url = agent_url.strip_suffix('/').unwrap_or(agent_url).to_string();

        let mut headers = HeaderMap::new();
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));

        // Get the agent certificate
        let certificate =
            Certificate(get_server_certificate_from_url(&agent_url).map_err(|e| {
                Error::Default(format!("Can't get the Cosmian VM Agent certificate: {e}"))
            })?);

        let builder = build_tls_client_tee(&certificate, accept_invalid_certs)?;

        // Build the client
        Ok(Self {
            client: builder
                .user_agent(format!("{USER_AGENT_ATTRIBUTE}/{cli_version}"))
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

    pub async fn delete<R, O>(&self, endpoint: &str, data: Option<&O>) -> Result<R, Error>
    where
        R: serde::de::DeserializeOwned + Sized + 'static,
        O: Serialize,
    {
        let agent_url = format!("{}{endpoint}", self.agent_url);
        let response = match data {
            Some(d) => self.client.delete(agent_url).query(d).send().await?,
            None => self.client.delete(agent_url).send().await?,
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
    /// Raw content of the configuration.
    ///
    /// Note: fully depends on the app, so
    /// we can't guess better than bytes.
    #[serde(with = "base64_serde")]
    pub content: Vec<u8>,
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

/// Get the leaf certificate from a `host`:`port`
pub fn get_server_certificate_from_url(url: &str) -> Result<Vec<u8>, Error> {
    let agent_url_parsed: Url = Url::parse(url)?;
    let host = agent_url_parsed
        .host_str()
        .ok_or_else(|| Error::Default("Host not found in agent url".to_string()))?;
    let port = agent_url_parsed.port().unwrap_or(443);

    get_server_certificate(host, port)
}

/// Get the leaf certificate from a `host`:`port`
pub fn get_server_certificate(host: &str, port: u16) -> Result<Vec<u8>, Error> {
    let root_store = rustls::RootCertStore::empty();
    let mut socket =
        std::net::TcpStream::connect(format!("{host}:{port}")).map_err(|_| Error::Connection)?;

    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(std::sync::Arc::new(NoVerifier));

    let rc_config = std::sync::Arc::new(config);
    let dns_name = host.try_into().map_err(|_| Error::DNSName)?;

    let mut client =
        rustls::ClientConnection::new(rc_config, dns_name).map_err(|_| Error::Connection)?;

    let mut stream = rustls::Stream::new(&mut client, &mut socket);
    stream.write_all(b"GET / HTTP/1.1\r\nConnection: close\r\n\r\n")?;

    let certificates = client.peer_certificates().ok_or(Error::ServerCertificate)?;

    Ok(certificates
        .first()
        .ok_or(Error::ServerCertificate)?
        .as_ref()
        .to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn test_ratls_get_server_certificate() {
        let server_cert = get_server_certificate("impots.gouv.fr", 443).unwrap();
        // Uncomment this if certifiate has been renewed
        // println!(
        //     "server_cert: {}",
        //     general_purpose::STANDARD.encode(&server_cert)
        // );
        let b64_expected_server_cert = r"
        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"
        .replace(['\n', ' '], "");
        let expected_server_cert = general_purpose::STANDARD
            .decode(b64_expected_server_cert)
            .unwrap();
        assert_eq!(expected_server_cert, server_cert);
    }
}
