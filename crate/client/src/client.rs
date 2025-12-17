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
        let agent_url = agent_url.strip_suffix('/').unwrap_or(agent_url).to_owned();

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
            StatusCode::NOT_FOUND => "Endpoint does not exist".to_owned(),
            StatusCode::UNAUTHORIZED => "Bad authorization token".to_owned(),
            _ => format!("{status} {text}"),
        })
    }
}

/// Build a `TLSClient` to use with a Cosmian VM Agent running inside a tee
/// The TLS verification is the basic one but also include the verification of the leaf certificate
/// The TLS socket is mounted since the leaf certificate is exactly the same than the expected one.
pub(crate) fn build_tls_client_tee(
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
        .ok_or_else(|| Error::Default("Host not found in agent url".to_owned()))?;
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
        println!(
             "server_cert: {}",
         general_purpose::STANDARD.encode(&server_cert)
        );
        let b64_expected_server_cert = r"
        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"
        .replace(['\n', ' '], "");
        let expected_server_cert = general_purpose::STANDARD
            .decode(b64_expected_server_cert)
            .unwrap();
        assert_eq!(expected_server_cert, server_cert);
    }
}
