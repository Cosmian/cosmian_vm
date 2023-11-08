use std::fs;

use cosmian_vm_client::snapshot::{SnapshotFiles, SnapshotFilesEntry};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::error::Error;

const EVENT_ENTRY_SIZE: usize = 28;
const IMA_ASCII_PATH: &str = "/sys/kernel/security/ima/ascii_runtime_measurements";
const IMA_BINARY_PATH: &str = "/sys/kernel/security/ima/binary_runtime_measurements";

/// Read the ascii IMA values
pub fn read_ima_ascii() -> Result<String, Error> {
    Ok(fs::read_to_string(IMA_ASCII_PATH)?)
}

/// Read the binary IMA values
pub fn read_ima_binary() -> Result<Vec<u8>, Error> {
    Ok(fs::read(IMA_BINARY_PATH)?)
}

#[repr(C)]
#[derive(Debug, Clone, Deserialize, Serialize)]
struct EventHeaderEntry {
    pub pcr: u32,
    pub digest: [u8; 20],
    pub name_length: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ImaEntry {
    pub pcr: u32,
    pub template_hash: Vec<u8>,
    pub template_name: String,
    pub filedata_hash: Vec<u8>,
    pub filename_hint: String,
}

impl TryFrom<&str> for ImaEntry {
    type Error = Error;

    /// Convert a string line to a ImaEntry
    fn try_from(line: &str) -> Result<Self, Error> {
        // Example of a line:
        // 10 479a8012721c06d45aedba1791ffab7d995ad30f ima-ng sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5
        let split: Vec<&str> = line.split_whitespace().collect();

        let pcr = split.first().ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 0)".to_string(),
        ))?;

        let template_hash = split.get(1).ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 1)".to_string(),
        ))?;

        let template_name = split.get(2).ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 2)".to_string(),
        ))?;

        let raw_filedata_hash = split.get(3).ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 3)".to_string(),
        ))?;

        let filedata_hash = if raw_filedata_hash.starts_with("sha1:") {
            &raw_filedata_hash[raw_filedata_hash.len() - 40..]
        } else if raw_filedata_hash.starts_with("sha256:") {
            &raw_filedata_hash[raw_filedata_hash.len() - 64..]
        } else if raw_filedata_hash.starts_with("sha384:") {
            &raw_filedata_hash[raw_filedata_hash.len() - 96..]
        } else {
            return Err(Error::NotImplemented("File hash not supported".to_owned()));
        };

        Ok(ImaEntry {
            pcr: pcr.parse::<u32>()?,
            template_hash: hex::decode(template_hash)?,
            template_name: template_name.to_string(),
            filedata_hash: hex::decode(filedata_hash)?,
            filename_hint: line
                .get(
                    (pcr.len()
                        + template_hash.len()
                        + template_name.len()
                        + raw_filedata_hash.len()
                        + 4)..,
                )
                .ok_or(Error::ImaParsingError(
                    "Ima entry line malformed (index: 4)".to_string(),
                ))?
                .to_string(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ima {
    pub entries: Vec<ImaEntry>,
}

impl TryFrom<&str> for Ima {
    type Error = Error;

    fn try_from(data: &str) -> Result<Self, Error> {
        let mut ima = vec![];
        for line in data.lines() {
            ima.push(ImaEntry::try_from(line)?)
        }
        Ok(Ima { entries: ima })
    }
}

impl TryFrom<&[u8]> for Ima {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Error> {
        let mut ima = Ima { entries: vec![] };
        let mut cursor = 0;
        while (cursor + EVENT_ENTRY_SIZE) < data.len() {
            // Parse the header (first 28 bytes of the ima entry)
            let event: EventHeaderEntry =
                bincode::deserialize(&data[cursor..(cursor + EVENT_ENTRY_SIZE)])?;

            cursor += EVENT_ENTRY_SIZE;

            // Parse the name of the template
            let template_name = &data[cursor..(cursor + event.name_length as usize)];
            let template_name = String::from_utf8_lossy(template_name).to_string();
            if template_name != "ima-ng" {
                // TODO: handle several templates
                return Err(Error::NotImplemented(format!(
                    "Template name '{template_name}' not supported"
                )));
            }

            cursor += event.name_length as usize;

            // Parse the length of the template data
            let length: u32 =
                bincode::deserialize(&data[cursor..(cursor + (u32::BITS as usize / 8))])?;
            cursor += u32::BITS as usize / 8;

            // Parse the template data
            let template_data = &data[cursor..(cursor + length as usize)];
            cursor += length as usize;

            // From the template data, parse the size of the hash field
            let mut template_cursor = 0;
            let hash_length: u32 =
                bincode::deserialize(&template_data[0..(u32::BITS as usize / 8)])?;
            template_cursor += u32::BITS as usize / 8;

            // From the template data, parse the hash field
            let hash = &template_data[template_cursor..(template_cursor + hash_length as usize)];
            template_cursor += hash_length as usize;

            // From the template data, parse the size of the file field
            let hint_length: u32 = bincode::deserialize(
                &template_data[template_cursor..(template_cursor + (u32::BITS as usize / 8))],
            )?;
            template_cursor += u32::BITS as usize / 8;

            // From the template data, parse the file field
            let hint =
                &template_data[template_cursor..(template_cursor + hint_length as usize - 1)];

            let hash = if hash.starts_with(b"sha1:") {
                &hash[hash.len() - 20..]
            } else if hash.starts_with(b"sha256:") {
                &hash[hash.len() - 32..]
            } else if hash.starts_with(b"sha384:") {
                &hash[hash.len() - 48..]
            } else {
                return Err(Error::NotImplemented("File hash not supported".to_owned()));
            };

            ima.entries.push(ImaEntry {
                pcr: event.pcr,
                template_hash: event.digest.to_vec(),
                template_name,
                filedata_hash: hash.to_vec(),
                filename_hint: String::from_utf8_lossy(hint).to_string(),
            });
        }

        Ok(ima)
    }
}

impl Ima {
    /// Compute the PCR value from the actual IMA list
    pub fn pcr_value(&self) -> Result<Vec<u8>, Error> {
        let mut old_entry = [0u8; 20];

        for entry in &self.entries {
            let mut hasher = Sha1::new();
            hasher.update(old_entry);
            hasher.update(&entry.template_hash);
            old_entry = hasher.finalize().into();
        }

        Ok(old_entry.to_vec())
    }

    /// Return the couple (hash, file) from the current IMA list not present in the given snapshot
    pub fn compare(&self, snapshot: &SnapshotFiles) -> SnapshotFiles {
        let mut ret = SnapshotFiles(vec![]);
        for entry in &self.entries {
            if entry.filename_hint == "boot_aggregate" {
                continue;
            }

            let found = snapshot.0.iter().any(|item| {
                (&item.hash, &item.path) == (&entry.filedata_hash, &entry.filename_hint)
            });

            if !found {
                ret.0.push(SnapshotFilesEntry {
                    hash: entry.filedata_hash.clone(),
                    path: entry.filename_hint.clone(),
                });
            }
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use cosmian_vm_client::snapshot::CosmianVmSnapshot;

    use super::*;

    #[test]
    fn test_binary_ima_parse() {
        let raw_ima = include_bytes!("../data/ima.bin");
        let ima = Ima::try_from(raw_ima.as_slice()).expect("Can't parse IMA file");

        assert_eq!(
            ima.entries[0],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("470f3a07c979dfda23c75b4865955df704e49e4b").unwrap(),
                template_name: "ima-ng".to_string(),
                filedata_hash: hex::decode("3d993d6bfad2564637310b643c404f54d23b85e2").unwrap(),
                filename_hint: "boot_aggregate".to_string()
            }
        );

        assert_eq!(
            ima.entries[1],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("a84ff12e903a050abff2f336292d8318e7430a89").unwrap(),
                template_name: "ima-ng".to_string(),
                filedata_hash: hex::decode("f4107171a62db56e4949c30fca97d09f7550aac5").unwrap(),
                filename_hint: "/usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko"
                    .to_string()
            }
        );

        assert_eq!(ima.entries.len(), 446)
    }

    #[test]
    fn test_ascii_ima_parse() {
        let raw_ima = include_str!("../data/ima.ascii");
        let ima = Ima::try_from(raw_ima).expect("Can't parse IMA file");

        assert_eq!(
            ima.entries[0],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("470f3a07c979dfda23c75b4865955df704e49e4b").unwrap(),
                template_name: "ima-ng".to_string(),
                filedata_hash: hex::decode("3d993d6bfad2564637310b643c404f54d23b85e2").unwrap(),
                filename_hint: "boot_aggregate".to_string()
            }
        );

        assert_eq!(
            ima.entries[1],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("a84ff12e903a050abff2f336292d8318e7430a89").unwrap(),
                template_name: "ima-ng".to_string(),
                filedata_hash: hex::decode("f4107171a62db56e4949c30fca97d09f7550aac5").unwrap(),
                filename_hint: "/usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko"
                    .to_string()
            }
        );

        assert_eq!(ima.entries.len(), 446)
    }

    #[test]
    fn test_pcr_value() {
        let raw_ima = include_bytes!("../data/ima.bin");
        let ima = Ima::try_from(raw_ima.as_slice()).expect("Can't parse IMA file");

        assert_eq!(
            ima.pcr_value().expect("Can't compute pcr value"),
            [
                211, 163, 104, 155, 152, 107, 49, 40, 63, 2, 43, 161, 0, 226, 91, 42, 50, 112, 192,
                218
            ]
        );
    }

    #[test]
    fn test_compare() {
        let raw_ima = include_bytes!("../data/ima.bin");
        let raw_snapshot = include_str!("../data/snapshot");
        let ima = Ima::try_from(raw_ima.as_slice()).expect("Can't parse IMA file");
        let snapshot: CosmianVmSnapshot =
            serde_json::from_str(raw_snapshot).expect("Can't parse snapshot file");

        let ret = ima.compare(&snapshot.filehashes);

        assert_eq!(ret.0.len(), 16);

        println!("{:?}", ret);

        assert_eq!(
            hex::encode(&ret.0[0].hash),
            "ad65f41a5efd4ad27bd5d1d74ad5f60917677611"
        );
        assert_eq!(ret.0[0].path, "/usr/libexec/netplan/generate"); // not present
        assert_eq!(
            hex::encode(&ret.0[5].hash),
            "5659fe4d0ce59b251d644eb52ca72280b4f17602"
        );
        assert_eq!(ret.0[5].path, "/usr/bin/aa-exec"); // present but not with that hash value
    }
}
