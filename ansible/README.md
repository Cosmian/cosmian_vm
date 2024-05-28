# Image build

## Cloud providers base images

### AWS images

|               |    Official image     | OS image | OS version | Cosmian base image    | Version | Creation date |
| :------------ | :-------------------: | :------: | ---------- | --------------------- | ------- | ------------- |
| AWS - AMD SEV | ami-0655bf2193e40564e |  Ubuntu  | 24.04      | ami-02398b8659888fbd8 | 0.1.0   | 2024-05-27    |
| AWS - AMD SEV | ami-08e592fbb0f535224 |  Redhat  | 9.3        | ami-0f1406ea14dde8c22 | 0.1.0   | 2024-05-27    |

If needed:

```sh
aws ec2 describe-images > aws_list.txt
```

### Azure images

|                   |                        Official image                         | OS image | OS version      | Cosmian base image    | Version | Creation date |
| :---------------- | :-----------------------------------------------------------: | :------: | --------------- | --------------------- | ------- | ------------- |
| Azure - Intel TDX | Canonical-0001-com-ubuntu-confidential-vm-jammy-22_04-lts-cvm |  Ubuntu  | 22.04.202404090 | base-image-ubuntu-tdx | 0.1.0   | 2024-05-28    |
| Azure - AMD SEV   | Canonical-0001-com-ubuntu-confidential-vm-jammy-22_04-lts-cvm |  Ubuntu  | 22.04.202404090 | base-image-ubuntu-sev | 0.1.0   | 2024-05-28    |
| Azure - AMD SEV   |                Redhat-rhel-cvm-9_3_cvm_sev_snp                |  Redhat  | 9.3.2023111017  | base-image-rhel-sev   | 0.1.0   | 2024-05-28    |

```sh
az vm list> azure_list.json
```

### GCP images

|                 |           Official image           | OS image | OS version | Cosmian base image                   | Creation date |
| :-------------- | :--------------------------------: | :------: | ---------- | ------------------------------------ | ------------- |
| GCP - Intel TDX |     ubuntu-2204-tdx-v20240220      |  Ubuntu  | 22.04      | base-image-0-1-0-ubuntu-tdx          | 2024-05-28    |
| GCP - AMD SEV   |    ubuntu-2204-jammy-v20240519     |  Ubuntu  | 22.04      | base-image-ubuntu-sev-20240528081327 | 2024-05-28    |
| GCP - AMD SEV   | ubuntu-2404-noble-amd64-v20240523a |  Ubuntu  | 24.04      | base-image-0-1-0-ubuntu-sev          | 2024-05-28    |
| GCP - AMD SEV   |          rhel-9-v20240515          |  Redhat  | 9.3        | base-image-0-1-0-rhel-sev            | 2024-05-28    |

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
ansible-playbook cosmian-vm-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.0
ansible-playbook kms-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.2.0 -e cosmian_kms_version=4.16.0
```

The machine has been configured
