# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0-rc.4] - 2024-04-05

### Bug Fixes

- Deployment on Azure via ansible ([#78](https://github.com/Cosmian/cosmian_vm/pull/78))
- App init trouble + add KMS playbook ([#83](https://github.com/Cosmian/cosmian_vm/pull/83))

## [1.1.0-rc.3] - 2024-03-28

### Bug Fixes

- Support for RHEL 9 on AMD SEV-SNP and Ubuntu 22.04 on Intel TDX is temporarily suspended because of some issues with `systemd-cryptenroll` when the instance reboot
- Create application storage folder if it does not exist
- Removed PCR-7 from systemd-cryptenroll for now because of failure at reboot (see <https://github.com/systemd/systemd/issues/24906>)
- `/var/tmp` is now a `tmpfs` filesystem to allow `dracut` temp files

### Features

- Base images for GCP have been updated: `ubuntu-2204-jammy-v20240319` and `rhel-9-v20240312`

## [1.1.0-rc.2] - 2024-03-14

### Bug Fixes

- Save LUKS password inside itself and write it even if the file does not exist
- Update rhel license

### Features

- Add Azure SEV quote (bump tee-tools dependency to 1.3.1)
- Add more context when cert and key files are not found (#70)
- Cloud provider detection to avoid verifying REPORT_DATA
- Adapt ansible script for ubuntu image on azure

### Miscellaneous Tasks

- Add business license for RHEL/Ubuntu Cosmian VM images
- Disable cargo-audit: du to mbox 0.6.0 yanked

## [1.1.0-rc.1] - 2024-03-14

### Bug Fixes

- Fix: try to use tmpfs for startup scripts
- Fix: remove PCR-8 to decrypt LUKS container
- Fix: hardcode tpm2 device with systemd-cryptenroll for RHEL 9

### Ci

- New workflow for GH Actions
  - Testing reboot of Cosmian VM instance (temporary continue-on-error when testing image)
  - Add instance_configs.cfg file for GCP guest-agent
  - Retrieve IP addr with gcloud CLI
  - Change GCP project and use gcloud beta
  - Auto-release image on GCP public project (#67)
  - Don't start and autostart supervisord but enable service
- Remove auto GH release on tag in order to add release candidates tags

### Miscellaneous Tasks

- Bump version of all crates to 1.1.0
- Check agent on SEV/TDX runners (#49)

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
