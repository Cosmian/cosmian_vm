# Changelog

All notable changes to this project will be documented in this file.

## [1.3.17] - 2025-10-28

### 🐛 Bug Fixes

- Upgrade GCP RHEL base image (CVE-2025-5914) and GCP Ubuntu base image (CVE-2025-49844) 

### 🚀 Features

- Bump KMS version to 5.11.0

## [1.3.16] - 2025-10-09

### 🚀 Features

- Bump KMS version to 5.9.0

## [1.3.15] - 2025-10-07

### 🐛 Bug Fixes

- Fix Azure Cosmian AI validation error ([#205](https://github.com/Cosmian/cosmian_vm/pull/205))

## [1.3.14] - 2025-09-18

### 🐛 Bug Fixes

- Fix redis vulnerability (CVE-2024-46981) on RHEL GCP images ([#203](https://github.com/Cosmian/cosmian_vm/pull/203))

## [1.3.13] - 2025-09-05

### 🐛 Bug Fixes

- Upgrade GCP RHEL base image

## [1.3.12] - 2025-09-03

### 🐛 Bug Fixes

- Fix sqlite vulnerability (CVE-2025-6965) on RHEL GCP images ([#195](https://github.com/Cosmian/cosmian_vm/pull/195))

## [1.3.11] - 2025-08-28

### 🚀 Features

- Bump KMS version to 5.7.1

## [1.3.10] - 2025-08-14

### 🚀 Features

- Bump AI runner version to 1.0.1

## [1.3.9] - 2025-08-13

### 🐛 Bug Fixes

- Upgrade GCP base image to 0.1.13 ([#185](https://github.com/Cosmian/cosmian_vm/pull/185))

## [1.3.8] - 2025-08-08

### 🚀 Features

- Bump KMS version to 5.6.2

## [1.3.7] - 2025-06-30

### 🐛 Bug Fixes

- Upgrade GCP and AWS base image to 0.1.12 ([#179](https://github.com/Cosmian/cosmian_vm/pull/179))

## [1.3.6] - 2025-05-09

### 🚀 Features

- Upgrade Cosmian KMS from v4.24.0 to v5.0.0 ([#178](https://github.com/Cosmian/cosmian_vm/pull/178))

### ⚙️ Miscellaneous Tasks

- Make GH workflows xxx_image.yml callable individually
- Fix use packer SSH key for EC2 AWS instance ([#177](https://github.com/Cosmian/cosmian_vm/pull/177))

## [1.3.5] - 2025-04-28

### 🚀 Features

- Upgrade Cosmian AI runner v1.0.0 ([#174](https://github.com/Cosmian/cosmian_vm/pull/174))
- Upgrade Cosmian KMS v4.24.0 ([#174](https://github.com/Cosmian/cosmian_vm/pull/174))
- Snapshot integrity verification issue on AI runner:
  - Fetch IMA again when PCRs hash digest does not match the one in TPM quote

### 🐛 Bug Fixes

- Fix attestation verification on AWS with AMD SEV-SNP via `tee-tools` ([#44](https://github.com/Cosmian/tee-tools/pull/44))
- AI Runner fixes
  - Only check health endpoint for AI runner
  - Increase timeout for AI runner HTTPS test connection
  - RHEL:
    - Upgrade sqlite3>=3.35.0 for `chromadb` requirement
    - Make Python 3.12 default
    - Use absolute path for python3.12
- Update tokio and openssl due to RUSTSEC-2025-0023 and RUSTSEC-2025-0022

### 📚 Documentation

- Add playbook example

### 🧪 Testing

- Run app wo config

### ⚙️ Miscellaneous Tasks

- Bump KMS to 4.24
  - Update KMS configuration path du to new KMS packaging

## [1.3.4] - 2025-03-20

### 🐛 Bug Fixes

- Fix failure in systemd unit `mount_luks.service`  when using restart on failure

### ⚙️ Miscellaneous Tasks

- Bump `tee-tools` to [1.5.0](https://github.com/Cosmian/tee-tools/tree/1.5.0)
- Bump Rust crates of `cosmian_vm` as up-to-date as possible

## [1.3.3] - 2025-01-27

### 🚀 Features

- Upgrade the Cosmian base image v0.1.11 to upgrade Azure Ubuntu 22.04 to 24.04 ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
- Regenerate all images with the new base image v0.1.11 ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
- Bump KMS version from 4.21.1 to 4.21.2
- *AWS RHEL*: Bump version to 9.4 (RHEL-9.4.0_HVM-20241210-x86_64-0-Hourly2-GP3)

### 🐛 Bug Fixes

- Revert changes on AI runner systemd service file ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
- *GCP RHEL*: Do not upgrade all RHEL packages - just refresh cache ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
- For releases, clean Github cache before anything

## [1.3.2] - 2025-01-18

### 🚀 Features

- Bump KMS from 4.19.3 to 4.21.1 ([#167](https://github.com/Cosmian/cosmian_vm/pull/167))

### 🐛 Bug Fixes

- List of bug fixes in ([#167](https://github.com/Cosmian/cosmian_vm/pull/167)):
  - About KMS systemd service:
    - service must wait for mount_luks service but using `Requires` argument
    - Also `StandardOutput` to `syslog+console` to display `stdout`
  - Make `9998` the default local `Nginx` port for KMS. No extra conf to do anymore on `cosmian` CLI side
  - Fix CVE of `idna` crate by upgrading it from `0.5.0` to `1.0.3`.

### 🧪 Testing

- Test in Ansible if KMS service is up:
  - after first boot, first reboot and after a `cosmian_vm app init` configuration
- Display TPM PCR-7 before and after first reboot

### ⚙️ Miscellaneous Tasks

- Add `dev-container` files for VSCode

## [1.3.1] - 2024-10-30

### 🚀 Features

- Add TDX GCP license ([#164](https://github.com/Cosmian/cosmian_vm/pull/164))
- Bump KMS version to 4.19.3 ([#165](https://github.com/Cosmian/cosmian_vm/pull/165))

## [1.3.0] - 2024-10-18

### 🚀 Features

- *RHEL*: Add `cosmiand` SELinux module on RHEL to protect scripts and configuration through IMA measurements ([#151](https://github.com/Cosmian/cosmian_vm/pull/151))
  - Bump Base Image to 0.1.10
  - Add SELinux documentation on [#96](https://github.com/Cosmian/public_documentation/pull/96)
- RHEL TDX on GCP ([#158](https://github.com/Cosmian/cosmian_vm/pull/158))
  - Note: Ubuntu and RedHat *GCP* images upgraded -> using now Cosmian Base Image version 0.1.10 for all images
- Bump KMS version to 4.19.1 ([#160](https://github.com/Cosmian/cosmian_vm/pull/160))

### 🧪 CI

- Make products testable individually in Github CI ([#159](https://github.com/Cosmian/cosmian_vm/pull/159))
- Simplify versions bump ([#157](https://github.com/Cosmian/cosmian_vm/pull/157))
- Remove symbolic links from `libtdx_attest.so` ([#156](https://github.com/Cosmian/cosmian_vm/pull/156))

## [1.2.9] - 2024-10-09

### 🚀 Features

- *RHEL*:
  - Build AI Runner images also on RHEL ([#155](https://github.com/Cosmian/cosmian_vm/pull/155))

### 🧪 CI

- Bump Cosmian Base image to 0.1.9:
  - *Azure: RHEL*: update `9_3_cvm_sev_snp` à `9_4_cvm` ([#155](https://github.com/Cosmian/cosmian_vm/pull/155))
- Make CI non-blocking a tags even if tests fail ([#155](https://github.com/Cosmian/cosmian_vm/pull/155))
- Display kernel version ([#155](htts://github.com/Cosmian/cosmian_vm/pull/155))
- Bump KMS version to 4.19.0

### ⚙️ Miscellaneous Tasks

- *AI Runner*: Change installation folder from `/src/` to `/opt/` where SELinux label are `usr_t`

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
