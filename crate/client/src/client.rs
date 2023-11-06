use std::time::Duration;

use http::{HeaderMap, HeaderValue, StatusCode};

use reqwest::{Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone)]
pub struct CosmianVmClient {
    pub server_url: String,
    client: Client,
}

#[derive(Serialize, Deserialize)]
pub struct QuoteParam {
    pub nonce: String,
}

impl CosmianVmClient {
    /// Proceed a snapshot of the VM
    pub async fn snapshot(&self) -> Result<String, Error> {
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
    pub fn instantiate(server_url: &str, accept_invalid_certs: bool) -> Result<Self, Error> {
        let server_url = match server_url.strip_suffix('/') {
            Some(s) => s.to_string(),
            None => server_url.to_string(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));

        let builder = ClientBuilder::new().danger_accept_invalid_certs(accept_invalid_certs);

        // Build the client
        Ok(Self {
            client: builder
                .connect_timeout(Duration::from_secs(5))
                .tcp_keepalive(Duration::from_secs(30))
                .default_headers(headers)
                .build()?,
            server_url,
        })
    }

    pub async fn get<R, O>(&self, endpoint: &str, data: Option<&O>) -> Result<R, Error>
    where
        R: serde::de::DeserializeOwned + Sized + 'static,
        O: Serialize,
    {
        let server_url = format!("{}{endpoint}", self.server_url);
        let response = match data {
            Some(d) => self.client.get(server_url).query(d).send().await?,
            None => self.client.get(server_url).send().await?,
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
        let server_url = format!("{}{endpoint}", self.server_url);
        let response = match data {
            Some(d) => self.client.post(server_url).json(d).send().await?,
            None => self.client.post(server_url).send().await?,
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
