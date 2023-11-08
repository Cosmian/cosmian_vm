use serde::{Deserialize, Deserializer, Serialize, Serializer};

use tee_attestation::TeeMeasurement;

/// Serializes `buffer` to a lowercase hex string.
pub fn as_base64<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(buffer))
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
pub fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| hex::decode(string).map_err(|err| Error::custom(err.to_string())))
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SnapshotFilesEntry {
    #[serde(
        serialize_with = "as_base64",
        deserialize_with = "from_base64",
        rename = "h"
    )]
    pub hash: Vec<u8>,
    #[serde(rename = "p")]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SnapshotFiles(pub Vec<SnapshotFilesEntry>);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CosmianVmSnapshot {
    pub filehashes: SnapshotFiles,
    pub measurement: TeeMeasurement,
}
