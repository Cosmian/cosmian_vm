use serde::{Deserialize, Serialize};

use crate::error::Error;

const SEPARATOR: char = '\x0C';

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SnapshotEntry {
    pub hash: Vec<u8>,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Snapshot {
    pub entries: Vec<SnapshotEntry>,
}

impl From<&SnapshotEntry> for String {
    fn from(val: &SnapshotEntry) -> Self {
        format!(r"{}{SEPARATOR}{}", hex::encode(&val.hash), val.path)
    }
}

impl From<Snapshot> for String {
    fn from(val: Snapshot) -> Self {
        val.entries
            .iter()
            .map(String::from)
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl TryFrom<&str> for Snapshot {
    type Error = Error;

    fn try_from(data: &str) -> Result<Self, Error> {
        let mut snapshot = Snapshot { entries: vec![] };

        for line in data.lines() {
            let mut s = line.split(SEPARATOR);
            snapshot.entries.push(SnapshotEntry {
                hash: hex::decode(s.next().ok_or(Error::ParsingError(format!(
                    "Hash field missing in line: {line}"
                )))?)?,
                path: s
                    .next()
                    .ok_or(Error::ParsingError(format!(
                        "Path field missing in line: {line}"
                    )))?
                    .to_string(),
            });
        }

        Ok(snapshot)
    }
}
