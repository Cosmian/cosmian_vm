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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ImaEntry {
    pub pcr: u32,
    pub template_hash: Vec<u8>,
    pub template_name: String,
    pub filedata_hash_method: ImaHashMethod,
    pub filedata_hash: Vec<u8>,
    pub filename_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum ImaHashMethod {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ima {
    pub entries: Vec<ImaEntry>,
}

const IMA_DEFAULT_PCR_ID: u32 = 10;
const IMA_DEFAULT_FILEHASH_FUNCTION: ImaHashMethod = ImaHashMethod::Sha1;

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

        let (filedata_hash_method, filedata_hash) = if raw_filedata_hash.starts_with("sha1:") {
            (
                ImaHashMethod::Sha1,
                &raw_filedata_hash[raw_filedata_hash.len() - 40..],
            )
        } else if raw_filedata_hash.starts_with("sha256:") {
            (
                ImaHashMethod::Sha256,
                &raw_filedata_hash[raw_filedata_hash.len() - 64..],
            )
        } else if raw_filedata_hash.starts_with("sha512:") {
            (
                ImaHashMethod::Sha512,
                &raw_filedata_hash[raw_filedata_hash.len() - 128..],
            )
        } else {
            return Err(Error::NotImplemented("File hash not supported".to_owned()));
        };

        Ok(ImaEntry {
            pcr: pcr.parse::<u32>()?,
            template_hash: hex::decode(template_hash)?,
            template_name: template_name.to_string(),
            filedata_hash_method,
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
                // - 'ima-sig' template (same format as ima-ng, but with an appended signature when present)
                // - original 'ima' template (no 'sha1:' prefix)
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

            let (filedata_hash_method, hash) = if hash.starts_with(b"sha1:") {
                (ImaHashMethod::Sha1, &hash[hash.len() - 20..])
            } else if hash.starts_with(b"sha256:") {
                (ImaHashMethod::Sha256, &hash[hash.len() - 32..])
            } else if hash.starts_with(b"sha512:") {
                (ImaHashMethod::Sha512, &hash[hash.len() - 64..])
            } else {
                return Err(Error::NotImplemented("File hash not supported".to_owned()));
            };

            ima.entries.push(ImaEntry {
                pcr: event.pcr,
                template_hash: event.digest.to_vec(),
                template_name,
                filedata_hash_method,
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

    /// Return the id of the extended pcr value
    ///
    /// If the IMA is empty, the default value is: `IMA_DEFAULT_PCR_ID`
    pub fn pcr_id(&self) -> u32 {
        self.entries.get(0).map_or(IMA_DEFAULT_PCR_ID, |e| e.pcr)
    }

    /// Return the hash method used to hash the files
    ///
    /// If the IMA is empty, the default value is: `ImaHashMethod::Sha1`
    pub fn hash_file_method(&self) -> ImaHashMethod {
        self.entries
            .get(0)
            .map_or(IMA_DEFAULT_FILEHASH_FUNCTION, |e| {
                e.filedata_hash_method.clone()
            })
    }

    /// Return the couple (hash, file) from the current IMA list not present in the given snapshot
    pub fn compare(&self, snapshot: &SnapshotFiles) -> Ima {
        Ima {
            entries: self
                .entries
                .iter()
                .filter_map(|entry| {
                    (entry.filename_hint != "boot_aggregate"
                        /* The kernel prohibits writing and executing a file concurrently.
                        Other files can be read and written concurrently:
                        - "open_writers" file already open for write, is opened for read
                        - "open_reader" file already open for read is opened for write
                        In these two cases, IMA cannot know what is actually read,
                        and invalidates the measurement with all zeros */
                        && entry.filedata_hash != vec![0; entry.filedata_hash.len()]
                        && !snapshot.0.contains(&SnapshotFilesEntry {
                            hash: entry.filedata_hash.clone(),
                            path: entry.filename_hint.clone(),
                        }))
                    .then_some(entry.clone())
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
                filedata_hash_method: ImaHashMethod::Sha1,
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
                filedata_hash_method: ImaHashMethod::Sha1,
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
                filedata_hash_method: ImaHashMethod::Sha1,
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
                filedata_hash_method: ImaHashMethod::Sha1,
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

        assert_eq!(ret.entries.len(), 16);

        let entries: HashSet<_> = ret.entries.iter().collect();

        assert!(&entries
            .get(&ImaEntry {
                filedata_hash: hex::decode("ad65f41a5efd4ad27bd5d1d74ad5f60917677611").unwrap(),
                filename_hint: "/usr/libexec/netplan/generate".to_string(),
                pcr: 10,
                filedata_hash_method: ImaHashMethod::Sha1,
                template_hash: [
                    27, 57, 158, 185, 6, 218, 99, 164, 90, 172, 217, 96, 154, 254, 6, 69, 128, 23,
                    236, 175
                ]
                .to_vec(),
                template_name: "ima-ng".to_string()
            })
            .is_some()); // not present in the snapshot
        assert!(&entries
            .get(&ImaEntry {
                filedata_hash: hex::decode("5659fe4d0ce59b251d644eb52ca72280b4f17602").unwrap(),
                filename_hint: "/usr/bin/aa-exec".to_string(),
                pcr: 10,
                filedata_hash_method: ImaHashMethod::Sha1,
                template_hash: [
                    215, 207, 23, 146, 41, 2, 129, 4, 150, 89, 180, 105, 253, 171, 147, 29, 9, 13,
                    207, 34
                ]
                .to_vec(),
                template_name: "ima-ng".to_string()
            })
            .is_some()); // present in the snapshot but not with that hash value
    }
}
