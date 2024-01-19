use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use openssl::{
    bn::BigNumContext,
    ec::{EcGroup, EcKey, EcPoint},
    md::Md,
    nid::Nid,
    pkey::{Id, PKey, Private, Public},
    pkey_ctx::PkeyCtx,
    rand::rand_bytes,
    sha::Sha256,
    symm::{decrypt_aead, encrypt_aead, Cipher},
};

pub(crate) const CURVE_NAME: Nid = Nid::SECP384R1;
pub(crate) const SHARED_KEY_SIZE: usize = 48;
pub(crate) const AES_KEY_SIZE: usize = 32;
pub(crate) const AES_NOUNCE_SIZE: usize = 12;
pub(crate) const HKDF_SALT_SIZE: usize = 16;
pub(crate) const AES_TAG_SIZE: usize = 16;
pub(crate) const AES_AAD: &[u8] = b"encrypt data with rafs";
pub(crate) const QUOTE_FINGERPRINT_SIZE: usize = 32;
const LOCK_FILE_EXT: &str = ".lock";

/// Proceed an HKDF using SHA256 on a key and using a given salt
pub fn hkdf(key: &[u8], salt: &[u8; HKDF_SALT_SIZE]) -> Result<Vec<u8>> {
    let mut k = [0; AES_KEY_SIZE];
    let mut pkey = PkeyCtx::new_id(Id::HKDF)?;

    pkey.derive_init()?;
    pkey.add_hkdf_info(b"rafs-encrypt")?;
    pkey.set_hkdf_salt(salt)?;
    pkey.set_hkdf_md(Md::sha256())?;
    pkey.set_hkdf_key(key)?;
    pkey.derive(Some(&mut k))?;

    Ok(k.to_vec())
}

/// Compute the sha256 digest on bytes
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut pubkey_hash = Sha256::new();
    pubkey_hash.update(data);
    pubkey_hash.finish().to_vec()
}

/// Derive a shared key using ECDH
pub fn derive_shared_key(
    private_key: EcKey<Private>,
    peer_public_key: EcKey<Public>,
) -> Result<[u8; SHARED_KEY_SIZE]> {
    let mut pkey = PkeyCtx::new(PKey::from_ec_key(private_key)?.as_ref())?;
    pkey.derive_init()?;
    pkey.derive_set_peer(PKey::from_ec_key(peer_public_key)?.as_ref())?;
    let mut shared_key = [0u8; SHARED_KEY_SIZE];
    pkey.derive(Some(&mut shared_key))?;

    Ok(shared_key)
}

/// Get a unique filename based on current time
pub fn unique_filename() -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(now.to_string())
}

/// Decrypt bytes
pub fn decrypt(content: &[u8], private_key: EcKey<Private>) -> Result<Vec<u8>> {
    let group = EcGroup::from_curve_name(CURVE_NAME)?;

    // Deserialize the bytes
    let salt: [u8; HKDF_SALT_SIZE] = content[0..HKDF_SALT_SIZE].try_into()?;
    let mut offset = HKDF_SALT_SIZE;
    let peer_public_key = &content[offset..offset + SHARED_KEY_SIZE + 1];
    offset += SHARED_KEY_SIZE + 1;
    let nounce: [u8; AES_NOUNCE_SIZE] = content[offset..offset + AES_NOUNCE_SIZE].try_into()?;
    offset += AES_NOUNCE_SIZE;
    let tag: [u8; AES_TAG_SIZE] = content[offset..offset + AES_TAG_SIZE].try_into()?;
    offset += AES_TAG_SIZE;
    let encrypted_content = &content[offset..];

    // Recompute the share key
    let mut ctx = BigNumContext::new()?;
    let peer_public_key = EcKey::from_public_key(
        &group,
        EcPoint::from_bytes(&group, peer_public_key, &mut ctx)?.as_ref(),
    )?;

    let shared_key = derive_shared_key(private_key.clone(), peer_public_key)?;

    // Decrypt the payload
    let aes_key = hkdf(&shared_key, &salt)?;
    let cipher = Cipher::aes_256_gcm();
    let plain_content = decrypt_aead(
        cipher,
        &aes_key,
        Some(&nounce),
        AES_AAD,
        encrypted_content,
        &tag,
    )?;

    Ok(plain_content)
}

/// Enncrypt bytes
pub fn encrypt(
    content: &[u8],
    shared_key: [u8; SHARED_KEY_SIZE],
    public_key: &[u8],
) -> Result<Vec<u8>> {
    let mut salt = [0u8; HKDF_SALT_SIZE];
    let mut tag = [0u8; AES_TAG_SIZE];
    rand_bytes(&mut salt)?;
    let mut nounce = [0; AES_NOUNCE_SIZE];
    rand_bytes(&mut nounce)?;

    let aes_key = hkdf(&shared_key, &salt)?;
    let cipher = Cipher::aes_256_gcm();
    let encrypted_content =
        encrypt_aead(cipher, &aes_key, Some(&nounce), AES_AAD, content, &mut tag)?;

    let mut output_bytes = Vec::<u8>::new();
    output_bytes.extend_from_slice(&salt);
    output_bytes.extend_from_slice(public_key);
    output_bytes.extend_from_slice(&nounce);
    output_bytes.extend_from_slice(&tag);
    output_bytes.extend_from_slice(&encrypted_content);

    Ok(output_bytes)
}

/// Check if the file can be read (not read by someone else in the same time)
pub fn is_thread_safe_readable(path: &Path) -> bool {
    path.exists() && path.is_file() && !path.starts_with(".") && !path.ends_with(LOCK_FILE_EXT)
}

/// Read a file but make sure no one read it in the same time
pub fn thread_safe_read(path: &Path) -> Result<Vec<u8>> {
    let mut work_path = path.to_path_buf();
    work_path.set_extension(".lock");
    // Rename the current file (to not being picked by the watch)
    fs::rename(path, &work_path)?;
    // Read the file
    let content = fs::read(&work_path)?;
    // Remove the file
    fs::remove_file(work_path)?;
    Ok(content)
}

/// Write into a file and wait for the writting is complete before giving it the final name
pub fn thread_safe_write(data: &[u8], output: &Path, filename: &str) -> Result<()> {
    let tmp_output = output.join(format!(".{filename}"));
    // Write in a hidden file
    fs::write(&tmp_output, data)?;
    // Give the file the proper name now the file is complete
    fs::rename(&tmp_output, output.join(filename))?;
    Ok(())
}
