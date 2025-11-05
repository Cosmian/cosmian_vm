#!/bin/bash

set -ex

VERSION=1.3.19

set -x
if [[ "${GITHUB_REF}" =~ 'refs/tags/' ]]; then
  export BRANCH="${GITHUB_REF_NAME}"
else
  export BRANCH="last_build/${GITHUB_HEAD_REF:-${GITHUB_REF#refs/heads/}}"
fi

DESTINATION_DIR=/mnt/package/cosmian_vm/${BRANCH}
ssh -o 'StrictHostKeyChecking no' -i /root/.ssh/id_rsa cosmian@package.cosmian.com mkdir -p "$DESTINATION_DIR/${DISTRIB}/"
ssh -o 'StrictHostKeyChecking no' -i /root/.ssh/id_rsa cosmian@package.cosmian.com mkdir -p "$DESTINATION_DIR/rhel9/"

ARTIFACT_NAME=cosmian_vm_${DISTRIB}
ARTIFACT_FOLDER=${ARTIFACT_NAME}/home/runner/work/cosmian_vm/cosmian_vm

scp -o 'StrictHostKeyChecking no' -i /root/.ssh/id_rsa \
  "./$ARTIFACT_FOLDER/target/release/cosmian_vm_agent" \
  "./$ARTIFACT_FOLDER/target/release/cosmian_vm" \
  "./$ARTIFACT_FOLDER/target/release/cosmian_certtool" \
  "./$ARTIFACT_NAME/usr/lib/x86_64-linux-gnu/libtdx_attest.so.1.23.100.0" \
  cosmian@package.cosmian.com:"$DESTINATION_DIR/${DISTRIB}/"

if [[ "${DISTRIB}" = *'ubuntu-22'* ]]; then
  scp -o 'StrictHostKeyChecking no' -i /root/.ssh/id_rsa \
    "./$ARTIFACT_FOLDER/target/debian/cosmian-vm-agent_$VERSION-1_amd64.deb" \
    "./$ARTIFACT_FOLDER/target/debian/cosmian-vm_$VERSION-1_amd64.deb" \
    cosmian@package.cosmian.com:"$DESTINATION_DIR/${DISTRIB}/"
  scp -o 'StrictHostKeyChecking no' -i /root/.ssh/id_rsa \
    "./$ARTIFACT_FOLDER/target/generate-rpm/cosmian_vm-$VERSION-1.x86_64.rpm" \
    "./$ARTIFACT_FOLDER/target/generate-rpm/cosmian_vm_agent-$VERSION-1.x86_64.rpm" \
    cosmian@package.cosmian.com:"$DESTINATION_DIR/rhel9/"
else
  scp -o 'StrictHostKeyChecking no' -i /root/.ssh/id_rsa \
    "./$ARTIFACT_FOLDER/target/debian/cosmian-vm_$VERSION-1_amd64.deb" \
    "./$ARTIFACT_FOLDER/target/debian/cosmian-vm-agent_$VERSION-1_amd64.deb" \
    cosmian@package.cosmian.com:"$DESTINATION_DIR/${DISTRIB}/"
fi

rm -rf cosmian_vm_ubuntu*
