#!/bin/sh

set -ex

PRODUCT=$1
DISTRIBUTION=$2
COSMIAN_VM_VERSION=$3

set

if [ "$DISTRIBUTION" = "ubuntu" ]; then
  SOURCE_IMAGE="ubuntu-2204-jammy-v20240319"
  SOURCE_IMAGE_FAMILY="ubuntu-2204-lts"
  GOOGLE_COMPUTE="ubuntu"
else
  SOURCE_IMAGE="rhel-9-v20240312"
  SOURCE_IMAGE_FAMILY="rhel-9"
  GOOGLE_COMPUTE="redhat"
fi

PACKER_FILE="gcp.pkr.hcl"

sed -i "s#TEMPLATE_PRODUCT#$PRODUCT#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_NAME#$IMAGE_NAME#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SOURCE_IMAGE#$SOURCE_IMAGE#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SOURCE_FAMILY#$SOURCE_IMAGE_FAMILY#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_GOOGLE_COMPUTE#$GOOGLE_COMPUTE#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_VM_VERSION#$COSMIAN_VM_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_KMS_VERSION#${KMS_VERSION}#g" "$PACKER_FILE"

cat "$PACKER_FILE"

packer init "$PACKER_FILE"

# Since packer build fails randomly because of external resources use, retry packer build until it succeeds
timeout 30m bash -c "until packer build $PACKER_FILE; do sleep 30; done"
