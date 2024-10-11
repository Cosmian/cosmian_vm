# Cosmian Base Image Changelog

## [0.1.10] - 2024-10-10

- *GCP*: ([#158](https://github.com/Cosmian/cosmian_vm/pull/158))
  - *Ubuntu SEV/TDX capable*: update from `ubuntu-2204-tdx-v20240220` and `ubuntu-2404-noble-amd64-v20240830` to `ubuntu-2404-noble-amd64-v20241004`
  - *RedHat SEV/TDX capable*: update from `rhel-9-v20240815` to `rhel-9-v20241009`
- *Azure*: build images on RHEL + TDX ([#158](https://github.com/Cosmian/cosmian_vm/pull/158))

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
