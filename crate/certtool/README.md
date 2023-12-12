# Cosmian VM cert-tool

Tool to generate certificates. It's a light-weight way to generate certificates which works on SGX. 

## Usage

```sh 
$ cargo build
$ sudo ./target/debug/cosmian_vm_certtool --domain sgx.cosmian.dev --email tech@cosmian.com --workspace work_dir --output cert
$ ls cert
11425030654049150057_crt_sgx_cosmian_dev.crt  11425030654049150057_key_acme_account.key  11425030654049150057_key_sgx_cosmian_dev.key cert.pem key.pem
```