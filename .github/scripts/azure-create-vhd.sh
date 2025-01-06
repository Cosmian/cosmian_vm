#!/bin/bash

set -x

reset

VM_NAME="$1"

export AZURE_STORAGE_ACCOUNT_NAME="packercosmian"
export AZURE_STORAGE_ACCOUNT_KEY="XXX" # Go to Passbolt

# Clean up
# for i in {1..8}; do
#   bash .github/scripts/azure-delete-instance.sh sev rhel "vhd-ima$i"
#   bash .github/scripts/azure-delete-instance.sh sev rhel "ima$i"
# done

bash .github/scripts/azure-delete-instance.sh sev rhel "vhd-$VM_NAME"
bash .github/scripts/azure-delete-instance.sh sev rhel "$VM_NAME"

set -e

bash .github/scripts/azure-new-instance.sh sev rhel "$VM_NAME"

cd ansible || exit 1
HOST=$(az vm show -d -g packer-snp -n "$VM_NAME" --query publicIps -o tsv)
export ANSIBLE_HOST_KEY_CHECKING=False
ansible-playbook base-image-packer-playbook.yml -i "$HOST", -u azureuser
cd ..

DISK=$(az vm show --resource-group packer-snp --name "$VM_NAME" --query "storageProfile.osDisk.managedDisk.id" -o tsv)
az snapshot create --resource-group packer-snp --source "$DISK" --name "snapshot-$VM_NAME"
az disk create --resource-group packer-snp --name "disk-$VM_NAME" --source "snapshot-$VM_NAME"

SAS=$(az disk grant-access --resource-group packer-snp --name "disk-$VM_NAME" --duration-in-seconds 3600 --access-level Read --query accessSas -o tsv)
az storage blob copy start --destination-blob "$VM_NAME.vhd" --destination-container packer --account-name $AZURE_STORAGE_ACCOUNT_NAME --account-key $AZURE_STORAGE_ACCOUNT_KEY --source-uri "$SAS"

az storage blob show --container-name packer --name "$VM_NAME.vhd" --account-name $AZURE_STORAGE_ACCOUNT_NAME --account-key $AZURE_STORAGE_ACCOUNT_KEY | jq '.properties.copy.progress'

sed "s/ima1/$VM_NAME/g" .github/scripts/template_vhd.json >vhd.json
