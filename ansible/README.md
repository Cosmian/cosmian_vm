# Image build

## Cloud providers base images

### AWS images

|               | Official image | OS image | OS version | AMI                   | Creation date |
|:--------------|:--------------:|:--------:|------------|-----------------------|---------------|
| AWS - AMD SEV |      yes       |  Ubuntu  | 24.04      | ami-083360161b7e953b6 | 2024-05-26    |
| AWS - AMD SEV |      yes       |  Redhat  | 9.3        | ami-02d912d1649d1e091 | 2024-05-26    |
| AWS - AMD SEV |      yes       |  Ubuntu  | 24.04      | ami-0655bf2193e40564e | 2024-05-27    |
| AWS - AMD SEV |      yes       |  Redhat  | 9.3        | ami-08e592fbb0f535224 | 2024-05-27    |
| AWS - AMD SEV |       no       |  Ubuntu  | 24.04      | ami-0620af58af89e9980 | 2024-05-27    |
| AWS - AMD SEV |       no       |  Redhat  | 9.3        | ami-06707d5f1aeecc075 | 2024-05-27    |

If needed:

```sh
aws ec2 describe-images > aws_list.txt
```

### Azure images

|                             | Official image | OS image | OS version      | Publisher            | Offer                                 | SKU             | Creation date |
|:----------------------------|:--------------:|:--------:|-----------------|----------------------|---------------------------------------|-----------------|---------------|
| Azure - AMD SEV / Intel TDX |      yes       |  Ubuntu  | 22.04.202404090 | Canonical            | 0001-com-ubuntu-confidential-vm-jammy | 22_04-lts-cvm   | 2024-04-18    |
| Azure - AMD SEV / Intel TDX |      yes       |  Redhat  | 9.3.2023111017  | Redhat               | rhel-cvm                              | 9_3_cvm_sev_snp | 2024-04-18    |
| Azure - AMD SEV / Intel TDX |      yes       |  Ubuntu  | 22.04.202404090 | Canonical            | 0001-com-ubuntu-confidential-vm-jammy | 22_04-lts-cvm   | 2024-04-18    |
| Azure - AMD SEV / Intel TDX |      yes       |  Redhat  | 9.3.2023111017  | Redhat               | rhel-cvm                              | 9_3_cvm_sev_snp | 2024-04-18    |
| Azure - AMD SEV             |       no       |  Redhat  | 0.1.0           | base-image-rhel-sev  | rhel-cvm                              | 9_3_cvm_sev_snp | 2024-05-27    |
| Azure - AMD TDX             |       no       |  Ubuntu  | 0.1.0           | base-image-ubntu-tdx | Canonical                             | 22_04-lts-cvm   | 2024-05-27    |
| Azure - AMD SEV             |       no       |  Ubuntu  | 0.1.0           | base-image-ubntu-sev | Canonical                             | 22_04-lts-cvm   | 2024-05-27    |


```sh
az vm list> azure_list.json
```

### GCP images

|                 | Official image | OS image | OS version | Name                                 | Creation date |
|:----------------|:--------------:|:--------:|------------|--------------------------------------|---------------|
| GCP - Intel TDX |      yes       |  Ubuntu  | 22.04      | ubuntu-2204-tdx-v20240220            | 2024-02-20    |
| GCP - AMD SEV   |      yes       |  Ubuntu  | 22.04      | ubuntu-2204-jammy-v20240319          | 2024-03-19    |
| GCP - AMD SEV   |      yes       |  Redhat  | 9.3        | rhel-9-v20240312                     | 2024-03-12    |
| GCP - Intel TDX |       no       |  Ubuntu  | 22.04      | base-image-ubuntu-tdx-20240527141624 | 2024-05-27    |
| GCP - AMD SEV   |       no       |  Ubuntu  | 22.04      | base-image-ubuntu-sev-20240527141626 | 2024-05-27    |
| GCP - AMD SEV   |       np       |  Redhat  | 9.3        | base-image-rhel-sev-20240527141627   | 2024-05-27    |

```sh
gcloud compute images list > gcloud_list.json
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
ansible-playbook cosmian-vm-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.0
ansible-playbook kms-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.0 -e cosmian_kms_version=4.16.0
```

The machine has been configured
