#!/bin/bash

set +ex

export STORAGE_ACCOUNT_KEY="XXX" # Go to Passbolt
export STORAGE_ACCOUNT_NAME="packercosmian"

VM_NAME="$1"

az storage blob show --container-name packer --name "$VM_NAME.vhd" --account-name $STORAGE_ACCOUNT_NAME --account-key $STORAGE_ACCOUNT_KEY | jq '.properties.copy.progress'
