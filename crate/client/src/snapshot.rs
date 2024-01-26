use serde::de::{SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Deserializer, Serialize};

use std::collections::HashSet;
use std::fmt;
use tee_attestation::TeePolicy;
use tpm_quote::policy::TpmPolicy;

/// Serializes a `HashSet<(String, Vec<u8>)>` to a json string.
pub fn serialize_hex<S>(
    buffer: &HashSet<(String, Vec<u8>)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_seq(Some(buffer.len()))?;
    for item in buffer {
        map.serialize_element(&(item.0.clone(), &hex::encode(&item.1)))?;
    }
    map.end()
}

struct HashSetDeserializer;

impl<'de> Visitor<'de> for HashSetDeserializer {
    type Value = HashSet<(String, Vec<u8>)>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("HashSet<(String, Vec<u8>)> key value sequence.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        use serde::de::Error;

        let mut new_obj = HashSet::<(String, Vec<u8>)>::new();
        while let Some((path, hash)) = seq.next_element()? {
            new_obj.insert((
                path,
                hex::decode::<String>(hash).map_err(|err| Error::custom(err.to_string()))?,
            ));
        }

        Ok(new_obj)
    }
}

/// Deserializes a json string to a `HashSet<(String, Vec<u8>)>`.
pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<HashSet<(String, Vec<u8>)>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(HashSetDeserializer)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnapshotFiles(
    #[serde(serialize_with = "serialize_hex", deserialize_with = "deserialize_hex")]
    pub  HashSet<(String, Vec<u8>)>,
);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CosmianVmSnapshot {
    pub tee_policy: TeePolicy,
    pub tpm_policy: Option<TpmPolicy>,
    pub filehashes: Option<SnapshotFiles>,
}
