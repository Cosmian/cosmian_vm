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
        // general_purpose::STANDARD.encode(&server_cert)
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
