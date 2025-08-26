#!/bin/sh

set -ex

PRODUCT=$1
DISTRIBUTION=$2
SOURCE_IMAGE=$3

set

if [ "$DISTRIBUTION" = "ubuntu" ]; then
  SOURCE_IMAGE_FAMILY="ubuntu-2204-lts"
  GOOGLE_COMPUTE="ubuntu"
else
  SOURCE_IMAGE_FAMILY="rhel-9"
  GOOGLE_COMPUTE="redhat"
fi

if [ "$TECHNO" = "sev" ]; then
  OS_FEATURES="SEV_SNP_CAPABLE"
else
  OS_FEATURES="TDX_CAPABLE"
fi

PACKER_FILE="gcp.pkr.hcl"

sed -i "s#TEMPLATE_PRODUCT#$PRODUCT#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_OS_FEATURES#$OS_FEATURES#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_IMAGE_NAME#$IMAGE_NAME#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SOURCE_IMAGE#$SOURCE_IMAGE#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_SOURCE_FAMILY#$SOURCE_IMAGE_FAMILY#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_GOOGLE_COMPUTE#$GOOGLE_COMPUTE#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_VM_VERSION#$COSMIAN_VM_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_KMS_VERSION#$KMS_VERSION#g" "$PACKER_FILE"
sed -i "s#TEMPLATE_COSMIAN_AI_RUNNER_VERSION#$AI_RUNNER_VERSION#g" "$PACKER_FILE"

cat "$PACKER_FILE"

plugins='https://github.com/hashicorp/packer-plugin-ansible.git https://github.com/hashicorp/packer-plugin-googlecompute.git'

for plugin in $plugins; do
  git clone $plugin
  plugin_name=$(echo "$plugin" | sed -E 's#.*/([^/]+)\.git#\1#')
  cd $plugin_name
  go build
  ./$plugin_name describe
  packer plugins install --path $plugin_name releases.hashicorp.com/latest/$plugin_name
  cd ..
done

# Since packer build fails randomly because of external resources use, retry packer build until it succeeds
timeout 60m bash -c "until packer build $PACKER_FILE; do sleep 30; done"
