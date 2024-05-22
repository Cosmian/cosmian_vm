#!/bin/sh

set -ex

PRODUCT=$1
DISTRIBUTION=$2
COSMIAN_VM_VERSION=$3

set

if [ "$DISTRIBUTION" = "ubuntu" ]; then
  SOURCE_AMI="ami-083360161b7e953b6"
  SSH_USERNAME="ubuntu"
  TEMPLATE_DISTRIBUTION="ubuntu"
  VOLUME_SIZE=8
else
  SOURCE_AMI="ami-02d912d1649d1e091"
  SSH_USERNAME="ec2-user"
  TEMPLATE_DISTRIBUTION="redhat"
  VOLUME_SIZE=12
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
sed -i "s#TEMPLATE_COSMIAN_KMS_VERSION#${KMS_VERSION}#g" "$PACKER_FILE"

cat "$PACKER_FILE"

packer init "$PACKER_FILE"

# Since packer build fails randomly because of external resources use, retry packer build until it succeeds
timeout 30m bash -c "until packer build $PACKER_FILE; do sleep 30; done"
