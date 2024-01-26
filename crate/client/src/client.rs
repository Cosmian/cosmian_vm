use std::{io::Write, sync::Arc, thread::sleep, time::Duration};

use http::{HeaderMap, HeaderValue, StatusCode};
use reqwest::{Client, ClientBuilder, Response, Url};
use rustls::{client::WebPkiVerifier, Certificate};
use serde::{Deserialize, Serialize};
use tpm_quote::PcrHashMethod;

use crate::{
    certificate_verifier::{LeafCertificateVerifier, NoVerifier},
    error::Error,
    ser_de::{base64_serde, base64_serde_opt},
    snapshot::CosmianVmSnapshot,
};

#[derive(Clone)]
pub struct CosmianVmClient {
    pub agent_url: String,
    client: Client,
    pub certificate: Certificate,
}

#[derive(Clone, Serialize, Deserialize)]
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
                sleep(Duration::from_secs(10))
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
    pub async fn init_app(
        &self,
        content: &[u8],
        key: Option<&[u8]>,
    ) -> Result<Option<Vec<u8>>, Error> {
        self.post(
            "/app/init",
            Some(&AppConf {
                content: content.to_vec(),
                key: key.map(<[u8]>::to_vec),
            }),
        )
        .await
    }

    /// Restart the deployed app
    pub async fn restart_app(&self, key: &[u8]) -> Result<(), Error> {
        self.post("/app/restart", Some(&RestartParam { key: key.to_vec() }))
            .await
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

    /// Key/password used to encrypt the app configuration.
    ///
    /// If `None` is provided, a new random key
    /// is generated when calling `/init` endpoint.
    #[serde(with = "base64_serde_opt")]
    pub key: Option<Vec<u8>>,
}

/// Configuration to restart a deployed app (ie: after a reboot)
#[derive(Serialize, Deserialize)]
pub struct RestartParam {
    /// Key/password used to decrypt the app configuration.
    #[serde(with = "base64_serde")]
    pub key: Vec<u8>,
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
        let server_cert = get_server_certificate("self-signed.badssl.com", 443).unwrap();

        let b64_server_cert = r"
        MIIDeTCCAmGgAwIBAgIJAMrlEvmXqB1TMA0GCSqGSIb3DQEBCwUAMGIxCzAJBgNV
        BAYTAlVTMRMwEQYDVQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1TYW4gRnJhbmNp
        c2NvMQ8wDQYDVQQKDAZCYWRTU0wxFTATBgNVBAMMDCouYmFkc3NsLmNvbTAeFw0y
        MzExMjkyMjM0MDRaFw0yNTExMjgyMjM0MDRaMGIxCzAJBgNVBAYTAlVTMRMwEQYD
        VQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1TYW4gRnJhbmNpc2NvMQ8wDQYDVQQK
        DAZCYWRTU0wxFTATBgNVBAMMDCouYmFkc3NsLmNvbTCCASIwDQYJKoZIhvcNAQEB
        BQADggEPADCCAQoCggEBAMIE7PiM7gTCs9hQ1XBYzJMY61yoaEmwIrX5lZ6xKyx2
        PmzAS2BMTOqytMAPgLaw+XLJhgL5XEFdEyt/ccRLvOmULlA3pmccYYz2QULFRtMW
        hyefdOsKnRFSJiFzbIRMeVXk0WvoBj1IFVKtsyjbqv9u/2CVSndrOfEk0TG23U3A
        xPxTuW1CrbV8/q71FdIzSOciccfCFHpsKOo3St/qbLVytH5aohbcabFXRNsKEqve
        ww9HdFxBIuGa+RuT5q0iBikusbpJHAwnnqP7i/dAcgCskgjZjFeEU4EFy+b+a1SY
        QCeFxxC7c3DvaRhBB0VVfPlkPz0sw6l865MaTIbRyoUCAwEAAaMyMDAwCQYDVR0T
        BAIwADAjBgNVHREEHDAaggwqLmJhZHNzbC5jb22CCmJhZHNzbC5jb20wDQYJKoZI
        hvcNAQELBQADggEBAJYrbHPC6Yor7oi3aimJAPnnTh9Z7sQyaGfZ4I1ZIayWGIF7
        +9dq/VtCYxEeq7bZELvqcK6LMtQQ7xGoJ5yCgJWjO/SbLaSy1AEa5m9im3Gg2k4w
        h1AE8Z3CQUEdazVTsLKxdCp+eN62jQAzTY8xQ6yKDaWmTUhvSgErJyBv/H+vTQ+9
        L5ghqMrDUZTkxgwlXs3OyJi/S/Rfv9OGiEua/T+h3yHEzOL53d+IiagOUCjUg7mP
        5g4MP8zks3VcxERVjtzOahBH7fvhsMuJ/i+lSiNMMVaOr/U9Y1Y9kq96YIPax6Re
        Jok9KYiYJsWbiimaCxWFT/HbLvD+qri7lD2Gm8A=
        "
        .replace(['\n', ' '], "");
        let expected_server_cert = general_purpose::STANDARD.decode(b64_server_cert).unwrap();
        assert_eq!(expected_server_cert, server_cert);
    }
}
