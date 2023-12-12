use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use acme_lib::{create_p384_key, persist::FilePersist, Account, Directory, DirectoryUrl};
use error::Error;
use openssl::{
    bn::{BigNum, BigNumContext},
    ec::{EcGroup, EcKey, EcPoint},
    nid::Nid,
    pkey::{PKey, Private},
};

pub mod error;

const CERT_FILE: &str = "cert.pem";
const KEY_FILE: &str = "key.pem";

fn new_account(email: &str, output: &Path, staging: bool) -> Result<Account<FilePersist>, Error> {
    let url = if staging {
        DirectoryUrl::LetsEncryptStaging
    } else {
        DirectoryUrl::LetsEncrypt
    };

    // Save/load keys and certificates to current dir
    let persist = FilePersist::new(output);

    // Create a directory entrypoint
    let dir = Directory::from_url(persist, url)?;

    // Create the account
    Ok(dir.account(email)?)
}

/// Generate a private key
///
/// If `use_tee_key` is None, the key is randomly generated
/// If `use_tee_key` is Some, the generated key depends on the TEE and a nonce
fn generate_private_key(use_tee_key: Option<&[u8]>) -> Result<PKey<Private>, Error> {
    if let Some(salt) = &use_tee_key {
        let key = tee_attestation::get_key(Some(salt))?;
        let private_number = BigNum::from_slice(&key)?;
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;

        let mut public_point = EcPoint::new(&group)?;
        let ctx = BigNumContext::new()?;
        public_point.mul_generator(&group, &private_number, &ctx)?;

        let pri_key_ec = EcKey::from_private_components(&group, &private_number, &public_point)?;
        Ok(PKey::from_ec_key(pri_key_ec)?)
    } else {
        Ok(create_p384_key())
    }
}

pub fn generate(
    domain: &str,
    san: &[&str],
    email: &str,
    workspace: &Path,
    output: &Path,
    use_tee_key: Option<&[u8]>,
    staging: bool,
) -> Result<(String, String), Error> {
    // Create an account
    let account = new_account(email, output, staging)?;

    // Generate a private key
    let pkey_pri = generate_private_key(use_tee_key)?;

    // Order a new TLS certificate for a domain
    let mut ord_new = account.new_order(domain, san)?;

    let target = Path::new(&workspace).join(".well-known/acme-challenge/");
    let target_parent = Path::new(&workspace).join(".well-known");

    // If the ownership of the domain(s) has already been
    // authorized in a previous order, you might be able to
    // skip validation. The ACME API provider decides.
    let ord_csr = loop {
        // Are we done?
        if let Some(ord_csr) = ord_new.confirm_validations() {
            break ord_csr;
        }

        // Get the possible authorizations (for a single domain
        // this will only be one element).
        let auths = ord_new.authorizations()?;

        // For HTTP, the challenge is a text file that needs to
        // be placed in your web server's root:
        //
        // /var/www/.well-known/acme-challenge/<token>
        //
        // The important thing is that it's accessible over the
        // web for the domain(s) you are trying to get a
        // certificate for:
        //
        // http://mydomain.io/.well-known/acme-challenge/<token>
        let chall = auths[0].http_challenge();

        // The token is the filename.
        let token = chall.http_token();

        // The proof is the contents of the file
        let proof = chall.http_proof();

        // Update my web server
        fs::create_dir_all(&target)?;
        fs::write(target.join(token), proof).expect("Unable to write the token file");

        // After the file is accessible from the web,
        // this tells the ACME API to start checking the
        // existence of the proof.
        //
        // The order at ACME will change status to either
        // confirm ownership of the domain, or fail due to the
        // not finding the proof. To see the change, we poll
        // the API with 5000 milliseconds wait between.
        chall.validate(5000)?;

        // Update the state against the ACME API.
        ord_new.refresh()?;

        // Clean the .well-known
        #[allow(clippy::needless_borrow)]
        fs::remove_dir_all(&target_parent)?;
    };

    // Submit the CSR. This causes the ACME provider to enter a
    // state of "processing" that must be polled until the
    // certificate is either issued or rejected. Again we poll
    // for the status change.
    let ord_cert = ord_csr.finalize_pkey(pkey_pri, 5000)?;

    // Now download the certificate. Also stores the cert in
    // the persistence.
    let certificate = ord_cert.download_and_save_cert()?;

    // We also save the certificate and the key in a more predictable filename
    // Note: no symlink because SGX does not support this syscall
    let mut file = File::create(output.join(KEY_FILE))?;
    file.write_all(certificate.private_key().as_bytes())?;

    let mut file = File::create(output.join(CERT_FILE))?;
    file.write_all(certificate.certificate().as_bytes())?;

    Ok((
        certificate.private_key().to_owned(),
        certificate.certificate().to_owned(),
    ))
}
