use std::time::Duration;

use cosmian_vm_client::cloud_provider::CloudProvider;

const METADATA_URL: &str = "http://169.254.169.254";
const AZURE_ENDPOINT: &str =
    "/metadata/instance/compute/azEnvironment?api-version=2021-02-01&format=text";
const GCP_ENDPOINT: &str = "/computeMetadata/v1/instance/";
const AWS_ENDPOINT: &str = "/latest/meta-data/";

/// Guess the cloud provider using metadata server.
///
/// # Examples
///
/// ```console
/// $ # Azure
/// $ curl -s -H Metadata:true --noproxy "*" "http://169.254.169.254/metadata/instance?api-version=2021-02-01"
/// $ # GCP
/// $ curl -H "Metadata-Flavor: Google" http://169.254.169.254/computeMetadata/v1/project/ -i
/// $ # AWS
/// $ curl http://169.254.169.254/latest/meta-data/
/// ```
pub async fn which_cloud_provider() -> Option<CloudProvider> {
    let client = awc::Client::builder()
        .timeout(Duration::from_secs(1))
        .finish();

    // Azure
    let req = client
        .get(format!("{METADATA_URL}{AZURE_ENDPOINT}"))
        .insert_header(("Metadata", "true"));

    if let Ok(mut res) = req.send().await {
        if res.status() == awc::http::StatusCode::OK {
            let body = res.body().await.expect("can't ready body");
            let content = String::from_utf8_lossy(body.as_ref());

            if content == "AzurePublicCloud" {
                return Some(CloudProvider::Azure);
            }
        }
    }

    // GCP
    let req = client
        .get(format!("{METADATA_URL}{GCP_ENDPOINT}"))
        .insert_header(("Metadata-Flavor", "Google"));

    if let Ok(res) = req.send().await {
        if res.status() == awc::http::StatusCode::OK {
            return Some(CloudProvider::GCP);
        }
    }

    // AWS
    let req = client.get(format!("{METADATA_URL}{AWS_ENDPOINT}"));

    if let Ok(res) = req.send().await {
        if res.status() == awc::http::StatusCode::OK {
            return Some(CloudProvider::AWS);
        }
    }

    None
}
