# Test Azure VHD RHEL deployment

1. Instantiate rhel-cvm:9_4_cvm
2. Run Ansible playbook base-image-packer-playbook.yml
3. Create VHD (always in success)
4. Deploy VHD

Below are detailed steps for the Ansible IMA configuration.

| Create IMA folder | Copy IMA policy | Run grub.sh | Update GRUB | Deploy VHD |
| ----------------- | --------------- | ----------- | ----------- | ---------- |
| x                 |                 |             |             | OK         |
| x                 | x               |             |             | OK         |
| x                 | x               |             | x           | OK         |
| x                 | x               | x           | x           | KO         |

## Zoom on grub.sh execution

Below are the options selected on the GRUB configuration.

| ima_policy | ima_hash | ima_template | ttyS0 | earlyprintk | Result |
| ---------- | -------- | ------------ | ----- | ----------- | ------ |
|            |          |              |       |             | OK     |
| x          |          |              |       |             | KO     |

## Note

<https://packercosmian.blob.core.windows.net/packer/ima1.vhd>
<https://packercosmian.blob.core.windows.net/packer/ima6.vhd>
