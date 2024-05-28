#!/bin/bash

set -x

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"

WHO="$(whoami)"
NAME="$WHO-$TECHNO-$DISTRIB"

SSH_PUB_KEY=$(cat ~/.ssh/id_rsa.pub)

az vm delete -g packer-snp -n "$NAME" --yes
az network public-ip delete -g packer-snp -n "${NAME}PublicIP"
az network nsg delete --resource-group packer-snp -n "${NAME}NSG"

set -ex

if [ "$TECHNO" = "tdx" ]; then
  # Ubuntu TDX
  az vm create \
    --resource-group packer-snp \
    --name "$NAME" \
    --size Standard_DC2es_v5 \
    --enable-secure-boot true \
    --image "Canonical:0001-com-ubuntu-confidential-vm-jammy:22_04-lts-cvm:latest" \
    --public-ip-sku Standard \
    --admin-username azureuser \
    --os-disk-delete-option delete \
    --nic-delete-option delete \
    --data-disk-delete-option delete \
    --ssh-key-values "$SSH_PUB_KEY"
else
  if [ "$DISTRIB" = "ubuntu" ]; then
    # Ubuntu SEV
    az vm create -g packer-snp -n "$NAME" \
      --image "Canonical:0001-com-ubuntu-confidential-vm-jammy:22_04-lts-cvm:latest" \
      --security-type ConfidentialVM \
      --os-disk-security-encryption-type VMGuestStateOnly \
      --size Standard_DC2ads_v5 \
      --enable-vtpm true \
      --enable-secure-boot true \
      --nic-delete-option delete \
      --os-disk-delete-option delete \
      --data-disk-delete-option delete \
      --admin-username azureuser \
      --ssh-key-values "$SSH_PUB_KEY"
  else
    # Redhat SEV
    az vm create -g packer-snp -n "$NAME" \
      --image "redhat:rhel-cvm:9_3_cvm_sev_snp:latest" \
      --security-type ConfidentialVM \
      --os-disk-security-encryption-type VMGuestStateOnly \
      --size Standard_DC2ads_v5 \
      --enable-vtpm true \
      --enable-secure-boot true \
      --nic-delete-option delete \
      --os-disk-delete-option delete \
      --data-disk-delete-option delete \
      --admin-username azureuser \
      --ssh-key-values "$SSH_PUB_KEY"
  fi
fi

az vm open-port -g packer-snp -n "$NAME" --priority 100 --port 5555,443,22
