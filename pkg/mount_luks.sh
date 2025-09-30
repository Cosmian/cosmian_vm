#!/bin/bash

set -x

mountpoint /var/lib/cosmian_vm/data

case $? in
# success; the directory is a mountpoint, or device is block device on --devno
0)
    exit 0
    ;;
# failure; incorrect invocation, permissions or system error
1)
    exit 1
    ;;
# failure; the directory is not a mountpoint, or device is not a block device on --devno
32)
    # unlock the partition
    /lib/systemd/systemd-cryptsetup attach cosmian_vm_container /var/lib/cosmian_vm/container - tpm2-device=auto,headless=true,header=/var/lib/cosmian_vm/header || exit 1
    # mount the partition
    mount /dev/mapper/cosmian_vm_container /var/lib/cosmian_vm/data || exit 1
    exit 0
    ;;
esac
