# Cosmian Base Image Changelog

## [0.1.15] - 2025-10-27

- *GCP*:
  - *Ubuntu SEV/TDX capable*: update from `ubuntu-2404-noble-amd64-v20250805` to `ubuntu-2404-noble-amd64-v20251021`
  - *RedHat SEV/TDX capable*: update from `rhel-9-v20250709` to `rhel-9-v20251016`

## [0.1.14] - 2025-09-04

- *GCP*:
  - *RedHat SEV/TDX capable*: update from `rhel-9-v20250709` to `rhel-9-v20250812`

## [0.1.13] - 2025-08-13

- *GCP*: ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
  - *Ubuntu SEV/TDX capable*: update from `ubuntu-2404-noble-amd64-v20250606`to `ubuntu-2404-noble-amd64-v20250805`
  - *RedHat SEV/TDX capable*: update from `rhel-9-v20250611` to `rhel-9-v20250709`

## [0.1.12] - 2025-06-30

- *GCP*: ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
  - *Ubuntu SEV/TDX capable*: update from `ubuntu-2404-noble-amd64-v20241004`to `ubuntu-2404-noble-amd64-v20250606`
  - *RedHat SEV/TDX capable*: update from `rhel-9-v20241009` to `rhel-9-v20250611`
- *AWS*:
  - *Ubuntu*: update from `ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-20240523.1`to `ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-20250610`
  - *RedHat*: update from `RHEL-9.4.0_HVM-20241210-x86_64-0-Hourly2-GP3` to `RHEL-9.4.0_HVM-20250519-x86_64-0-Hourly2-GP3`

## [0.1.11] - 2025-01-27

- *Azure*: ([#168](https://github.com/Cosmian/cosmian_vm/pull/168))
  - Upgrade Ubuntu 22.04 to 24.04:

## [0.1.10] - 2024-10-10

- *GCP*: ([#158](https://github.com/Cosmian/cosmian_vm/pull/158))
  - *Ubuntu SEV/TDX capable*: update from `ubuntu-2204-tdx-v20240220` and `ubuntu-2404-noble-amd64-v20240830` to `ubuntu-2404-noble-amd64-v20241004`
  - *RedHat SEV/TDX capable*: update from `rhel-9-v20240815` to `rhel-9-v20241009`
- *RHEL*: Add `cosmiand` SELinux module on RHEL to protect scripts and configuration through IMA measurements ([#151](https://github.com/Cosmian/cosmian_vm/pull/151))

## [0.1.9] - 2024-10-09

- *Azure: RHEL*: update `9_3_cvm_sev_snp` to `9_4_cvm` ([#154](https://github.com/Cosmian/cosmian_vm/pull/154))

## [0.1.8] - 2024-09-30

- Du to Azure certification process, use the last RedHat kernel on Redhat images ([#154](https://github.com/Cosmian/cosmian_vm/pull/154))

## [0.1.7] - 2024-09-12

- Du to Azure certification process, remove all old linux kernels on Redhat images ([#152](https://github.com/Cosmian/cosmian_vm/pull/152))

## [0.1.6] - 2024-09-06

- Du to [CVE-2024-6387](https://ubuntu.com/security/CVE-2024-6387), upgrade GCP official images to last versions: ([#149](https://github.com/Cosmian/cosmian_vm/pull/149))
  - ubuntu-2404-noble-amd64-v20240523a -> ubuntu-2404-noble-amd64-v20240830
  - rhel-9-v20240515 -> rhel-9-v20240815

## [0.1.5] - 2024-06-15

- Clean RHEL/Ubuntu after builds: ([#140](https://github.com/Cosmian/cosmian_vm/pull/140))
  - clean all authorized_keys
  - clean users

## [0.1.4] - 2024-06-12

- RHEL:
  - Force security update for shim-x64 package ([#137](https://github.com/Cosmian/cosmian_vm/pull/137))

## [0.1.3] - 2024-06-11

- RHEL:
  - Fix grub2-mkconfig invalid output path
  - Add RHEL security updates

## [0.1.2] - 2024-06-05

- Modify GRUB for Azure security check: add `console=ttyS0 earlyprintk=ttyS0` to GRUB_CMDLINE_LINUX

## [0.1.1] - 2024-06-03

- Install TPM2 Access Broker & Resource Manager (tpm2-abrmd)
- Add dependency to tpm2-abrmd in cosmian_vm_agent.service

## [0.1.0] - 2024-05-28

- Modify GRUB for IMA support
- Add TPM Support
- Install Intel dependencies (libtdx-attest support)
- Upgrade distribution package
- Disable auto upgrade services
