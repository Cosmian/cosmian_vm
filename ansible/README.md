This ansible is designed to work with packer.

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
ansible-playbook cosmian-vm-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.1.1
ansible-playbook kms-playbook.yml -i ${HOST}, -u $USERNAME -e cosmian_vm_version=1.1.1 -e cosmian_kms_version=4.15.0
```

The machine has been configured
