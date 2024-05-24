#!/bin/sh

set -ex

PRODUCT=$1
DISTRIBUTION=$2
SOURCE_AMI=$3

VOLUME_SIZE=20

set

if [ "$DISTRIBUTION" = "ubuntu" ]; then
  SSH_USERNAME="ubuntu"
  TEMPLATE_DISTRIBUTION="ubuntu"
else
  SSH_USERNAME="ec2-user"
  TEMPLATE_DISTRIBUTION="redhat"
fi

PACKER_FILE="aws.pkr.hcl"
DEVICE_NAME="/dev/sda1"
SUPPORT="v2.0"
VOLUME_TYPE="gp3"

sed -i "s#TEMPLATE_PRODUCT#$PRODUCT#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_NAME#$IMAGE_NAME#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SOURCE_AMI#$SOURCE_AMI#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SUPPORT#$SUPPORT#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_VOLUME_SIZE#$VOLUME_SIZE#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SSH_USERNAME#$SSH_USERNAME#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_DEVICE_NAME#$DEVICE_NAME#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_VOLUME_TYPE#$VOLUME_TYPE#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_DISTRIBUTION#$TEMPLATE_DISTRIBUTION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_VM_VERSION#$COSMIAN_VM_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_KMS_VERSION#$KMS_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_AI_RUNNER_VERSION#$AI_RUNNER_VERSION#g" "$PACKER_FILE"

cat "$PACKER_FILE"

packer init "$PACKER_FILE"

# Since packer build fails randomly because of external resources use, retry packer build until it succeeds
timeout 30m bash -c "until packer build $PACKER_FILE; do sleep 30; done"
