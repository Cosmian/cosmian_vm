use ima::ima::ImaHashMethod;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};
use std::path::Path;
use std::{fs, io};
use walkdir::DirEntry;

use crate::error::Error;

#[inline(always)]
pub fn hash_file(path: &Path, hash_method: &ImaHashMethod) -> Result<Vec<u8>, Error> {
    let mut file = fs::File::open(path)?;

    match hash_method {
        ImaHashMethod::Sha1 => {
            let mut hasher = Sha1::new();
            let _ = io::copy(&mut file, &mut hasher)?;
            Ok(hasher.finalize().to_vec())
        }
        ImaHashMethod::Sha256 => {
            let mut hasher = Sha256::new();
            let _ = io::copy(&mut file, &mut hasher)?;
            Ok(hasher.finalize().to_vec())
        }
        ImaHashMethod::Sha512 => {
            let mut hasher = Sha512::new();
            let _ = io::copy(&mut file, &mut hasher)?;
            Ok(hasher.finalize().to_vec())
        }
    }
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

pub fn start_detached_app() {}
