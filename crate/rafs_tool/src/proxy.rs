use anyhow::Result;
use clap::Args;
use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::pkey::{Private, Public};
use sev_quote::quote::get_quote;
use std::thread::{self, JoinHandle};

use std::path::{Path, PathBuf};
use std::{fs, time};

use crate::common::{
    decrypt, derive_shared_key, encrypt, is_thread_safe_readable, sha256, thread_safe_read,
    thread_safe_write, unique_filename, CURVE_NAME,
};

/// Proxify files en-/de-cryption
#[derive(Args, Debug)]
pub struct ProxyArgs {
    /// Path of the client public key (SECP384R1)
    #[arg(long, required = true)]
    client_public_key: PathBuf,

    /// Path containing shared secrets between the proxy and the clients
    #[arg(long, default_value = PathBuf::from("shared").into_os_string())]
    shared_secret: PathBuf,

    /// Path to watch for files to decrypt
    #[arg(long, default_value = PathBuf::from("input_enc").into_os_string())]
    encrypted_input: PathBuf,

    /// Path to write decrypted files
    #[arg(long, default_value = PathBuf::from("input_plain").into_os_string())]
    decrypted_input: PathBuf,

    /// Path to watch for files to encrypt
    #[arg(long, default_value = PathBuf::from("output_plain").into_os_string())]
    decrypted_output: PathBuf,

    /// Path to write encrypted files
    #[arg(long, default_value = PathBuf::from("output_enc").into_os_string())]
    encrypted_output: PathBuf,
}

impl ProxyArgs {
    pub fn run(&self) -> Result<()> {
        // // Step 1: Generate an enclave/VM key
        // let key = get_key(true)?;

        // // Step 2: Create the DH bi-keys from the key
        // let group = EcGroup::from_curve_name(Nid::SECP384R1)?;
        // let private_number = BigNum::from_slice(&key)?;
        // let public_point = EcPoint::new(&group)?;
        // let private_key =
        //     EcKey::from::from_private_components(&group, &private_number, &public_point)?;
        // private_key.check_key()?;

        // Step 1: Generate the bi-key for the ECDH
        let group = EcGroup::from_curve_name(CURVE_NAME)?;
        let private_key = EcKey::generate(&group)?;

        // Step 2: Generate the quote
        let mut inner_user_report_data = [0u8; sev_quote::REPORT_DATA_SIZE];
        let report_data = sha256(&private_key.public_key_to_der()?);
        inner_user_report_data[0..report_data.len()].copy_from_slice(&report_data);
        let quote = get_quote(&inner_user_report_data)?;

        // Step 3: Store the quote and the key
        let proxy_public_key_path = self.shared_secret.join(PathBuf::from("key.pub"));
        let quote_path = self.shared_secret.join(PathBuf::from("quote"));

        fs::write(&quote_path, bincode::serialize(&quote)?)?;
        fs::write(&proxy_public_key_path, private_key.public_key_to_pem()?)?;
        let client_public_key = EcKey::public_key_from_pem(&fs::read(&self.client_public_key)?)?;

        println!("Quote: {quote_path:?}");
        println!("Public key: {proxy_public_key_path:?}");

        fs::create_dir_all(&self.encrypted_input)?;
        fs::create_dir_all(&self.decrypted_input)?;
        fs::create_dir_all(&self.encrypted_output)?;
        fs::create_dir_all(&self.decrypted_output)?;

        // Step 4: start the encryption/decryption
        let output_handler = watch_directory_and_decrypt(
            private_key.clone(),
            self.encrypted_input.clone(),
            self.decrypted_input.clone(),
        );

        let input_handler = watch_directory_and_encrypt(
            private_key.clone(),
            client_public_key.clone(),
            self.decrypted_output.clone(),
            self.encrypted_output.clone(),
        );

        input_handler.join().expect("Can't stop input watcher");
        output_handler.join().expect("Can't stop output watcher");

        Ok(())
    }
}

fn watch_directory_and_decrypt(
    private_key: EcKey<Private>,
    input: PathBuf,
    output: PathBuf,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Output watcher".to_string())
        .spawn(move || loop {
            if let Err(e) = decrypt_directory(&private_key, &input, &output) {
                println!("An error occured: {e:?}");
            }

            thread::sleep(time::Duration::from_secs(1));
        })
        .expect("Can't start the decrypt watcher thread")
}

fn decrypt_directory(private_key: &EcKey<Private>, input: &Path, output: &Path) -> Result<()> {
    let paths = fs::read_dir(input)?;

    for path in paths {
        let path = path?.path();
        let filename = path
            .file_name()
            .map_or_else(unique_filename, |f| Ok(f.to_string_lossy().to_string()))?;

        if !is_thread_safe_readable(&path) {
            continue;
        }

        println!("Processing {path:?}...");

        // Read the file
        let content = thread_safe_read(&path)?;

        // Decrypt the file
        let plain_content = decrypt(&content, private_key.clone())?;

        // Save the plain text file
        thread_safe_write(&plain_content, output, &filename)?;
    }

    Ok(())
}

fn watch_directory_and_encrypt(
    proxy_private_key: EcKey<Private>,
    peer_public_key: EcKey<Public>,
    input: PathBuf,
    output: PathBuf,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Input watcher".to_string())
        .spawn(move || loop {
            if let Err(e) = encrypt_directory(&proxy_private_key, &peer_public_key, &input, &output)
            {
                println!("An error occured: {e:?}");
            }

            thread::sleep(time::Duration::from_secs(1));
        })
        .expect("Can't start the decrypt watcher thread")
}

fn encrypt_directory(
    proxy_private_key: &EcKey<Private>,
    peer_public_key: &EcKey<Public>,
    input: &Path,
    output: &Path,
) -> Result<()> {
    let group = EcGroup::from_curve_name(CURVE_NAME)?;

    let paths = fs::read_dir(input)?;

    for path in paths {
        let path = path?.path();

        if !is_thread_safe_readable(&path) {
            continue;
        }

        println!("Processing {path:?}...");

        // Read the file
        let content = thread_safe_read(&path)?;

        // Recompute the share key
        let shared_key = derive_shared_key(proxy_private_key.clone(), peer_public_key.clone())?;
        let mut ctx = BigNumContext::new()?;

        // Encrypt the file
        let output_bytes = encrypt(
            &content,
            shared_key,
            &proxy_private_key.public_key().to_bytes(
                &group,
                PointConversionForm::COMPRESSED,
                &mut ctx,
            )?,
        )?;

        // Save the encrypted file
        thread_safe_write(&output_bytes, output, &unique_filename()?)?;
    }

    Ok(())
}
