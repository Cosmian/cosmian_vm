use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct CosmianVmAgent {
    pub agent: Agent,
    pub app: Option<App>,
}

#[derive(Deserialize)]
pub(crate) struct Agent {
    /// Certificate of the VM in PEM format
    #[serde(with = "pem_reader")]
    pub pem_certificate: String,
}

#[derive(Deserialize)]
pub(crate) struct App {
    /// Name of the Linux service (ie: nginx)
    pub service_app_name: String,
    /// Encrypted data storage (ie: tmpfs)
    pub encrypted_folder: PathBuf,
    /// Where the secret app conf is stored encrypted
    pub secret_app_conf: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct EncryptedAppConf {
    /// Version of the app (ie: "1.0")
    pub version: String,
    /// Algorithm used for encryption (ie: "aes256-gcm")
    pub algorithm: EncryptedAppConfAlgorithm,
    /// Base64-encoded nonce of the encrypted data (ie: "base64(abcdef)")
    pub nonce: String,
    /// Base64-encoded content of the encrypted data (ie: "base64(aes256-gcm(file_content))")
    pub data: String,
}

#[derive(Deserialize, Serialize)]
pub enum EncryptedAppConfAlgorithm {
    #[serde(rename = "aes256-gcm")]
    Aes256Gcm,
}

mod pem_reader {
    use serde::{Deserialize as _, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let pem_content = std::fs::read_to_string(s).map_err(serde::de::Error::custom)?;
        Ok(pem_content)
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose, Engine as _};

    use crate::{
        conf::{EncryptedAppConf, EncryptedAppConfAlgorithm},
        CosmianVmAgent,
    };

    #[test]
    fn test_agent_toml() {
        let cfg_str = r#"
            [agent]
            pem_certificate = "../../tests/data/cert.pem"
    
            [app]
            service_app_name = "cosmian_kms"
            encrypted_folder = "/mnt/cosmian_vm/data"
            secret_app_conf = "/etc/cosmian_vm/app_secrets.json"
            "#;

        let config: CosmianVmAgent = toml::from_str(cfg_str).unwrap();
        // test that the content of PEM cert is read
        assert!(config
            .agent
            .pem_certificate
            .starts_with("-----BEGIN CERTIFICATE"));
    }

    #[test]
    fn test_encrypted_app_conf() {
        let eac = EncryptedAppConf {
            version: "1.0".to_string(),
            algorithm: EncryptedAppConfAlgorithm::Aes256Gcm,
            nonce: general_purpose::STANDARD_NO_PAD.encode(b"1234"),
            data: general_purpose::STANDARD_NO_PAD.encode(b"5678"),
        };
        let ser = serde_json::to_string(&eac).unwrap();
        assert_eq!(
            ser,
            r#"{"version":"1.0","algorithm":"aes256-gcm","nonce":"MTIzNA","data":"NTY3OA"}"#
        );
    }
}
