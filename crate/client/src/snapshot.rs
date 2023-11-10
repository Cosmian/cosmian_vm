use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use tee_attestation::TeeMeasurement;

use crate::error::Error;

const SEPARATOR: char = '\x0C';

/// Serializes `buffer` to a string.
pub fn serialize_snapshotfiles<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<SnapshotFiles>,
    S: Serializer,
{
    serializer.serialize_str(&String::from(buffer.as_ref()))
}

/// Deserializes a string to a `SnapshotFiles`.
pub fn deserialize_snapshotfiles<'de, D>(deserializer: D) -> Result<SnapshotFiles, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        SnapshotFiles::try_from(string.as_ref()).map_err(|err| Error::custom(err.to_string()))
    })
}

impl From<&SnapshotFilesEntry> for String {
    fn from(val: &SnapshotFilesEntry) -> Self {
        format!(r"{}{SEPARATOR}{}", hex::encode(&val.hash), val.path)
    }
}

impl From<&SnapshotFiles> for String {
    fn from(val: &SnapshotFiles) -> Self {
        val.0
            .iter()
            .map(String::from)
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl TryFrom<&str> for SnapshotFiles {
    type Error = Error;

    fn try_from(data: &str) -> Result<Self, Error> {
        let mut snapshot = SnapshotFiles(HashSet::new());

        for line in data.lines() {
            let mut s = line.split(SEPARATOR);
            snapshot.0.insert(SnapshotFilesEntry {
                hash: hex::decode(s.next().ok_or(Error::Default(format!(
                    "Hash field missing in line: {line}"
                )))?)?,
                path: s
                    .next()
                    .ok_or(Error::Default(format!(
                        "Path field missing in line: {line}"
                    )))?
                    .to_string(),
            });
        }

        Ok(snapshot)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub struct SnapshotFilesEntry {
    pub hash: Vec<u8>,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct SnapshotFiles(pub HashSet<SnapshotFilesEntry>);

// Needed for `serde` serialization convenience
impl AsRef<SnapshotFiles> for SnapshotFiles {
    fn as_ref(&self) -> &SnapshotFiles {
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CosmianVmSnapshot {
    #[serde(
        serialize_with = "serialize_snapshotfiles",
        deserialize_with = "deserialize_snapshotfiles"
    )]
    pub filehashes: SnapshotFiles,
    pub measurement: TeeMeasurement,
}
