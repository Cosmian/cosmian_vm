use sha1::{Digest, Sha1};
use std::path::Path;
use std::{fs, io};
use walkdir::DirEntry;

use crate::errors::Error;

const IMA_ASCII_PATH: &str = "/sys/kernel/security/ima/ascii_runtime_measurements";
const IMA_BINARY_PATH: &str = "/sys/kernel/security/ima/binary_runtime_measurements";

pub struct ImaEntry {
    pub pcr: u8,
    pub template_hash: String,
    pub template_id: String,
    pub filedata_hash: String,
    pub filename_hint: String,
}

impl ImaEntry {
    pub fn parse(line: &str) -> Result<ImaEntry, Error> {
        // Example of a line:
        // 10 479a8012721c06d45aedba1791ffab7d995ad30f ima-ng sha1:4f509d391aa126829f746cc3961dc39ffbef21ab /usr/lib/x86_64-linux-gnu/liblzma.so.5.2.5
        let split: Vec<&str> = line.split_whitespace().collect();

        let pcr = split.first().ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 0)".to_string(),
        ))?;

        let template_hash = split.get(1).ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 1)".to_string(),
        ))?;

        let template_id = split.get(2).ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 2)".to_string(),
        ))?;

        let filedata_hash = split.get(3).ok_or(Error::ImaParsingError(
            "Ima entry line malformed (index: 3)".to_string(),
        ))?;

        Ok(ImaEntry {
            pcr: pcr.parse::<u8>()?,
            template_hash: template_hash.to_string(),
            template_id: template_id.to_string(),
            filedata_hash: filedata_hash
                .strip_prefix("sha1:")
                .ok_or(Error::ImaParsingError(
                    "Ima entry `filedata_hash` is malformed (missing prefix `sha1:`)".to_string(),
                ))?
                .to_string(),
            filename_hint: line
                .get(
                    (pcr.len()
                        + template_hash.len()
                        + template_id.len()
                        + filedata_hash.len()
                        + 4)..,
                )
                .ok_or(Error::ImaParsingError(
                    "Ima entry line malformed (index: 3)".to_string(),
                ))?
                .to_string(),
        })
    }
}

#[inline(always)]
pub fn hash_file(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let _ = io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_vec())
}

pub fn filter_whilelist(entry: &DirEntry) -> bool {
    _filter_whilelist(entry).unwrap_or(false)
}

const BASE_EXCLUDE_DIRS: [&str; 8] = [
    "/sys/",
    "/run/",
    "/proc/",
    "/lost+found/",
    "/dev/",
    "/media/",
    "/var/",
    "/tmp/",
];

pub fn _filter_whilelist(entry: &DirEntry) -> Result<bool, Error> {
    // Do not keep files in some folders
    if BASE_EXCLUDE_DIRS
        .iter()
        .any(|exclude_dir| entry.path().starts_with(exclude_dir))
    {
        return Ok(false);
    }

    Ok(true)
}

pub fn read_ima_ascii() -> Result<String, Error> {
    Ok(fs::read_to_string(IMA_ASCII_PATH)?)
}

pub fn read_ima_binary() -> Result<Vec<u8>, Error> {
    Ok(fs::read(IMA_BINARY_PATH)?)
}

pub fn parse_ima_ascii(raw_ima: &str) -> Result<Vec<ImaEntry>, Error> {
    let mut ima = vec![];
    for line in raw_ima.lines() {
        ima.push(ImaEntry::parse(line)?)
    }
    Ok(ima)
}
