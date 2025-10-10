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
    LUKS_DUMP=$(cryptsetup luksDump --dump-json-metadata /var/lib/cosmian_vm/header)
    STATUS=$?

    if [ $STATUS -ne 0 ]; then
        echo "LUKS header does not exist"
        exit 2
    fi

    NULL_CIPHERS=$(echo "$LUKS_DUMP" | jq '[.keyslots.[].area.encryption] | select(any(contains("null")))')

    if [ -n "$NULL_CIPHERS" ]; then
        echo "cipher_null is not allowed in LUKS header"
        exit 3
    fi 

    # unlock the partition
    /lib/systemd/systemd-cryptsetup attach cosmian_vm_container /var/lib/cosmian_vm/container - tpm2-device=auto,headless=true,header=/var/lib/cosmian_vm/header || exit 1
    # mount the partition
    mount /dev/mapper/cosmian_vm_container /var/lib/cosmian_vm/data || exit 1
    exit 0
    ;;
esac
