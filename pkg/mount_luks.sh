#!/bin/bash

set -ex

# unlock the partition
/lib/systemd/systemd-cryptsetup attach cosmian_vm_container /var/lib/cosmian_vm/container - tpm2-device=auto,headless=true

# mount the partition
mount /dev/mapper/cosmian_vm_container /var/lib/cosmian_vm/data
