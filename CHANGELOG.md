# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2023-01-23

### Update

- Use tee-tools 1.2
- Use acme lib from original repository instead of Cosmian fork
- Improve snapshotting performance (25% CPU time gain)

### New 

- Support TDX build with packer on GCP
- Log received requests
- Can't process a snapshot when another is currently processing
- Endpoint `/snapshot` is no more blocking (a task is spawned)

### Fix

- Dockerker sgx

## [0.3.0] - 2023-01-19

### New

- Add `--application` in the `verify` subcommand
- Set CLI 'min-version' into the user agent

### Fix

- Support TPM in Amazon Linux image

## [0.2.0] - 2024-01-16

### New

- Support TPM aggregate using sha256, sha384, sha512
- `cosmian_certtool` has been added to the generated images

## [0.1.0] - 2024-01-13

### New

- First release supporting TDX, SEV and SGX on Ubuntu 22.04 & RHEL9
