#!/bin/bash

set -ex

PRODUCT=$1
DISTRIBUTION=$2
COSMIAN_VM_VERSION=$3

set

if [ "$DISTRIBUTION" = "ubuntu" ]; then
  IMAGE_PUBLISHER="Canonical"
  IMAGE_OFFER="0001-com-ubuntu-confidential-vm-jammy"
  IMAGE_SKU="22_04-lts-cvm"
else
  IMAGE_PUBLISHER="redhat"
  IMAGE_OFFER="9_3_cvm_sev_snp"
  IMAGE_SKU="rhel-cvm"
fi

if [ "$PRODUCT" = "cosmian-vm" ]; then
  if [[ $COSMIAN_VM_VERSION = *'last_build'* ]]; then
    AZURE_IMAGE_VERSION="0.0.0"
  else
    AZURE_IMAGE_VERSION="$COSMIAN_VM_VERSION"
  fi
else
  AZURE_IMAGE_VERSION="$KMS_VERSION"
fi

PACKER_FILE="azure.pkr.hcl"

sed -i "s#TEMPLATE_PRODUCT#$PRODUCT#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_CLIENT_ID#$CLIENT_ID#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_TENANT_ID#$TENANT_ID#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SUBSCRIPTION_ID#$SUBSCRIPTION_ID#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_CLIENT_SECRET#$CLIENT_SECRET#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_DISTRIBUTION#$DISTRIBUTION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_RESOURCE_GROUP#$RESOURCE_GROUP#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_IMAGE_PUBLISHER#$IMAGE_PUBLISHER#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_OFFER#$IMAGE_OFFER#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_SKU#$IMAGE_SKU#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_AZURE_IMAGE_VERSION#$AZURE_IMAGE_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_VM_SIZE#$VM_SIZE#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_COSMIAN_VM_VERSION#$COSMIAN_VM_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_KMS_VERSION#${KMS_VERSION}#g" "$PACKER_FILE"

cat "$PACKER_FILE"

packer init "$PACKER_FILE"

# Since packer build fails randomly because of external resources use, retry packer build until it succeeds
timeout 30m bash -c "until packer build -force $PACKER_FILE; do sleep 30; done"
