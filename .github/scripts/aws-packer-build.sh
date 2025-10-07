#!/bin/bash

# Only for testing, DO NOT UNCOMMENT
# DISTRIBUTION=rhel
# PRODUCT=cosmian-vm
# VERSION=0.1.12 # Optional
# KMS_VERSION=5.7.1 # Provided by Github workflow
# AI_RUNNER_VERSION=1.0.1 # Provided by Github workflow
# GITHUB_REF=refs/tags/1.3.15 # Provided by Github Actions
# GITHUB_REF_NAME=1.3.15 # Provided by Github Actions
# IMAGE_NAME="cosmian-vm-${GITHUB_REF_NAME}-sev-${DISTRIBUTION}" # Only for testing

set -ex

if [[ "${GITHUB_REF}" =~ 'refs/tags/' ]]; then
  export COSMIAN_VM_VERSION="$GITHUB_REF_NAME"
else
  export COSMIAN_VM_VERSION="last_build/${GITHUB_HEAD_REF:-${GITHUB_REF#refs/heads/}}"
fi

if [ -n "${VERSION+x}" ]; then
  BASE_VERSION=$(echo "$VERSION" | sed 's/\./-/g; s/_/-/g; s/+/-/g')
  BASE_IMAGE_NAME="base-image-${BASE_VERSION}-${DISTRIBUTION}-sev"
else
  if [ "$DISTRIBUTION" = "ubuntu" ]; then
    BASE_IMAGE_NAME="ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-20250610"
  else
    BASE_IMAGE_NAME="RHEL-9.4.0_HVM-20250519-x86_64-0-Hourly2-GP3"
  fi
fi

SOURCE_AMI=$(aws ec2 describe-images --filters "Name=name,Values=${BASE_IMAGE_NAME}" --query "Images[*].{ID:ImageId}" --output text)

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
timeout 60m bash -c "until packer build $PACKER_FILE; do sleep 30; done"
