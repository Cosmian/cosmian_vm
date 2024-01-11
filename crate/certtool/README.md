# Cosmian CertTool

Tool to generate certificates. It could be a CA signed certificate generated using ACME protocol or a RATLS certificate. 

It's a light-weight way to generate certificates which works on SGX. 

Concerning RATLS certificate, the tool also allow the user to verify the certificate.

## Compile

```console
$ cargo build
```

## Usage

### ACME 

```console 
$ sudo cosmian_certtool acme --domain sgx.cosmian.dev --email tech@cosmian.com --workspace work_dir --output cert
$ ls cert
11425030654049150057_crt_sgx_cosmian_dev.crt  11425030654049150057_key_acme_account.key  11425030654049150057_key_sgx_cosmian_dev.key cert.pem key.pem
```

###  RATLS

```console
# Require an SGX enclave/SEV VM to run:
$ ratls_certtool generate --help

# On any hosts:
$ ratls_certtool verify --help
```