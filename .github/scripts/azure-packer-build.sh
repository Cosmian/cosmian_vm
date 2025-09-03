#!/bin/bash

set -ex

PRODUCT=$1
DISTRIBUTION=$2

set

if [ "$DISTRIBUTION" = "ubuntu" ]; then
  IMAGE_PUBLISHER="canonical"
  IMAGE_OFFER="ubuntu-24_04-lts"
  IMAGE_SKU="cvm"
else
  IMAGE_PUBLISHER="redhat"
  IMAGE_OFFER="rhel-cvm"
  IMAGE_SKU="9_4_cvm"
fi

if [ "$TECHNO" = "sev" ]; then
  VM_SIZE="Standard_DC2ads_v5"
else
  # TDX
  VM_SIZE="Standard_DC2es_v6"
fi

PACKER_FILE="azure.pkr.hcl"

if [ "$KEEP_OS_DISK" = "true" ]; then
  sed -i "s#TEMPLATE_OS_DISK_NAME#$OS_DISK_NAME#g" "$PACKER_FILE"
else
  sed -i "s#  temp_os_disk_name         = \"TEMPLATE_OS_DISK_NAME\"##g" "$PACKER_FILE"
fi

sed -i "s#TEMPLATE_PRODUCT#$PRODUCT#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_CLIENT_ID#$CLIENT_ID#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_TENANT_ID#$TENANT_ID#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SUBSCRIPTION_ID#$SUBSCRIPTION_ID#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_CLIENT_SECRET#$CLIENT_SECRET#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_DISTRIBUTION#$DISTRIBUTION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_TECHNO#$TECHNO#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_RESOURCE_GROUP#$RESOURCE_GROUP#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_KEEP_OS_DISK#$KEEP_OS_DISK#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_IMAGE_PUBLISHER#$IMAGE_PUBLISHER#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_OFFER#$IMAGE_OFFER#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_SKU#$IMAGE_SKU#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_AZURE_IMAGE_VERSION#$AZURE_IMAGE_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_VM_SIZE#$VM_SIZE#g" "$PACKER_FILE"

sed -i "s#TEMPLATE_COSMIAN_VM_VERSION#$COSMIAN_VM_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_KMS_VERSION#$KMS_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_AI_RUNNER_VERSION#$AI_RUNNER_VERSION#g" "$PACKER_FILE"

if [ ! "$PRODUCT" = "base-image" ]; then
  # We want to use the shared_image_gallery parameters
  sed -i "s/# //g" "$PACKER_FILE"
  sed -i "s/image_publisher/# image_publisher/g" "$PACKER_FILE"
  sed -i "s/image_offer/# image_offer/g" "$PACKER_FILE"
  sed -i "s/image_sku/# image_sku/g" "$PACKER_FILE"
  sed -i "s#TEMPLATE_BASE_IMAGE_VERSION#$BASE_IMAGE_VERSION#g" "$PACKER_FILE"
fi

cat "$PACKER_FILE"

packer init "$PACKER_FILE"

# Since packer build fails randomly because of external resources use, retry packer build until it succeeds
timeout 60m bash -c "until packer build -force $PACKER_FILE; do sleep 30; done"
