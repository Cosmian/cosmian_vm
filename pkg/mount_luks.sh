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
        echo "cipher_null in keyslots is not allowed in LUKS header"
        exit 3
    fi

    NULL_CIPHERS=$(echo "$LUKS_DUMP" | jq '[.segments.[].encryption] | select(any(contains("null")))')

    if [ -n "$NULL_CIPHERS" ]; then
        echo "cipher_null in segments is not allowed in LUKS header"
        exit 4
    fi

    export TPM2TOOLS_TCTI="device:/dev/tpmrm0"

    # unlock the partition (retry up to 30 times as the vTPM may not be fully initialized yet)
    MAX_RETRIES=30
    RETRY_DELAY=5
    for i in $(seq 1 $MAX_RETRIES); do
        /lib/systemd/systemd-cryptsetup attach cosmian_vm_container /var/lib/cosmian_vm/container - tpm2-device=auto,headless=true,header=/var/lib/cosmian_vm/header
        STATUS=$?
        if [ $STATUS -eq 0 ]; then
            break
        fi
        echo "TPM unseal attempt $i/$MAX_RETRIES failed (code $STATUS), retrying in ${RETRY_DELAY}s..."
        if [ "$i" -eq "$MAX_RETRIES" ]; then
            echo "Failed to attach LUKS container after $MAX_RETRIES attempts"
            exit 1
        fi
        sleep $RETRY_DELAY
    done
    # mount the partition
    mount /dev/mapper/cosmian_vm_container /var/lib/cosmian_vm/data || exit 1
    exit 0
    ;;
esac
