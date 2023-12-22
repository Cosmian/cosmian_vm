use std::{
    collections::HashSet,
    fs,
    io::{BufRead, BufReader},
};

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

/// Read the first line of the ascii IMA values (probably: boot_aggregate)
pub fn read_ima_ascii_first_line() -> Result<String, Error> {
    let ima_file = std::fs::File::open(IMA_ASCII_PATH)?;
    let reader = BufReader::new(ima_file);
    let ima_first_line = reader
        .lines()
        .next()
        .ok_or_else(|| Error::Unexpected("Event log is empty".to_owned()))??;

    Ok(ima_first_line)
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
    pub template_name: ImaTemplate,
    pub filedata_hash_method: ImaHashMethod,
    pub filedata_hash: Vec<u8>,
    pub filename_hint: String,
    pub file_signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum ImaHashMethod {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum ImaTemplate {
    #[serde(rename = "ima")]
    /// template-hash: sha1 hash(filedata-hash, filename-hint)
    /// filedata-hash: sha1 hash(filedata)
    /// Example: 10 2c7020ad8cab6b7419e4973171cb704bdbf52f77 ima e09e048c48301268ff38645f4c006137e42951d0 /init
    Ima,
    #[serde(rename = "ima-ng")]
    /// template-hash: sha1 hash(filedata-hash length, filedata-hash, pathname length, pathname)
    /// filedata-hash: sha256 hash(filedata)
    /// Example: 10 8b1683287f61f96e5448f40bdef6df32be86486a ima-ng sha256:efdd249edec97caf9328a4a01baa99b7d660d1afc2e118b69137081c9b689954 /init
    ImaNg,
    #[serde(rename = "ima-sig")]
    /// ima-sig' template (same format as ima-ng, but with an appended signature when present)
    /// Example: 10 f63c10947347c71ff205ebfde5971009af27b0ba ima-sig sha256:6c118980083bccd259f069c2b3c3f3a2f5302d17a685409786564f4cf05b3939 /usr/lib64/libgspell-1.so.1.0.0   0302046e6c10460100aa43a4b1136f45735669632ad ...
    ImaSig,
}

impl TryFrom<&str> for ImaTemplate {
    type Error = Error;

    fn try_from(string: &str) -> Result<Self, Error> {
        match string {
            "ima" => Ok(ImaTemplate::Ima),
            "ima-ng" => Ok(ImaTemplate::ImaNg),
            "ima-sig" => Ok(ImaTemplate::ImaSig),
            _ => Err(Error::Parsing(format!(
                "Unsupported '{string}' ima template",
            ))),
        }
    }
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
        // 10 2c7020ad8cab6b7419e4973171cb704bdbf52f77 ima e09e048c48301268ff38645f4c006137e42951d0 /init
        // 10 479a8012721c06d45aedba1791ffab7d995ad30f ima-ng sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5
        // 10 479a8012721c06d45aedba1791ffab7d995ad30f ima-sig sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5 0302046e6c10460100aa43a4b1136f45735669632a
        let split: Vec<&str> = line.split_whitespace().collect();

        // The filename_hint can't contain whitespaces. If the filename contains file, they are replace by '_' before being inserted in the IMA
        // We can therefore simply split the line using the whitespaces

        let pcr = split.first().ok_or(Error::ImaParsing(
            "Ima entry line malformed (index: 0)".to_string(),
        ))?;

        let template_hash = split.get(1).ok_or(Error::ImaParsing(
            "Ima entry line malformed (index: 1)".to_string(),
        ))?;

        let template_name = ImaTemplate::try_from(*split.get(2).ok_or(Error::ImaParsing(
            "Ima entry line malformed (index: 2)".to_string(),
        ))?)?;

        let raw_filedata_hash = split.get(3).ok_or(Error::ImaParsing(
            "Ima entry line malformed (index: 3)".to_string(),
        ))?;

        let (filedata_hash_method, filedata_hash) = if template_name == ImaTemplate::Ima {
            (ImaHashMethod::Sha1, &raw_filedata_hash[..])
        } else if raw_filedata_hash.starts_with("sha1:") {
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

        let filename_hint = split
            .get(4)
            .ok_or(Error::ImaParsing(
                "Ima entry line malformed (index: 4)".to_string(),
            ))?
            .to_string();

        let file_signature = if template_name == ImaTemplate::ImaSig && split.len() == 6 {
            Some(hex::decode(split.get(5).ok_or(Error::ImaParsing(
                "Ima entry line malformed (index: 5)".to_string(),
            ))?)?)
        } else {
            None
        };

        if template_name == ImaTemplate::ImaSig {
            if split.len() > 6 {
                return Err(Error::ImaParsing(format!(
                    "Extra field detected: {}",
                    split.len()
                )));
            }
        } else if split.len() > 5 {
            return Err(Error::ImaParsing(format!(
                "Extra field detected: {}",
                split.len()
            )));
        }

        Ok(ImaEntry {
            pcr: pcr.parse::<u32>()?,
            template_hash: hex::decode(template_hash)?,
            template_name,
            filedata_hash_method,
            filedata_hash: hex::decode(filedata_hash)?,
            filename_hint,
            file_signature,
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
            let template_name = &data
                .get(cursor..(cursor + event.name_length as usize))
                .ok_or(Error::ImaParsing(
                    "Not enough bytes in the buffer to parse IMA entry template name".to_string(),
                ))?;
            let template_name = String::from_utf8_lossy(template_name).to_string();
            let template_name = ImaTemplate::try_from(template_name.as_ref())?;

            cursor += event.name_length as usize;

            // Parse the length of the template data
            let length = bincode::deserialize::<u32>(
                data.get(cursor..(cursor + (u32::BITS as usize / 8)))
                    .ok_or(Error::ImaParsing(
                        "Not enough bytes in the buffer to parse IMA entry length".to_string(),
                    ))?,
            )? as usize;
            cursor += u32::BITS as usize / 8;

            // Parse the template data
            let template_data = &data
                .get(cursor..(cursor + length))
                .ok_or(Error::ImaParsing(format!(
                    "Not enough bytes in the buffer to parse IMA entry template: {} > {}",
                    cursor + length,
                    data.len(),
                )))?;
            cursor += length;

            // From the template data, parse the size of the hash field
            let mut template_cursor = 0;
            let hash_length =
                bincode::deserialize::<u32>(&template_data[0..(u32::BITS as usize / 8)])? as usize;
            template_cursor += u32::BITS as usize / 8;

            // From the template data, parse the hash field
            let hash = &template_data[template_cursor..(template_cursor + hash_length)];
            template_cursor += hash_length;

            // From the template data, parse the size of the file field
            let hint_length = bincode::deserialize::<u32>(
                &template_data[template_cursor..(template_cursor + (u32::BITS as usize / 8))],
            )? as usize;
            template_cursor += u32::BITS as usize / 8;

            // From the template data, parse the file field
            let hint = &template_data[template_cursor..(template_cursor + hint_length - 1)];

            template_cursor += hint_length;

            // From the template data, parse the signature if any
            let sig = if template_name == ImaTemplate::ImaSig {
                let sig_length = bincode::deserialize::<u32>(
                    &template_data[template_cursor..(template_cursor + (u32::BITS as usize / 8))],
                )? as usize;
                template_cursor += u32::BITS as usize / 8;

                if sig_length != 0 {
                    let sig = &template_data[template_cursor..(template_cursor + sig_length)];
                    template_cursor += sig_length;

                    Some(sig.to_vec())
                } else {
                    None
                }
            } else {
                None
            };

            if template_cursor != length {
                return Err(Error::ImaParsing(format!(
                    "Extra bytes {} unparsed in buffer",
                    template_cursor - length
                )));
            }

            let (filedata_hash_method, hash) = if template_name == ImaTemplate::Ima {
                (ImaHashMethod::Sha1, hash)
            } else if hash.starts_with(b"sha1:") {
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
                file_signature: sig,
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

    /// Return the couple (file, hash) from the current IMA list not present in the given snapshot
    pub fn compare(&self, snapshot: &HashSet<(String, Vec<u8>)>) -> Ima {
        // Pre-process the snapshot to be use later:
        // - Replace all whitespaces in filenames by underscores (to fit IMA filename-hint)
        let snapshot_ima = snapshot
            .iter()
            .map(|(path, hash)| (path.replace(' ', "_"), hash))
            .collect::<HashSet<_>>();

        Ima {
            entries: self
                .entries
                .iter()
                .filter_map(|entry| {
                    (entry.filename_hint != "boot_aggregate"
                        // The kernel prohibits writing and executing a file concurrently.
                        // Other files can be read and written concurrently:
                        // - "open_writers" file already open for write, is opened for read
                        // - "open_reader" file already open for read is opened for write
                        // In these two cases, IMA cannot know what is actually read,
                        // and invalidates the measurement with all zeros
                        && entry.filedata_hash != vec![0; entry.filedata_hash.len()]
                        && !snapshot_ima.contains(&(entry.filename_hint.clone(), &entry.filedata_hash)))
                    .then_some(entry.clone())
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_ascii_ima_parse_template_ima() {
        let line = "10 2c7020ad8cab6b7419e4973171cb704bdbf52f77 ima e09e048c48301268ff38645f4c006137e42951d0 /init";
        let ima = ImaEntry::try_from(line).expect("Can't parse IMA file");
        assert_eq!(
            ima,
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("2c7020ad8cab6b7419e4973171cb704bdbf52f77").unwrap(),
                template_name: ImaTemplate::Ima,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("e09e048c48301268ff38645f4c006137e42951d0").unwrap(),
                filename_hint: "/init".to_string(),
                file_signature: None
            }
        );
    }

    #[test]
    fn test_ascii_ima_parse_template_imang() {
        // Test sha1
        let line = "10 a84ff12e903a050abff2f336292d8318e7430a89 ima-ng sha1:f4107171a62db56e4949c30fca97d09f7550aac5 /usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko
        ";
        let ima = ImaEntry::try_from(line).expect("Can't parse IMA file");
        assert_eq!(
            ima,
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("a84ff12e903a050abff2f336292d8318e7430a89").unwrap(),
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("f4107171a62db56e4949c30fca97d09f7550aac5").unwrap(),
                filename_hint: "/usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko"
                    .to_string(),
                file_signature: None
            }
        );

        // Test sha256
        let line = "10 ab6cd51adcff9f5ca04ff9e23f35099125073bae ima-ng sha256:0e340b558513b76fbe6e5a6b2a03f3e8f42257b95e6ed980697baf4680e8eeeb boot_aggregate";
        let ima = ImaEntry::try_from(line).expect("Can't parse IMA file");
        assert_eq!(
            ima,
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("ab6cd51adcff9f5ca04ff9e23f35099125073bae").unwrap(),
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha256,
                filedata_hash: hex::decode(
                    "0e340b558513b76fbe6e5a6b2a03f3e8f42257b95e6ed980697baf4680e8eeeb"
                )
                .unwrap(),
                filename_hint: "boot_aggregate".to_string(),
                file_signature: None
            }
        );

        // Test sha512 (+ extra whitespaces)
        let line = "10    0b800bc9073bea5973484e047a12b66fcf78b616      ima-ng   sha512:d47b283c5f72fcd3d0655c9cbb0e7a175bb0d424d7b56b0a437f29ed4915fd4ec1d6712346a5ede957de265bee36792dc4660b2cac1161f471dd8f7ec27785bd     /usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko";
        let ima = ImaEntry::try_from(line).expect("Can't parse IMA file");
        assert_eq!(
            ima,
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("0b800bc9073bea5973484e047a12b66fcf78b616").unwrap(),
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha512,
                filedata_hash: hex::decode(
                    "d47b283c5f72fcd3d0655c9cbb0e7a175bb0d424d7b56b0a437f29ed4915fd4ec1d6712346a5ede957de265bee36792dc4660b2cac1161f471dd8f7ec27785bd"
                )
                .unwrap(),
                filename_hint: "/usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko".to_string(),
                file_signature: None
            }
        );

        // Test sha384
        let line = "10 0b800bc9073bea5973484e047a12b66fcf78b616 ima-ng sha384:d47b283c5f72fcd3d0655c9cbb0e7a175bb0d424d7b56b0a437f29ed4915fd4ec1d6712346a5ede957de265bee36792d /usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko";
        assert!(ImaEntry::try_from(line).is_err()); // Not supported hash
    }

    #[test]
    fn test_ascii_ima_parse_template_imasig() {
        let line = "10 479a8012721c06d45aedba1791ffab7d995ad30f ima-sig sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5 0302046e6c10460100aa43a4b1136f45735669632a";
        let ima = ImaEntry::try_from(line).expect("Can't parse IMA file");
        assert_eq!(
            ima,
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("479a8012721c06d45aedba1791ffab7d995ad30f").unwrap(),
                template_name: ImaTemplate::ImaSig,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("4f509d391aa126829f746cc3961dc39ffbef21ab").unwrap(),
                filename_hint: "/usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5".to_string(),
                file_signature: Some(
                    hex::decode("0302046e6c10460100aa43a4b1136f45735669632a").unwrap()
                )
            }
        );

        let line = "10 479a8012721c06d45aedba1791ffab7d995ad30f ima-sig sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5";
        let ima = ImaEntry::try_from(line).expect("Can't parse IMA file");
        assert_eq!(
            ima,
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("479a8012721c06d45aedba1791ffab7d995ad30f").unwrap(),
                template_name: ImaTemplate::ImaSig,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("4f509d391aa126829f746cc3961dc39ffbef21ab").unwrap(),
                filename_hint: "/usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5".to_string(),
                file_signature: None
            }
        );
    }

    #[test]
    fn test_binary_ima_parse_template_imang() {
        let raw_ima = include_bytes!("../data/ima.bin");
        let ima = Ima::try_from(raw_ima.as_slice()).expect("Can't parse IMA file");

        assert_eq!(
            ima.entries[0],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("470f3a07c979dfda23c75b4865955df704e49e4b").unwrap(),
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("3d993d6bfad2564637310b643c404f54d23b85e2").unwrap(),
                filename_hint: "boot_aggregate".to_string(),
                file_signature: None
            }
        );

        assert_eq!(
            ima.entries[1],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("a84ff12e903a050abff2f336292d8318e7430a89").unwrap(),
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("f4107171a62db56e4949c30fca97d09f7550aac5").unwrap(),
                filename_hint: "/usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko"
                    .to_string(),
                file_signature: None
            }
        );

        assert_eq!(ima.entries.len(), 446)
    }

    #[test]
    fn test_binary_ima_parse_template_imasig() {
        let raw_ima = include_bytes!("../data/ima-sig.bin");
        let ima = Ima::try_from(raw_ima.as_slice()).expect("Can't parse IMA file");

        assert_eq!(
            ima.entries[0],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("0d6280b024aa96ad8cfbdd417fb8caa9b24fe988").unwrap(),
                template_name: ImaTemplate::ImaSig,
                filedata_hash_method: ImaHashMethod::Sha256,
                filedata_hash: hex::decode(
                    "68c995e4fc8cc809326c0dc95e6cce6eb43c5c788e276b415b3977efc24876e0"
                )
                .unwrap(),
                filename_hint: "boot_aggregate".to_string(),
                file_signature: None
            }
        );

        assert_eq!(
            ima.entries[1],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("6cf1a382b30d3c0884e649dfa32efdb7ce7832cb").unwrap(),
                template_name: ImaTemplate::ImaSig,
                filedata_hash_method: ImaHashMethod::Sha256,
                filedata_hash: hex::decode(
                    "65f5ef64e1c9cb46447e157f10417d3b3bf68ddbd318e71f0b3ce78c69eaf121"
                )
                .unwrap(),
                filename_hint: "/etc/systemd/system.conf".to_string(),
                file_signature: None
            }
        );

        assert_eq!(
            ima.entries[16],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("412dd9c12c28f32ad6c9b1531b259ae2e0374f60").unwrap(),
                template_name: ImaTemplate::ImaSig,
                filedata_hash_method: ImaHashMethod::Sha256,
                filedata_hash: hex::decode("0e71237a2db762d0b43a7b3d9aedd2210ff2327bfb9c984fc2557042a2d5c103").unwrap(),
                filename_hint: "/usr/lib64/systemd/libsystemd-shared-252.so".to_string(),
                file_signature: Some(hex::decode("030204577055f401006a5d222918b0222967185cd5d52d24f7d779280034932dc735850ab0d259b9b16e2e4d34484642d22bbd6a3d7b3c0c2dd002748ef16eb35ff5ccc720a7ad93d8baa05846d1cb6b92713044286f380c807dc9d2f181eebb1e3b34738bf0d505ae13824d9a31f73f6475dcf3848d6d1c38263b5bbe27cf1e96b85e24f55f0ad5911670ccd794587413e4d2f57aaf5df03a321af5c90d93126b9f1d05ede7b3d7d6f3f45ed7016b4e7b2cda66bda0c9cf86f9f2587536ef1a8fcf7ddacb52b457be600489dbb1046ae346346531d5e6953cb86f6697498a5973d738d4162e86078473a4a8d096e01bd98cad82b52edf3e2bc43e8149de2f66f7b67593ee5cc6a617").unwrap()),
            }
        );

        assert_eq!(ima.entries.len(), 401)
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
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("3d993d6bfad2564637310b643c404f54d23b85e2").unwrap(),
                filename_hint: "boot_aggregate".to_string(),
                file_signature: None
            }
        );

        assert_eq!(
            ima.entries[1],
            ImaEntry {
                pcr: 10,
                template_hash: hex::decode("a84ff12e903a050abff2f336292d8318e7430a89").unwrap(),
                template_name: ImaTemplate::ImaNg,
                filedata_hash_method: ImaHashMethod::Sha1,
                filedata_hash: hex::decode("f4107171a62db56e4949c30fca97d09f7550aac5").unwrap(),
                filename_hint: "/usr/lib/modules/6.2.0-1018-gcp/kernel/fs/autofs/autofs4.ko"
                    .to_string(),
                file_signature: None
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
        let ima = Ima::try_from(raw_ima.as_slice()).expect("Can't parse IMA file");
        let snapshot = HashSet::from([
            (
                "/usr/lib/systemd/system-generators/systemd-debug-generator".to_owned(),
                hex::decode("545cac360cece7aa86f73c4dc6e518a2a25ffe1c").unwrap(),
            ),
            (
                "/usr/libexec/netplan/generate2".to_owned(),
                hex::decode("ad65f41a5efd4ad27bd5d1d74ad5f60917677611").unwrap(),
            ),
            (
                "/usr/bin/aa-exec".to_owned(),
                hex::decode("3259fe4d0ce59b251d644eb52ca72280b4f17602").unwrap(),
            ),
        ]);

        let ret = ima.compare(&snapshot);

        assert_eq!(ret.entries.len(), 444);

        let entries: HashSet<_> = ret.entries.iter().collect();

        assert!(&entries
            .get(&ImaEntry {
                filedata_hash: hex::decode("545cac360cece7aa86f73c4dc6e518a2a25ffe1c").unwrap(),
                filename_hint: "/usr/lib/systemd/system-generators/systemd-debug-generator"
                    .to_string(),
                pcr: 10,
                filedata_hash_method: ImaHashMethod::Sha1,
                template_hash: hex::decode("e01a2b6dfdc98466531ec38dda66e641ded2525a").unwrap(),
                template_name: ImaTemplate::ImaNg,
                file_signature: None
            })
            .is_none()); // present in the snapshot and in ima
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
                template_name: ImaTemplate::ImaNg,
                file_signature: None
            })
            .is_some()); // not present in the snapshot but present in ima
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
                template_name: ImaTemplate::ImaNg,
                file_signature: None
            })
            .is_some()); // present in the snapshot but not with that hash value
    }

    #[test]
    fn test_compare_with_whitespace() {
        let line = "10 479a8012721c06d45aedba1791ffab7d995ad30f ima-sig sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /home/cosmian/cosmian_vm_agent_";
        let ima = Ima::try_from(line).expect("Can't parse IMA file");

        let ret = ima.compare(&HashSet::from([(
            "/home/cosmian/cosmian_vm agent ".to_owned(),
            hex::decode("4f509d391aa126829f746cc3961dc39ffbef21ab").unwrap(),
        )]));

        assert_eq!(ret.entries.len(), 0);
    }
}
