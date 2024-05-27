# Ansible + Packer

# AWS images

|               |   Ubuntu base image   |    RHEL base image    | Creation date |
|:--------------|:---------------------:|:---------------------:|---------------|
| AWS - AMD SEV | ami-083360161b7e953b6 | ami-02d912d1649d1e091 |               |

# Azure images

|                  | Ubuntu base image | RHEL base image | Creation date |
|:-----------------|:-----------------:|:---------------:|---------------|
| Azure - AMD SEV  |     canonical     |     redhat      |               |
| Azure- Intel TDX |                   |                 |               |

# GCP images

|                 |      Ubuntu base image      | RHEL base image  | Creation date |
|:----------------|:---------------------------:|:----------------:|---------------|
| GCP - Intel TDX |  ubuntu-2204-tdx-v20240220  |                  |               |
| GCP - AMD SEV   | ubuntu-2204-jammy-v20240319 | rhel-9-v20240312 |               |

## Manually run playbook

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
