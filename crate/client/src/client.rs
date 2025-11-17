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
MIIYHDCCFgSgAwIBAgITMwMDo5MURl4JyKTFPQAAAwOjkzANBgkqhkiG9w0BAQwFADBdMQswCQYDVQQGEwJVUzEeMBwGA1UEChMVTWljcm9zb2Z0IENvcnBvcmF0aW9uMS4wLAYDVQQDEyVNaWNyb3NvZnQgQXp1cmUgUlNBIFRMUyBJc3N1aW5nIENBIDA0MB4XDTI1MTEwNjE0MzcyM1oXDTI2MDUwNTE0MzcyM1owZDELMAkGA1UEBhMCVVMxCzAJBgNVBAgTAldBMRAwDgYDVQQHEwdSZWRtb25kMR4wHAYDVQQKExVNaWNyb3NvZnQgQ29ycG9yYXRpb24xFjAUBgNVBAMTDW1pY3Jvc29mdC5jb20wggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDe5316uLylnfRk5tHjcYQ6POddiakFmMA2hnkFl+roeVkonKiRYw7rIXgmyUe8nCe+lqG7jRoStEISDmyeA9FYbLaJ8CUXrH72iM1r++lvev9kMkKA8fADp6WBArEVLEnflIavnaLcqwd1XXC/43AZWtjz/omNwryOwJGzmEC5z0lJlmOZmwrCTA7fjVSKj6l1lSBvgfti6NpSMVziGrJnlLja48Wo5hyTboV1/QPIWfZN3s0Mb/LVP3YuFgDIPues4HaCjVnluUrcz2cHe06cLJzjJednaW7ePnvyfWJZtE2LQr3VDnaBqnGAM1R6ZW38pBEbR4hQQ3liHKQQ8VR1AgMBAAGjghPMMIITyDCCAX8GCisGAQQB1nkCBAIEggFvBIIBawFpAHcAlpdkv1VYl633Q4doNwhCd+nwOtX2pPM2bkakPw/KqcYAAAGaWaMMQwAABAMASDBGAiEA3Tjgd/74y9b43kwxFUDtXZTIiBw3DuvGEV/OTg8ezpACIQDkaH2jj/GTL1JQDaXwyyJiNQVSRgEquQiP+YSI5/1p2AB3AGQRxGykEuyniRyiAi4AvKtPKAfUHjUnq+r+1QPJfc3wAAABmlmjDAgAAAQDAEgwRgIhAJegclRpi7g2lUCQ0OVKdjOlCRnb52h1qmv5ElYu3V6gAiEAnZqHddO+Db3wy5JPIzi+xj6f9dEr1O2zU6M05dc0EbMAdQAOV5S8866pPjMbLJkHs/eQ35vCPXEyJd0hqSWsYcVOIQAAAZpZowvZAAAEAwBGMEQCIAlBcuPqx7G0pttToyfwUyCw0mBWdvKyI01sYod0AH5lAiBCr+JZUwlXYIpnjtUJbseS5UMh3slAIx/f4X/Ed5WaNTAnBgkrBgEEAYI3FQoEGjAYMAoGCCsGAQUFBwMCMAoGCCsGAQUFBwMBMDwGCSsGAQQBgjcVBwQvMC0GJSsGAQQBgjcVCIe91xuB5+tGgoGdLo7QDIfw2h1dgqvnMIft8R8CAWQCAS0wgbQGCCsGAQUFBwEBBIGnMIGkMHMGCCsGAQUFBzAChmdodHRwOi8vd3d3Lm1pY3Jvc29mdC5jb20vcGtpb3BzL2NlcnRzL01pY3Jvc29mdCUyMEF6dXJlJTIwUlNBJTIwVExTJTIwSXNzdWluZyUyMENBJTIwMDQlMjAtJTIweHNpZ24uY3J0MC0GCCsGAQUFBzABhiFodHRwOi8vb25lb2NzcC5taWNyb3NvZnQuY29tL29jc3AwHQYDVR0OBBYEFAJU7YlJntIP9tycK4TQaDO7YXI5MA4GA1UdDwEB/wQEAwIFoDCCD9IGA1UdEQSCD8kwgg/Fgg1taWNyb3NvZnQuY29tgg9zLm1pY3Jvc29mdC5jb22CEGdhLm1pY3Jvc29mdC5jb22CEWFlcC5taWNyb3NvZnQuY29tghFhZXIubWljcm9zb2Z0LmNvbYIRZ3J2Lm1pY3Jvc29mdC5jb22CEWh1cC5taWNyb3NvZnQuY29tghFtYWMubWljcm9zb2Z0LmNvbYIRbWtiLm1pY3Jvc29mdC5jb22CEXBtZS5taWNyb3NvZnQuY29tghFwbWkubWljcm9zb2Z0LmNvbYIRcnNzLm1pY3Jvc29mdC5jb22CEXNhci5taWNyb3NvZnQuY29tghF0Y28ubWljcm9zb2Z0LmNvbYISZnVzZS5taWNyb3NvZnQuY29tghJpZWFrLm1pY3Jvc29mdC5jb22CEm1hYzIubWljcm9zb2Z0LmNvbYISbWNzcC5taWNyb3NvZnQuY29tghJvcGVuLm1pY3Jvc29mdC5jb22CEnNob3AubWljcm9zb2Z0LmNvbYISc3B1ci5taWNyb3NvZnQuY29tghNpdHByby5taWNyb3NvZnQuY29tghNtYW5nby5taWNyb3NvZnQuY29tghNtdXNpYy5taWNyb3NvZnQuY29tghNweW1lcy5taWNyb3NvZnQuY29tghNzdG9yZS5taWNyb3NvZnQuY29tghRhZXRoZXIubWljcm9zb2Z0LmNvbYIUYWxlcnRzLm1pY3Jvc29mdC5jb22CFGRlc2lnbi5taWNyb3NvZnQuY29tghRnYXJhZ2UubWljcm9zb2Z0LmNvbYIUZ2lnamFtLm1pY3Jvc29mdC5jb22CFG1zY3RlYy5taWNyb3NvZnQuY29tghRvbmxpbmUubWljcm9zb2Z0LmNvbYIUc3RyZWFtLm1pY3Jvc29mdC5jb22CFWFmZmxpbmsubWljcm9zb2Z0LmNvbYIVY29ubmVjdC5taWNyb3NvZnQuY29tghVkZXZlbG9wLm1pY3Jvc29mdC5jb22CFWRvbWFpbnMubWljcm9zb2Z0LmNvbYIVZXhhbXBsZS5taWNyb3NvZnQuY29tghVtYWRlaXJhLm1pY3Jvc29mdC5jb22CFW1zZG5pc3YubWljcm9zb2Z0LmNvbYIVbXNwcmVzcy5taWNyb3NvZnQuY29tghV3d3cuYWVwLm1pY3Jvc29mdC5jb22CFXd3dy5hZXIubWljcm9zb2Z0LmNvbYIVd3d3YmV0YS5taWNyb3NvZnQuY29tghZidXNpbmVzcy5taWNyb3NvZnQuY29tghZlbXByZXNhcy5taWNyb3NvZnQuY29tghZsZWFybmluZy5taWNyb3NvZnQuY29tghZtc2Rud2lraS5taWNyb3NvZnQuY29tghZvcGVubmVzcy5taWNyb3NvZnQuY29tghZwaW5wb2ludC5taWNyb3NvZnQuY29tghZzbmFja2JveC5taWNyb3NvZnQuY29tghZzcG9uc29ycy5taWNyb3NvZnQuY29tghZzdGF0aW9ucS5taWNyb3NvZnQuY29tghdhaXN0b3JpZXMubWljcm9zb2Z0LmNvbYIXY29tbXVuaXR5Lm1pY3Jvc29mdC5jb22CF2NyYXdsbXNkbi5taWNyb3NvZnQuY29tghdpb3RzY2hvb2wubWljcm9zb2Z0LmNvbYIXbWVzc2VuZ2VyLm1pY3Jvc29mdC5jb22CF21pbmVjcmFmdC5taWNyb3NvZnQuY29tghhiYWNrb2ZmaWNlLm1pY3Jvc29mdC5jb22CGGVudGVycHJpc2UubWljcm9zb2Z0LmNvbYIYaW90Y2VudHJhbC5taWNyb3NvZnQuY29tghhwaW51bmJsb2NrLm1pY3Jvc29mdC5jb22CGHJlcm91dGU0NDMubWljcm9zb2Z0LmNvbYIZY29tbXVuaXRpZXMubWljcm9zb2Z0LmNvbYIZZXhwbG9yZS1zbWIubWljcm9zb2Z0LmNvbYIZZXhwcmVzc2lvbnMubWljcm9zb2Z0LmNvbYIZb25kZXJuZW1lcnMubWljcm9zb2Z0LmNvbYIZdGVjaGFjYWRlbXkubWljcm9zb2Z0LmNvbYIZdGVycmFzZXJ2ZXIubWljcm9zb2Z0LmNvbYIaY29tbXVuaXRpZXMyLm1pY3Jvc29mdC5jb22CGmNvbm5lY3RldmVudC5taWNyb3NvZnQuY29tghpkYXRhcGxhdGZvcm0ubWljcm9zb2Z0LmNvbYIaZW50cmVwcmVuZXVyLm1pY3Jvc29mdC5jb22CGmh4ZC5yZXNlYXJjaC5taWNyb3NvZnQuY29tghptc3BhcnRuZXJpcmEubWljcm9zb2Z0LmNvbYIabXlkYXRhaGVhbHRoLm1pY3Jvc29mdC5jb22CGm9lbWNvbW11bml0eS5taWNyb3NvZnQuY29tghpyZWFsLXN0b3JpZXMubWljcm9zb2Z0LmNvbYIad3d3LmZvcm1zcHJvLm1pY3Jvc29mdC5jb22CG2Z1dHVyZWRlY29kZWQubWljcm9zb2Z0LmNvbYIbdXBncmFkZWNlbnRlci5taWNyb3NvZnQuY29tghxsZWFybmFuYWx5dGljcy5taWNyb3NvZnQuY29tghxvbmxpbmVsZWFybmluZy5taWNyb3NvZnQuY29tgh1idXNpbmVzc2NlbnRyYWwubWljcm9zb2Z0LmNvbYIdY2xvdWQtaW1tZXJzaW9uLm1pY3Jvc29mdC5jb22CHXN0dWRlbnRwYXJ0bmVycy5taWNyb3NvZnQuY29tgh5hbmFseXRpY3NwYXJ0bmVyLm1pY3Jvc29mdC5jb22CHmJ1c2luZXNzcGxhdGZvcm0ubWljcm9zb2Z0LmNvbYIeZXhwbG9yZS1zZWN1cml0eS5taWNyb3NvZnQuY29tgh5rbGVpbnVudGVybmVobWVuLm1pY3Jvc29mdC5jb22CHnBhcnRuZXJjb21tdW5pdHkubWljcm9zb2Z0LmNvbYIfZXhwbG9yZS1tYXJrZXRpbmcubWljcm9zb2Z0LmNvbYIfaW5ub3ZhdGlvbmNvbnRlc3QubWljcm9zb2Z0LmNvbYIfcGFydG5lcmluY2VudGl2ZXMubWljcm9zb2Z0LmNvbYIfcGhvZW5peGNhdGFsb2d1YXQubWljcm9zb2Z0LmNvbYIfc3prb2x5cHJ6eXN6bG9zY2kubWljcm9zb2Z0LmNvbYIfd3d3LnBvd2VyYXV0b21hdGUubWljcm9zb2Z0LmNvbYIgc3VjY2Vzc2lvbnBsYW5uaW5nLm1pY3Jvc29mdC5jb22CImx1bWlhY29udmVyc2F0aW9uc3VrLm1pY3Jvc29mdC5jb22CI3N1Y2Nlc3Npb25wbGFubmluZ3VhdC5taWNyb3NvZnQuY29tgiRidXNpbmVzc21vYmlsaXR5Y2VudGVyLm1pY3Jvc29mdC5jb22CJXNreXBlYW5kdGVhbXMuZmFzdHRyYWNrLm1pY3Jvc29mdC5jb22CJ3d3dy5taWNyb3NvZnRkbGFwYXJ0bmVyb3cubWljcm9zb2Z0LmNvbYIoY29tbWVyY2lhbGFwcGNlcnRpZmljYXRpb24ubWljcm9zb2Z0LmNvbYIpd3d3LnNreXBlYW5kdGVhbXMuZmFzdHRyYWNrLm1pY3Jvc29mdC5jb22CImNlb2Nvbm5lY3Rpb25zLmV2ZW50Lm1pY3Jvc29mdC5jb22CGGJpejRhZnJpa2EubWljcm9zb2Z0LmNvbYIWY2FzaGJhY2subWljcm9zb2Z0LmNvbYIad3d3LmNhc2hiYWNrLm1pY3Jvc29mdC5jb22CE3Zpc2lvLm1pY3Jvc29mdC5jb22CF2luc2lkZW1zci5taWNyb3NvZnQuY29tgh9kZXZlbG9wZXJ2ZWxvY2l0eWFzc2Vzc21lbnQuY29tgiN3d3cuZGV2ZWxvcGVydmVsb2NpdHlhc3Nlc3NtZW50LmNvbYIKZ2VhcnM1LmNvbYIOd3d3LmdlYXJzNS5jb22CFHd3dy5nZWFyc3RhY3RpY3MuY29tghBnZWFyc3RhY3RpY3MuY29tghFtMTIubWljcm9zb2Z0LmNvbYIMc2VlaW5nYWkuY29tghh5b3VyY2hvaWNlLm1pY3Jvc29mdC5jb22CGW12dGQuZXZlbnRzLm1pY3Jvc29mdC5jb22CFWltYWdpbmUubWljcm9zb2Z0LmNvbYIQbWljcm9zb2Z0LmNvbS5hdYIUd3d3Lm1pY3Jvc29mdC5jb20uYXWCFmR5bmFtaWNzLm1pY3Jvc29mdC5jb22CG3Bvd2VycGxhdGZvcm0ubWljcm9zb2Z0LmNvbYIXcG93ZXJhcHBzLm1pY3Jvc29mdC5jb22CG3Bvd2VyYXV0b21hdGUubWljcm9zb2Z0LmNvbYIgcG93ZXJ2aXJ0dWFsYWdlbnRzLm1pY3Jvc29mdC5jb22CGHBvd2VycGFnZXMubWljcm9zb2Z0LmNvbYIfdGVzdC5pZGVhcy5mYWJyaWMubWljcm9zb2Z0LmNvbYIRc2RzLm1pY3Jvc29mdC5jb22CFXBwZS5zZHMubWljcm9zb2Z0LmNvbYIbd3d3Lm1pY3Jvc29mdDM2NWNvcGlsb3QuY29tghB3d3cuamNsYXJpdHkuY29tght0ZWNoaW5ub3ZhdG9yc3Nwb3RsaWdodC5jb22CH3d3dy50ZWNoaW5ub3ZhdG9yc3Nwb3RsaWdodC5jb22CCmNvcGlsb3QuYWmCFWdldGxpY2Vuc2luZ3JlYWR5LmNvbYIZd3d3LmdldGxpY2Vuc2luZ3JlYWR5LmNvbYIUanBuLmRlbHZlLm9mZmljZS5jb22CFGF1cy5kZWx2ZS5vZmZpY2UuY29tghRpbmQuZGVsdmUub2ZmaWNlLmNvbYIUa29yLmRlbHZlLm9mZmljZS5jb22CFmNvYnJhLm1lLm1pY3Jvc29mdC5jb22CF3d3dy5idXNpbmVzc2NlbnRyYWwuY29tghNidXNpbmVzc2NlbnRyYWwuY29tghxtc2FpZGF0YXN0dWRpby5vZmZpY2VwcGUubmV0ghppZGVhcy5mYWJyaWMubWljcm9zb2Z0LmNvbYIMd3d3LmNwdC5saW5rgghjcHQubGlua4IMeWFycC5kb3QubmV0ghNtaWNyb3NvZnRzdHJlYW0uY29tghd3d3cubWljcm9zb2Z0c3RyZWFtLmNvbYIXd2ViLm1pY3Jvc29mdHN0cmVhbS5jb22CE2Rpc2NvdmVyLmNvcGlsb3QuYWmCC2NvcGlsb3QuY29tgg93d3cuY29waWxvdC5jb22CFGRpc2NvdmVyLmNvcGlsb3QuY29tghtyZXNlYXJjaGZvcnVtLm1pY3Jvc29mdC5jb20wDAYDVR0TAQH/BAIwADBqBgNVHR8EYzBhMF+gXaBbhllodHRwOi8vd3d3Lm1pY3Jvc29mdC5jb20vcGtpb3BzL2NybC9NaWNyb3NvZnQlMjBBenVyZSUyMFJTQSUyMFRMUyUyMElzc3VpbmclMjBDQSUyMDA0LmNybDBmBgNVHSAEXzBdMFEGDCsGAQQBgjdMg30BATBBMD8GCCsGAQUFBwIBFjNodHRwOi8vd3d3Lm1pY3Jvc29mdC5jb20vcGtpb3BzL0RvY3MvUmVwb3NpdG9yeS5odG0wCAYGZ4EMAQICMB8GA1UdIwQYMBaAFDtw0VPpdiWdYKjKZg/Gm65vVBZqMB0GA1UdJQQWMBQGCCsGAQUFBwMCBggrBgEFBQcDATANBgkqhkiG9w0BAQwFAAOCAgEAq3pWzil0AcdFJOPyXvotBlGXU1CvH5b9xePwGWFmu9yG+qgx6QDqhSye6pUSaEAviuG512M2+KC2WlMC6bVgF09rDlnuRCW7FELIJBe0OKgxeDm2TnZ0N0FAQsnqzlpqUzmTz9tDvYtedZUWTkqGlOOMNa4TnaeTDe+P13wXqrJ+LDe127HB+LKhRQ9DzXJgxAEqG5NggemIjEwC928TDjyWeniy+FCGnVQF7GgvfUgLBlQyn/Cfn0DPsp+iXIyOAWwUWC0gDrEVJEU+YTAiCQJ3gHW3FFqJ9DzaJa4fkkN//Vq7uqYaaJHJ5TwRcvRRpzjkTCn6j6Cnyj7leHZy9l4d8vp9nX+ZT4DiJbOLZ9qRe61taCYdbb0nEv9B8J+N8c32JobPSU0LSn81vM3YQ/R43JDfv90+Xx+geKZR6TEEhtmJalK9Au01YXXXOBDuM5pzWTYiSe0ner2OGSF6lnflgVGcqtXOxnwje2eLhYx82hRnKQWmruuuzUYmg5IlIHKz+DCs4JDuYr+ldQokNbX0Qqq/3pWj7m2xhKC1fTCc1sm5CsPyfugPQWkNQW0gHx75WUEbUH8lBpDtZvToHXD4ERYW6HYniDGEeS8OkvTKYeaICcK2xZC9MY/ZWFPp9hnWk4X7pqURGP6KfBZ7lTbgNQLwsCRpChxMXt45/GM="        .replace(['\n', ' '], "");
        let expected_server_cert = general_purpose::STANDARD
            .decode(b64_expected_server_cert)
            .unwrap();
        assert_eq!(expected_server_cert, server_cert);
    }
}
