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
    /// Algorithm used for encryption
    pub algorithm: EncryptedAppConfAlgorithm,
    /// Nonce of the encrypted data.
    ///
    /// This data is base64 encoded when serialized in conf
    #[serde(with = "base64_serde")]
    pub nonce: Vec<u8>,
    /// Encrypted data (ie: "aes256-gcm(file_content)").
    ///
    /// This data is base64 encoded when serialized in conf
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub enum EncryptedAppConfAlgorithm {
    #[serde(rename = "aes256-gcm")]
    Aes256Gcm,
}

mod base64_serde {
    use base64::{engine::general_purpose, Engine as _};
    use serde::Serializer;
    use serde::{Deserialize as _, Deserializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = general_purpose::STANDARD_NO_PAD.encode(v);
        s.serialize_str(&base64)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        general_purpose::STANDARD_NO_PAD
            .decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)
    }
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
            nonce: b"1234".to_vec(),
            data: b"5678".to_vec(),
        };
        let ser = serde_json::to_string(&eac).unwrap();
        assert_eq!(
            ser,
            r#"{"version":"1.0","algorithm":"aes256-gcm","nonce":"MTIzNA","data":"NTY3OA"}"#
        );
    }
}