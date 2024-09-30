#!/bin/bash

set -ex

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"
WHO="$(whoami)"
DEFAULT_NAME="$WHO-$TECHNO-$DISTRIB"
NAME="${3:-$DEFAULT_NAME}"

SSH_PUB_KEY=$(cat ~/.ssh/id_rsa.pub)

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
    --security-type ConfidentialVM \
    --ssh-key-values "$SSH_PUB_KEY"
else
  IMAGE_NAME="/subscriptions/bc07f5de-3498-43b8-94aa-34b4a34a89b8/resourceGroups/packer-snp/providers/Microsoft.Compute/galleries/cosmian_packer/images/base-image-${DISTRIB}-${TECHNO}/versions/0.1.8"
  IMAGE_NAME="/subscriptions/bc07f5de-3498-43b8-94aa-34b4a34a89b8/resourceGroups/packer-snp/providers/Microsoft.Compute/galleries/cosmian_packer/images/cosmian-vm-${DISTRIB}-${TECHNO}/versions/1.2.7"

  if [ "$DISTRIB" = "ubuntu" ]; then
    # Ubuntu SEV
    IMAGE_NAME="Canonical:0001-com-ubuntu-confidential-vm-jammy:22_04-lts-cvm:latest"
  else
    # Redhat SEV
    IMAGE_NAME="redhat:rhel-cvm:9_3_cvm_sev_snp:latest"
  fi

  az vm create -g packer-snp -n "$NAME" \
    --image "$IMAGE_NAME" \
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

az vm open-port -g packer-snp -n "$NAME" --priority 100 --port 5555,443,22

HOST=$(az vm show -d -g packer-snp -n "$NAME" --query publicIps -o tsv)
echo "$HOST"
