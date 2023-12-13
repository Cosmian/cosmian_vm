use std::path::PathBuf;

use cosmian_vm_client::ser_de::base64_serde;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct CosmianVmAgent {
    pub agent: Agent,
    pub app: Option<App>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Agent {
    /// The host to listen on
    pub host: String,
    /// The port to listen on
    pub port: u16,
    /// SSL certificate of the VM in PEM format
    pub ssl_certificate: PathBuf,
    /// SSL private key of the VM in PEM format
    pub ssl_private_key: PathBuf,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct App {
    /// Name of the Linux service (ie: nginx)
    pub service_app_name: String,
    /// Decrypted data storage (ie: tmpfs)
    pub decrypted_folder: PathBuf,
    /// Where the secret app conf is stored encrypted
    pub encrypted_secret_app_conf: PathBuf,
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

#[cfg(test)]
mod tests {
    use crate::conf::App;
    use crate::{
        conf::{Agent, EncryptedAppConf, EncryptedAppConfAlgorithm},
        CosmianVmAgent,
    };
    use std::path::PathBuf;

    #[test]
    fn test_agent_toml() {
        let cfg_str = r#"
            [agent]
            host = "127.0.0.1"
            port = 5355
            ssl_certificate = "../tests/data/cert.pem"
            ssl_private_key = "../../tests/data/key.pem"

            [app]
            service_app_name = "cosmian_kms"
            decrypted_folder = "/mnt/cosmian_vm/data"
            encrypted_secret_app_conf = "/etc/cosmian_vm/app_secrets.json"
            "#;

        let config: CosmianVmAgent = toml::from_str(cfg_str).unwrap();
        assert_eq!(
            config,
            CosmianVmAgent {
                agent: Agent {
                    host: "127.0.0.1".to_string(),
                    port: 5355,
                    ssl_certificate: PathBuf::from("../tests/data/cert.pem"),
                    ssl_private_key: PathBuf::from("../../tests/data/key.pem")
                },
                app: Some(App {
                    service_app_name: "cosmian_kms".to_string(),
                    decrypted_folder: PathBuf::from("/mnt/cosmian_vm/data"),
                    encrypted_secret_app_conf: PathBuf::from("/etc/cosmian_vm/app_secrets.json")
                })
            }
        );
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
