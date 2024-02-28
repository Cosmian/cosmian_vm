# Changelog

All notable changes to this project will be documented in this file.

## [1.0.1] - 2024-02-07

### Fix

- Do not start supervisor when building the image but only when instantiate the built image. Otherwise it creates a luks inside it which can't be decrypted when instantiating the VM on GCP.

## [1.0.0] - 2024-02-06

### Update

- Sending app configuration does not require a key (and file is no more encrypted)
- Restarting the app does not require a key
- Relative path in the configuration file is now allowed: related to `/var/lib/cosmian_vm`
- `cosmian_vm_agent` can be started on non-tee host without panicking

### New

- `cosmian_fstool` is released
- Create a LUKS container when starting the `cosmian_vm_agent` for the first time

## [0.4.0] - 2024-01-23

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

- Docker sgx

## [0.3.0] - 2024-01-19

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
