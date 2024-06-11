# Image build

## Cloud providers base images

### Changelog

## [0.1.3] - 2024-06-05

- RHEL:
  * Fix grub2-mkconfig invalid output path
  * Add RHEL security updates

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

Replace `X.Y.Z` in the 3 following tables.

### AWS images

|               |                            Official image                            | OS image | OS version | Cosmian base image          |
| :------------ | :------------------------------------------------------------------: | :------: | ---------- | --------------------------- |
| AWS - AMD SEV | ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-20240523.1 |  Ubuntu  | 24.04      | base-image-X-Y-Z-ubuntu-sev |
| AWS - AMD SEV |            RHEL-9.3.0_HVM-20240117-x86_64-49-Hourly2-GP3             |  Redhat  | 9.3        | base-image-X-Y-Z-ubuntu-sev |

If needed:

```sh
aws ec2 describe-images --output json > aws_list.json
```

### Azure images

|                   |                        Official image                         | OS image | OS version      | Cosmian base image    | Version |
| :---------------- | :-----------------------------------------------------------: | :------: | --------------- | --------------------- | ------- |
| Azure - Intel TDX | Canonical-0001-com-ubuntu-confidential-vm-jammy-22_04-lts-cvm |  Ubuntu  | 22.04.202404090 | base-image-ubuntu-tdx | X.Y.Z   |
| Azure - AMD SEV   | Canonical-0001-com-ubuntu-confidential-vm-jammy-22_04-lts-cvm |  Ubuntu  | 22.04.202404090 | base-image-ubuntu-sev | X.Y.Z   |
| Azure - AMD SEV   |                Redhat-rhel-cvm-9_3_cvm_sev_snp                |  Redhat  | 9.3.2023111017  | base-image-rhel-sev   | X.Y.Z   |

```sh
az vm list> azure_list.json
```

### Redhat security updates on base image

Last metadata expiration check: 0:02:49 ago on Tue 11 Jun 2024 05:53:08 AM UTC.
RHSA-2023:5838 Important/Sec. libnghttp2-1.43.0-5.el9_2.1.x86_64
RHSA-2023:6745 Important/Sec. curl-7.76.1-26.el9_3.2.x86_64
RHSA-2023:6745 Important/Sec. libcurl-7.76.1-26.el9_3.2.x86_64
RHSA-2023:6746 Important/Sec. libnghttp2-1.43.0-5.el9_3.1.x86_64
RHSA-2023:7747 Moderate/Sec.  libxml2-2.9.13-5.el9_3.x86_64
RHSA-2023:7749 Important/Sec. kernel-modules-core-5.14.0-362.13.1.el9_3.x86_64
RHSA-2023:7749 Important/Sec. kernel-uki-virt-5.14.0-362.13.1.el9_3.x86_64
RHSA-2024:0108 Moderate/Sec.  nspr-4.35.0-4.el9_3.x86_64
RHSA-2024:0108 Moderate/Sec.  nss-3.90.0-4.el9_3.x86_64
RHSA-2024:0108 Moderate/Sec.  nss-softokn-3.90.0-4.el9_3.x86_64
RHSA-2024:0108 Moderate/Sec.  nss-softokn-freebl-3.90.0-4.el9_3.x86_64
RHSA-2024:0108 Moderate/Sec.  nss-sysinit-3.90.0-4.el9_3.x86_64
RHSA-2024:0108 Moderate/Sec.  nss-util-3.90.0-4.el9_3.x86_64
RHSA-2024:0310 Moderate/Sec.  openssl-1:3.0.7-25.el9_3.x86_64
RHSA-2024:0310 Moderate/Sec.  openssl-libs-1:3.0.7-25.el9_3.x86_64
RHSA-2024:0461 Important/Sec. kernel-modules-core-5.14.0-362.18.1.el9_3.x86_64
RHSA-2024:0461 Important/Sec. kernel-uki-virt-5.14.0-362.18.1.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  python3-rpm-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-build-libs-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-libs-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-plugin-audit-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-plugin-selinux-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-plugin-systemd-inhibit-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0463 Moderate/Sec.  rpm-sign-libs-4.16.1.3-27.el9_3.x86_64
RHSA-2024:0464 Moderate/Sec.  python3-urllib3-1.26.5-3.el9_3.1.noarch
RHSA-2024:0465 Moderate/Sec.  sqlite-libs-3.34.1-7.el9_3.x86_64
RHSA-2024:0466 Moderate/Sec.  python3-3.9.18-1.el9_3.1.x86_64
RHSA-2024:0466 Moderate/Sec.  python3-libs-3.9.18-1.el9_3.1.x86_64
RHSA-2024:0466 Moderate/Sec.  python-unversioned-command-3.9.18-1.el9_3.1.noarch
RHSA-2024:0533 Moderate/Sec.  gnutls-3.7.6-23.el9_3.3.x86_64
RHSA-2024:0790 Moderate/Sec.  nspr-4.35.0-6.el9_3.x86_64
RHSA-2024:0790 Moderate/Sec.  nss-3.90.0-6.el9_3.x86_64
RHSA-2024:0790 Moderate/Sec.  nss-softokn-3.90.0-6.el9_3.x86_64
RHSA-2024:0790 Moderate/Sec.  nss-softokn-freebl-3.90.0-6.el9_3.x86_64
RHSA-2024:0790 Moderate/Sec.  nss-sysinit-3.90.0-6.el9_3.x86_64
RHSA-2024:0790 Moderate/Sec.  nss-util-3.90.0-6.el9_3.x86_64
RHSA-2024:0811 Moderate/Sec.  sudo-1.9.5p2-10.el9_3.x86_64
RHSA-2024:1129 Moderate/Sec.  curl-7.76.1-26.el9_3.3.x86_64
RHSA-2024:1129 Moderate/Sec.  libcurl-7.76.1-26.el9_3.3.x86_64
RHSA-2024:1130 Moderate/Sec.  openssh-8.7p1-34.el9_3.3.x86_64
RHSA-2024:1130 Moderate/Sec.  openssh-clients-8.7p1-34.el9_3.3.x86_64
RHSA-2024:1130 Moderate/Sec.  openssh-server-8.7p1-34.el9_3.3.x86_64
RHSA-2024:1248 Important/Sec. kernel-modules-core-5.14.0-362.24.1.el9_3.x86_64
RHSA-2024:1248 Important/Sec. kernel-uki-virt-5.14.0-362.24.1.el9_3.x86_64
RHSA-2024:1530 Moderate/Sec.  expat-2.5.0-1.el9_3.1.x86_64
RHSA-2024:1692 Moderate/Sec.  less-590-3.el9_3.x86_64
RHSA-2024:1879 Moderate/Sec.  gnutls-3.7.6-23.el9_3.4.x86_64
RHSA-2024:1903 Important/Sec. shim-x64-15.8-4.el9_3.x86_64
RHSA-2024:2348 Moderate/Sec.  python3-jinja2-2.11.3-5.el9.noarch
RHSA-2024:2394 Important/Sec. kernel-modules-core-5.14.0-427.13.1.el9_4.x86_64
RHSA-2024:2394 Important/Sec. kernel-uki-virt-5.14.0-427.13.1.el9_4.x86_64
RHSA-2024:2396 Moderate/Sec.  squashfs-tools-4.4-10.git1.el9.x86_64
RHSA-2024:2438 Moderate/Sec.  pam-1.5.1-19.el9.x86_64
RHSA-2024:2447 Low/Sec.       openssl-1:3.0.7-27.el9.x86_64
RHSA-2024:2447 Low/Sec.       openssl-libs-1:3.0.7-27.el9.x86_64
RHSA-2024:2463 Moderate/Sec.  systemd-252-32.el9_4.x86_64
RHSA-2024:2463 Moderate/Sec.  systemd-libs-252-32.el9_4.x86_64
RHSA-2024:2463 Moderate/Sec.  systemd-pam-252-32.el9_4.x86_64
RHSA-2024:2463 Moderate/Sec.  systemd-resolved-252-32.el9_4.x86_64
RHSA-2024:2463 Moderate/Sec.  systemd-rpm-macros-252-32.el9_4.noarch
RHSA-2024:2463 Moderate/Sec.  systemd-udev-252-32.el9_4.x86_64
RHSA-2024:2504 Low/Sec.       libssh-0.10.4-13.el9.x86_64
RHSA-2024:2504 Low/Sec.       libssh-config-0.10.4-13.el9.noarch
RHSA-2024:2512 Low/Sec.       file-5.39-16.el9.x86_64
RHSA-2024:2512 Low/Sec.       file-libs-5.39-16.el9.x86_64
RHSA-2024:2512 Low/Sec.       python3-file-magic-5.39-16.el9.noarch
RHSA-2024:2570 Moderate/Sec.  gnutls-3.8.3-4.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  libsss_certmap-2.9.4-6.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  libsss_idmap-2.9.4-6.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  libsss_nss_idmap-2.9.4-6.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  libsss_sudo-2.9.4-6.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  sssd-client-2.9.4-6.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  sssd-common-2.9.4-6.el9_4.x86_64
RHSA-2024:2571 Moderate/Sec.  sssd-kcm-2.9.4-6.el9_4.x86_64
RHSA-2024:2679 Moderate/Sec.  libxml2-2.9.13-6.el9_4.x86_64
RHSA-2024:2758 Moderate/Sec.  kernel-modules-core-5.14.0-427.16.1.el9_4.x86_64
RHSA-2024:2758 Moderate/Sec.  kernel-uki-virt-5.14.0-427.16.1.el9_4.x86_64
RHSA-2024:3306 Moderate/Sec.  kernel-modules-core-5.14.0-427.18.1.el9_4.x86_64
RHSA-2024:3306 Moderate/Sec.  kernel-uki-virt-5.14.0-427.18.1.el9_4.x86_64
RHSA-2024:3339 Important/Sec. glibc-2.34-100.el9_4.2.x86_64
RHSA-2024:3339 Important/Sec. glibc-common-2.34-100.el9_4.2.x86_64
RHSA-2024:3339 Important/Sec. glibc-gconv-extra-2.34-100.el9_4.2.x86_64
RHSA-2024:3339 Important/Sec. glibc-langpack-en-2.34-100.el9_4.2.x86_64
RHSA-2024:3501 Moderate/Sec.  libnghttp2-1.43.0-5.el9_4.3.x86_64
RHSA-2024:3513 Important/Sec. less-590-4.el9_4.x86_64
RHSA-2024:3619 Moderate/Sec.  kernel-modules-core-5.14.0-427.20.1.el9_4.x86_64
RHSA-2024:3619 Moderate/Sec.  kernel-uki-virt-5.14.0-427.20.1.el9_4.x86_64

#### Update Unified Kernel Image: UKI

Links:

- <https://www.redhat.com/fr/blog/rhel-confidential-virtual-machines-azure-technical-deep-dive>
- <https://access.redhat.com/documentation/ml/red_hat_enterprise_linux/9/pdf/deploying_rhel_9_on_microsoft_azure/red_hat_enterprise_linux-9-deploying_rhel_9_on_microsoft_azure-en-us.pdf>

### GCP images

|                 |           Official image           | OS image | OS version | Cosmian base image          |
| :-------------- | :--------------------------------: | :------: | ---------- | --------------------------- |
| GCP - Intel TDX |     ubuntu-2204-tdx-v20240220      |  Ubuntu  | 22.04      | base-image-X-Y-Z-ubuntu-tdx |
| GCP - AMD SEV   | ubuntu-2404-noble-amd64-v20240523a |  Ubuntu  | 24.04      | base-image-X-Y-Z-ubuntu-sev |
| GCP - AMD SEV   |          rhel-9-v20240515          |  Redhat  | 9.3        | base-image-X-Y-Z-rhel-sev   |

```sh
gcloud compute images list > gcloud_list.json
gcloud beta compute images list --filter="guestOsFeatures[].type=SEV_SNP_CAPABLE" --format=json > gcloud_images_SEV_SNP_CAPABLE.json
gcloud beta compute images list --filter="guestOsFeatures[].type=TDX_CAPABLE" --format=json > gcloud_images_TDX.json
```

## Ansible script

This Ansible script is designed to work with packer.

You can run it anyway without packer as follow:

```console
# Be sure to first login using SSH through the AWS Console
# And then create the `cosmian` user
sudo useradd -s /bin/bash -d /home/cosmian -m -G sudo cosmian
sudo echo "cosmian ALL =(ALL) NOPASSWD:ALL" >> /etc/sudoers
# And then add your own `.ssh/id_rsa.pub` in the remote `.ssh/authorized_keys`
sudo su cosmian && cd
mkdir -p .ssh/
vi .ssh/authorized_keys


# Then on your localhost
export USERNAME=cosmian
export HOST=35.204.83.49
# Be sure to install deps: `pip install -r python_modules.txt` on your localhost
cd ansible
ansible-playbook cosmian-vm-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.1
ansible-playbook kms-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.1 -e cosmian_kms_version=4.16.0
```

The machine has been configured
