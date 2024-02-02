use std::{
    fs::File,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{app::service::ServiceType, error::Error};

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct CosmianVmAgent {
    pub agent: Agent,
    pub app: Option<App>,
}

impl CosmianVmAgent {
    /// Extract the leaf certificate from a pem file. Returning in DER.
    pub fn read_leaf_certificate(&self) -> Result<Vec<u8>, Error> {
        let mut reader = std::io::BufReader::new(File::open(self.ssl_certificate())?);
        let certificate = rustls_pemfile::read_one(&mut reader)?;
        if let Some(rustls_pemfile::Item::X509Certificate(certificate)) = certificate {
            Ok(certificate)
        } else {
            Err(Error::Certificate(format!(
                "No PEM certificate found in {:?}",
                self.ssl_certificate()
            )))
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Agent {
    /// Data storage (encrypted fs, ramfs and session/cache data)
    pub data_storage: PathBuf,
    /// The host to listen to
    pub host: String,
    /// The port to listen to
    pub port: u16,
    /// SSL certificate of the VM in PEM format
    ssl_certificate: PathBuf,
    /// SSL private key of the VM in PEM format
    ssl_private_key: PathBuf,
    /// Transmission interface with the TPM (ie: "/dev/tpmrm0")
    pub tpm_device: Option<PathBuf>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct App {
    /// Type of application
    pub service_type: ServiceType,
    /// Name of the Linux service (ie: nginx)
    pub service_name: String,
    /// Data storage for this application
    app_storage: PathBuf,
}

impl CosmianVmAgent {
    fn _relative_to_data_storage(data_storage: &Path, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            data_storage.join(path)
        }
    }

    pub fn ssl_certificate(&self) -> PathBuf {
        Self::_relative_to_data_storage(&self.agent.data_storage, &self.agent.ssl_certificate)
    }

    pub fn ssl_private_key(&self) -> PathBuf {
        Self::_relative_to_data_storage(&self.agent.data_storage, &self.agent.ssl_private_key)
    }

    pub fn app_storage(&self) -> Option<PathBuf> {
        self.app
            .as_ref()
            .map(|app| Self::_relative_to_data_storage(&self.agent.data_storage, &app.app_storage))
    }
}

#[cfg(test)]
mod tests {
    use crate::app::service::ServiceType;
    use crate::{
        conf::{Agent, App},
        CosmianVmAgent,
    };
    use std::path::PathBuf;

    #[test]
    fn test_agent_toml() {
        let cfg_str = r#"
            [agent]
            data_storage = "/var/lib/cosmian_vm/"
            host = "127.0.0.1"
            port = 5355
            ssl_certificate = "data/cert.pem"
            ssl_private_key = "data/key.pem"
            tpm_device = "/dev/tpmrm0"

            [app]
            service_type = "supervisor"
            service_name = "cosmian_kms"
            app_storage = "data/app"
            "#;

        let config: CosmianVmAgent = toml::from_str(cfg_str).unwrap();
        assert_eq!(
            config,
            CosmianVmAgent {
                agent: Agent {
                    host: "127.0.0.1".to_string(),
                    port: 5355,
                    ssl_certificate: PathBuf::from("data/cert.pem"),
                    ssl_private_key: PathBuf::from("data/key.pem"),
                    tpm_device: Some(PathBuf::from("/dev/tpmrm0")),
                    data_storage: PathBuf::from("/var/lib/cosmian_vm/"),
                },
                app: Some(App {
                    service_type: ServiceType::Supervisor,
                    service_name: "cosmian_kms".to_string(),
                    app_storage: PathBuf::from("data/app"),
                })
            }
        );

        assert_eq!(
            config.ssl_certificate(),
            PathBuf::from("/var/lib/cosmian_vm/data/cert.pem")
        );
        assert_eq!(
            config.ssl_private_key(),
            PathBuf::from("/var/lib/cosmian_vm/data/key.pem")
        );
        assert_eq!(
            config.app_storage(),
            Some(PathBuf::from("/var/lib/cosmian_vm/data/app"))
        );

        let config = CosmianVmAgent {
            agent: Agent {
                host: "127.0.0.1".to_string(),
                port: 5355,
                ssl_certificate: PathBuf::from("data/cert.pem"),
                ssl_private_key: PathBuf::from("data/key.pem"),
                tpm_device: None,
                data_storage: PathBuf::from("./"),
            },
            app: None,
        };

        assert_eq!(
            config.read_leaf_certificate().unwrap(),
            [
                48, 130, 4, 244, 48, 130, 3, 220, 160, 3, 2, 1, 2, 2, 18, 4, 135, 154, 148, 249,
                20, 235, 199, 206, 248, 179, 34, 33, 224, 83, 210, 65, 140, 48, 13, 6, 9, 42, 134,
                72, 134, 247, 13, 1, 1, 11, 5, 0, 48, 50, 49, 11, 48, 9, 6, 3, 85, 4, 6, 19, 2, 85,
                83, 49, 22, 48, 20, 6, 3, 85, 4, 10, 19, 13, 76, 101, 116, 39, 115, 32, 69, 110,
                99, 114, 121, 112, 116, 49, 11, 48, 9, 6, 3, 85, 4, 3, 19, 2, 82, 51, 48, 30, 23,
                13, 50, 51, 48, 54, 50, 57, 48, 54, 51, 53, 48, 54, 90, 23, 13, 50, 51, 48, 57, 50,
                55, 48, 54, 51, 53, 48, 53, 90, 48, 30, 49, 28, 48, 26, 6, 3, 85, 4, 3, 12, 19, 42,
                46, 100, 101, 118, 46, 99, 111, 115, 109, 105, 108, 105, 110, 107, 46, 99, 111,
                109, 48, 130, 1, 34, 48, 13, 6, 9, 42, 134, 72, 134, 247, 13, 1, 1, 1, 5, 0, 3,
                130, 1, 15, 0, 48, 130, 1, 10, 2, 130, 1, 1, 0, 200, 99, 45, 162, 198, 49, 254, 49,
                209, 133, 85, 51, 132, 54, 115, 75, 196, 199, 106, 71, 6, 210, 170, 86, 29, 69,
                131, 51, 147, 131, 75, 37, 100, 195, 119, 9, 181, 97, 61, 17, 25, 45, 194, 212, 59,
                56, 111, 48, 129, 211, 96, 5, 173, 200, 56, 15, 128, 12, 40, 140, 208, 21, 213, 89,
                252, 59, 53, 167, 132, 253, 8, 29, 240, 202, 17, 53, 51, 24, 171, 67, 38, 140, 101,
                88, 161, 246, 79, 116, 16, 96, 192, 184, 154, 120, 154, 90, 55, 14, 132, 109, 21,
                85, 14, 113, 109, 75, 56, 221, 239, 137, 37, 144, 153, 227, 214, 177, 61, 171, 93,
                188, 111, 243, 68, 225, 151, 105, 101, 165, 106, 236, 79, 132, 226, 15, 247, 153,
                208, 50, 12, 231, 11, 194, 63, 32, 68, 202, 190, 19, 48, 162, 39, 173, 244, 90, 4,
                17, 120, 106, 137, 170, 197, 203, 118, 181, 172, 165, 47, 218, 83, 129, 110, 65,
                220, 61, 174, 100, 254, 170, 189, 23, 197, 121, 60, 28, 180, 100, 108, 145, 29,
                221, 108, 55, 133, 135, 27, 132, 1, 6, 121, 167, 203, 170, 57, 209, 187, 112, 102,
                173, 98, 159, 102, 201, 46, 2, 187, 251, 149, 46, 177, 17, 83, 255, 46, 75, 116,
                128, 202, 128, 76, 93, 31, 59, 227, 17, 80, 3, 201, 92, 218, 220, 172, 67, 21, 110,
                221, 190, 224, 62, 42, 254, 221, 237, 65, 155, 230, 185, 2, 3, 1, 0, 1, 163, 130,
                2, 22, 48, 130, 2, 18, 48, 14, 6, 3, 85, 29, 15, 1, 1, 255, 4, 4, 3, 2, 5, 160, 48,
                29, 6, 3, 85, 29, 37, 4, 22, 48, 20, 6, 8, 43, 6, 1, 5, 5, 7, 3, 1, 6, 8, 43, 6, 1,
                5, 5, 7, 3, 2, 48, 12, 6, 3, 85, 29, 19, 1, 1, 255, 4, 2, 48, 0, 48, 29, 6, 3, 85,
                29, 14, 4, 22, 4, 20, 114, 117, 61, 194, 54, 95, 254, 130, 237, 213, 217, 143, 143,
                64, 209, 217, 134, 67, 3, 120, 48, 31, 6, 3, 85, 29, 35, 4, 24, 48, 22, 128, 20,
                20, 46, 179, 23, 183, 88, 86, 203, 174, 80, 9, 64, 230, 31, 175, 157, 139, 20, 194,
                198, 48, 85, 6, 8, 43, 6, 1, 5, 5, 7, 1, 1, 4, 73, 48, 71, 48, 33, 6, 8, 43, 6, 1,
                5, 5, 7, 48, 1, 134, 21, 104, 116, 116, 112, 58, 47, 47, 114, 51, 46, 111, 46, 108,
                101, 110, 99, 114, 46, 111, 114, 103, 48, 34, 6, 8, 43, 6, 1, 5, 5, 7, 48, 2, 134,
                22, 104, 116, 116, 112, 58, 47, 47, 114, 51, 46, 105, 46, 108, 101, 110, 99, 114,
                46, 111, 114, 103, 47, 48, 30, 6, 3, 85, 29, 17, 4, 23, 48, 21, 130, 19, 42, 46,
                100, 101, 118, 46, 99, 111, 115, 109, 105, 108, 105, 110, 107, 46, 99, 111, 109,
                48, 19, 6, 3, 85, 29, 32, 4, 12, 48, 10, 48, 8, 6, 6, 103, 129, 12, 1, 2, 1, 48,
                130, 1, 5, 6, 10, 43, 6, 1, 4, 1, 214, 121, 2, 4, 2, 4, 129, 246, 4, 129, 243, 0,
                241, 0, 119, 0, 183, 62, 251, 36, 223, 156, 77, 186, 117, 242, 57, 197, 186, 88,
                244, 108, 93, 252, 66, 207, 122, 159, 53, 196, 158, 29, 9, 129, 37, 237, 180, 153,
                0, 0, 1, 137, 6, 19, 198, 158, 0, 0, 4, 3, 0, 72, 48, 70, 2, 33, 0, 222, 37, 83,
                255, 40, 178, 44, 249, 249, 148, 114, 42, 21, 215, 149, 188, 52, 206, 84, 191, 36,
                181, 9, 71, 27, 235, 115, 175, 68, 187, 155, 18, 2, 33, 0, 222, 210, 212, 59, 222,
                125, 180, 82, 186, 17, 181, 83, 217, 190, 26, 95, 102, 1, 38, 87, 102, 146, 95,
                249, 205, 12, 129, 130, 130, 229, 152, 70, 0, 118, 0, 173, 247, 190, 250, 124, 255,
                16, 200, 139, 157, 61, 156, 30, 62, 24, 106, 180, 103, 41, 93, 207, 177, 12, 36,
                202, 133, 134, 52, 235, 220, 130, 138, 0, 0, 1, 137, 6, 19, 198, 203, 0, 0, 4, 3,
                0, 71, 48, 69, 2, 32, 117, 176, 203, 132, 40, 72, 232, 86, 28, 0, 196, 160, 166, 4,
                196, 219, 60, 150, 118, 60, 217, 155, 25, 235, 212, 189, 82, 97, 68, 147, 188, 29,
                2, 33, 0, 176, 56, 227, 167, 70, 51, 81, 126, 118, 183, 166, 119, 74, 169, 59, 17,
                49, 238, 246, 146, 43, 197, 160, 111, 208, 144, 104, 122, 241, 1, 22, 19, 48, 13,
                6, 9, 42, 134, 72, 134, 247, 13, 1, 1, 11, 5, 0, 3, 130, 1, 1, 0, 42, 69, 27, 225,
                4, 125, 46, 132, 67, 93, 222, 212, 150, 42, 224, 56, 83, 166, 160, 229, 206, 94,
                97, 63, 163, 83, 68, 228, 58, 46, 166, 33, 201, 17, 32, 85, 209, 158, 155, 76, 154,
                178, 235, 177, 0, 172, 175, 169, 3, 74, 192, 224, 110, 188, 235, 199, 248, 184,
                159, 41, 181, 96, 172, 187, 87, 5, 174, 108, 38, 113, 207, 196, 146, 105, 155, 84,
                188, 189, 216, 145, 143, 143, 122, 200, 163, 11, 117, 201, 70, 152, 196, 244, 15,
                13, 37, 66, 30, 220, 246, 82, 147, 147, 231, 38, 26, 237, 103, 12, 52, 199, 122,
                82, 119, 60, 149, 105, 102, 244, 121, 27, 74, 110, 214, 38, 194, 13, 72, 172, 37,
                192, 132, 131, 253, 97, 9, 147, 158, 136, 251, 92, 223, 114, 145, 151, 0, 6, 190,
                43, 182, 63, 135, 158, 187, 140, 191, 124, 226, 39, 227, 72, 76, 94, 193, 86, 234,
                31, 211, 50, 171, 22, 149, 218, 21, 235, 27, 34, 59, 210, 79, 8, 195, 180, 35, 131,
                16, 246, 31, 116, 36, 179, 105, 43, 213, 69, 30, 6, 242, 27, 8, 153, 197, 108, 103,
                124, 143, 155, 201, 183, 227, 142, 193, 159, 40, 69, 71, 155, 216, 91, 71, 207, 49,
                15, 148, 212, 29, 88, 65, 163, 77, 42, 82, 182, 147, 122, 196, 93, 174, 91, 178,
                145, 241, 54, 138, 215, 39, 238, 185, 208, 104, 181, 70, 35, 205, 83, 208, 63
            ]
        );

        let cfg_str = r#"
            [agent]
            data_storage = "/var/lib/cosmian_vm/"
            host = "127.0.0.1"
            port = 5355
            ssl_certificate = "/data/cert.pem"
            ssl_private_key = "/data/key.pem"
            "#;

        let config: CosmianVmAgent = toml::from_str(cfg_str).unwrap();

        assert_eq!(config.ssl_certificate(), PathBuf::from("/data/cert.pem"));
        assert_eq!(config.ssl_private_key(), PathBuf::from("/data/key.pem"));
        assert_eq!(config.app_storage(), None);
    }
}
