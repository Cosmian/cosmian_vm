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
