# Changelog

All notable changes to this project will be documented in this file.

## [1.3.0] - 2024-XX-XX

### 🚀 Features

- *RHEL*:
  - Use custom SELinux module to enforce IMA on any files Add KMS FIPS image. ([#151](https://github.com/Cosmian/cosmian_vm/pull/151))
  - Build VM/KMS/AI Runner images also for TDX on RHEL ([#155](https://github.com/Cosmian/cosmian_vm/pull/155))

### 🧪 CI

- Make CI non-blocking a tags even if tests fail ([#155](https://github.com/Cosmian/cosmian_vm/pull/155))
- Display kernel version ([#155](htts://github.com/Cosmian/cosmian_vm/pull/155))

### ⚙️ Miscellaneous Tasks

- *Azure: RHEL*: Replace old image `9_3_cvm_sev_snp` by `9_4_cvm` ([#155](https://github.com/Cosmian/cosmian_vm/pull/155))

## [1.2.8] - 2024-09-30

### 🧪 CI

- Du to Azure certification process, use the last RedHat kernel on Redhat images ([#154](https://github.com/Cosmian/cosmian_vm/pull/154))
  - Cosmian VM, KMS and AI runner images are now based on Cosmian Base Image version 0.1.8

### ⚙️ Miscellaneous Tasks

- Bump libtdx_attest.so from 1.21.100.3 to 1.22.100.3 ([#154](https://github.com/Cosmian/cosmian_vm/pull/154))

## [1.2.7] - 2024-09-12

### 🐛 Bug Fixes

- Cleanup old RedHat kernels for Azure certification process ([#150](https://github.com/Cosmian/cosmian_vm/pull/150))

### 🧪 CI

- Bump KMS version to 4.18.0
- Small fix about tags detection in Bash ([#152](https://github.com/Cosmian/cosmian_vm/pull/152))

## [1.2.6] - 2024-09-06

### 🚀 Features

- Make KMS/AI images usable even if KMS/AI softs have not been configured ([#148](https://github.com/Cosmian/cosmian_vm/pull/148))

### 🐛 Bug Fixes

- Du to [CVE-2024-6387](https://ubuntu.com/security/CVE-2024-6387), upgrade GCP official images to last versions ([#149](https://github.com/Cosmian/cosmian_vm/pull/149)) and use Cosmian base image 0.1.6:
  - ubuntu-2404-noble-amd64-v20240523a -> ubuntu-2404-noble-amd64-v20240830
  - rhel-9-v20240515 -> rhel-9-v20240815

### 🧪 CI

- Wait for agent to be ready before verifying snapshot ([#144](https://github.com/Cosmian/cosmian_vm/pull/144))
- *AWS*: Make aws-packer-build.sh standalone ([#145](https://github.com/Cosmian/cosmian_vm/pull/145))

### ⚙️ Miscellaneous Tasks

- Copy AWS images from eu to us for marketplace ([#146](https://github.com/Cosmian/cosmian_vm/pull/146))
- Update crates versions for security reasons verified via `cargo deny` ([#147](https://github.com/Cosmian/cosmian_vm/pull/147))

## [1.2.5] - 2024-07-08

### 🚀 Features

- *(Azure)* Add KMS FIPS image. ([#142](https://github.com/Cosmian/cosmian_vm/pull/142))
- Bump KMS version to 4.17.0 ([#142](https://github.com/Cosmian/cosmian_vm/pull/142))

### 🐛 Bug Fixes

- Fix uninstall of DEB/RPM packages ([#142](https://github.com/Cosmian/cosmian_vm/pull/142))

### ⚙️ Miscellaneous Tasks

- *(Azure)* Clean OS disk if exist on packer build

## [1.2.4] - 2024-06-28

### 🐛 Bug Fixes

- Timing variability in curve25519-dalek's Scalar29::sub/Scalar52::sub ([#141](https://github.com/Cosmian/cosmian_vm/pull/141))

### 📚 Documentation

- Move cloud providers images info in main README.md
- Create dedicated CHANGELOG for Cosmian base image

### ⚙️ Miscellaneous Tasks

- Create Debian and RPM packages for Ubuntu 22.04/24.04 and RedHat 9 ([#112](https://github.com/Cosmian/cosmian_vm/pull/112))
- Add bash script for VM image definition creation

## [1.2.3] - 2024-06-15

### 🐛 Bug Fixes

- Upgrade base image to 0.1.5: clean all authorized_keys and users ([#140](https://github.com/Cosmian/cosmian_vm/pull/140))

## [1.2.2] - 2024-06-13

### 🐛 Bug Fixes

- Update RHEL image by forcing installation of security update of shim-x64 package - if exists ([#137](https://github.com/Cosmian/cosmian_vm/pull/137))

## [1.2.1] - 2024-06-11

### 🚀 Features

- Add support for Cosmian AI Runner images ([#117](https://github.com/Cosmian/cosmian_vm/pull/117))
- Create frozen base image for Ubuntu/RHEL for GCP/Azure/AWS ([#120](https://github.com/Cosmian/cosmian_vm/pull/120))
- Modify GRUB for Azure security check: add `console=ttyS0 earlyprintk=ttyS0` to GRUB_CMDLINE_LINUX ([#132](https://github.com/Cosmian/cosmian_vm/pull/132))

### 🐛 Bug Fixes

- On KMS and AI Runner, remove unnecessarily opened ports ([#124](https://github.com/Cosmian/cosmian_vm/pull/124))
- Freeze packer plugins versions ([#127](https://github.com/Cosmian/cosmian_vm/pull/127))
- Use tpm2-abrmd as cosmian_vm_agent.service dependency to fix PCR Hash digest error ([#129](https://github.com/Cosmian/cosmian_vm/pull/129))
- Create VHD from OS disk to publish to marketplace ([#130](https://github.com/Cosmian/cosmian_vm/pull/130))
- AWS spawning retry ([#131](https://github.com/Cosmian/cosmian_vm/pull/131))

### Testing

- Merge Ansible roles for checking KMS or AI Runner ([#122](https://github.com/Cosmian/cosmian_vm/pull/122))

## [1.2.0] - 2024-05-23

### 🚀 Features

- Support Intel TDX on GCP and Azure ([#102](https://github.com/Cosmian/cosmian_vm/pull/102))
- Support Ubuntu/RHEL image on AWS

### 🐛 Bug Fixes

- Handle error in Ansible command
- Fix rust test `test_ratls_get_server_certificate`

### 📚 Documentation

- Sync with public doc

### ⚙️ Miscellaneous Tasks

- Bump KMS version to 4.16.0

### Ci

- Add cargo deny in CI ([#106](https://github.com/Cosmian/cosmian_vm/pull/106))
- Systematically clean cloud provider resources before and after ([#111](https://github.com/Cosmian/cosmian_vm/pull/111))
- Run concurrency build by cloud provider ([#113](https://github.com/Cosmian/cosmian_vm/pull/113))

## [1.1.2] - 2024-05-06

### 🚀 Features

- Move to systemd service for Cosmian VM and Cosmian KMS ([#100](https://github.com/Cosmian/cosmian_vm/pull/100))

### 🐛 Bug Fixes

- Add/remove privilege escalation on local tasks ([#97](https://github.com/Cosmian/cosmian_vm/pull/97))
- Create GCP firewall rule on test instances ([#101](https://github.com/Cosmian/cosmian_vm/pull/101))
- Fix RUSTSEC-2024-0336 ([#103](https://github.com/Cosmian/cosmian_vm/pull/103))
- Fetch TPM quote just after IMA event log to prevent side effects ([#104](https://github.com/Cosmian/cosmian_vm/pull/104))

### ⚙️ Miscellaneous Tasks

- Run KMS playbook on a raw VM ([#104](https://github.com/Cosmian/cosmian_vm/pull/104))

### Refactor

- Reuse cargo workspace version in all subcrates ([#106](https://github.com/Cosmian/cosmian_vm/pull/106))

## [1.1.1] - 2024-04-16

### 🐛 Bug Fixes

- [Ansible] Automate reboot right after dracut IMA-relative ([#95](https://github.com/Cosmian/cosmian_vm/pull/95))
- [Rust] Generate TPM keys before generate encrypted FS ([#95](https://github.com/Cosmian/cosmian_vm/pull/95))

## [1.1.0] - 2024-04-12

### 🚀 Features

- For GCP (SEV) ([#94](https://github.com/Cosmian/cosmian_vm/pull/94)):
  - Deploy Cosmian VM/KMS images based on `ubuntu-2204-jammy-v20240319` and `rhel-9-v20240312`. Images deployment on tags only.
  - Remove use of startup scripts:
    - cosmian_vm_agent is auto-restarting on failures
    - for KMS, nginx is auto-restarting on failures
- For Azure (SEV):
  - Add Ansible Cosmian VM/KMS installation

### 🐛 Bug Fixes

- Fix reboot problem on RHEL ([#84](https://github.com/Cosmian/cosmian_vm/pull/84))

## [1.1.0-rc.4] - 2024-04-05

### 🐛 Bug Fixes

- Deployment on Azure via ansible ([#78](https://github.com/Cosmian/cosmian_vm/pull/78))
- App init trouble + add KMS playbook ([#83](https://github.com/Cosmian/cosmian_vm/pull/83))

## [1.1.0-rc.3] - 2024-03-28

### 🐛 Bug Fixes

- Support for RHEL 9 on AMD SEV-SNP and Ubuntu 22.04 on Intel TDX is temporarily suspended because of some issues with `systemd-cryptenroll` when the instance reboot
- Create application storage folder if it does not exist
- Removed PCR-7 from systemd-cryptenroll for now because of failure at reboot (see <https://github.com/systemd/systemd/issues/24906>)
- `/var/tmp` is now a `tmpfs` filesystem to allow `dracut` temp files

### 🚀 Features

- Base images for GCP have been updated: `ubuntu-2204-jammy-v20240319` and `rhel-9-v20240312`

## [1.1.0-rc.2] - 2024-03-14

### 🐛 Bug Fixes

- Save LUKS password inside itself and write it even if the file does not exist
- Update rhel license

### 🚀 Features

- Add Azure SEV quote (bump tee-tools dependency to 1.3.1)
- Add more context when cert and key files are not found (#70)
- Cloud provider detection to avoid verifying REPORT_DATA
- Adapt ansible script for ubuntu image on azure

### ⚙️ Miscellaneous Tasks

- Add business license for RHEL/Ubuntu Cosmian VM images
- Disable cargo-audit: du to mbox 0.6.0 yanked

## [1.1.0-rc.1] - 2024-03-14

### 🐛 Bug Fixes

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

### ⚙️ Miscellaneous Tasks

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
