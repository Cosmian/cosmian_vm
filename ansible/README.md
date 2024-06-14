# Image build

## Cloud providers base images

### Changelog

## [0.1.5] - 2024-06-15

- RHEL/Ubuntu: clean all authorized_keys ([#139](https://github.com/Cosmian/cosmian_vm/pull/139))

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
ansible-playbook cosmian-vm-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.3
ansible-playbook kms-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.3 -e cosmian_kms_version=4.16.0
```

The machine has been configured
