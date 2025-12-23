#!/bin/bash

set -x

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"
PREFIX="${3:-$(whoami)}"
PREFIX=$(echo "$PREFIX" | sed 's/\./-/g; s/_/-/g; s/+/-/g')
DISK_SIZE=20

NAME="$PREFIX-$TECHNO-$DISTRIB"
DURATION=240m

SSH_PUB_KEY=$(cat ~/.ssh/id_rsa.pub)

gcloud compute firewall-rules delete "$NAME" --quiet
gcloud compute instances delete --quiet "$NAME" --zone "us-central1-a" --project cosmian-dev
gcloud compute instances delete --quiet "$NAME" --zone "europe-west4-a" --project cosmian-dev

set -ex

if [ "$TECHNO" = "tdx" ]; then
  # Ubuntu TDX
  gcloud compute instances create "$NAME" \
    --machine-type c3-standard-4 \
    --zone us-central1-a \
    --min-cpu-platform=AUTOMATIC \
    --confidential-compute-type=TDX \
    --shielded-secure-boot \
    --image=ubuntu-2404-noble-amd64-v20251021 \
    --project cosmian-dev \
    --tags "$NAME-cli" \
    --maintenance-policy=TERMINATE \
    --max-run-duration=$DURATION \
    --instance-termination-action=DELETE \
    --boot-disk-size=$DISK_SIZE \
    --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
else
  if [ "$DISTRIB" = "ubuntu" ]; then
    # Base Ubuntu SEV
    IMAGE="base-image-0-1-15-ubuntu-sev"
    IMAGE_PROJECT="cosmian-dev"
    # Cosmian Ubuntu SEV
    IMAGE="cosmian-vm-1-3-20-sev-ubuntu"
    IMAGE_PROJECT="cosmian-dev"
    # Cosmian KMS Ubuntu SEV
    IMAGE="cosmian-vm-1-3-20-kms-5-11-0-sev-ubuntu"
    IMAGE_PROJECT="cosmian-dev"
    # Ubuntu SEV
    IMAGE="ubuntu-2404-noble-amd64-v20251021"
    IMAGE_PROJECT="ubuntu-os-cloud"
  else
    # Base RHEL SEV
    IMAGE="base-image-0-1-15-rhel-sev"
    IMAGE_PROJECT="cosmian-dev"
    # Cosmian vm RHEL SEV
    IMAGE="cosmian-vm-1-3-20-rhel-sev"
    IMAGE_PROJECT="cosmian-dev"
    # Cosmian kms RHEL SEV
    IMAGE="cosmian-vm-1-3-20-kms-5-11-0-sev-rhel"
    IMAGE_PROJECT="cosmian-dev"
    # Cosmian ai runner RHEL SEV
    IMAGE="cosmian-vm-1-3-14-ai-runner-1-0-1-sev-rhel"
    IMAGE_PROJECT="cosmian-dev"
    # RHEL SEV
    IMAGE="rocky-linux-10-v20251113"
    IMAGE_PROJECT="rocky-linux-cloud"
  fi

  if [ "$IMAGE" = "cosmian-vm-1-3-14-ai-runner-1-0-1-sev-rhel" ]; then
    DISK_SIZE=75
  else
    DISK_SIZE=20
  fi

  gcloud compute instances create "$NAME" \
    --machine-type n2d-standard-2 \
    --zone europe-west4-a \
    --min-cpu-platform='AMD Milan' \
    --confidential-compute-type=SEV_SNP \
    --shielded-secure-boot \
    --image=$IMAGE --image-project=$IMAGE_PROJECT \
    --project cosmian-dev \
    --tags "$NAME-cli" \
    --maintenance-policy=TERMINATE \
    --instance-termination-action=DELETE \
    --max-run-duration=$DURATION \
    --boot-disk-size=$DISK_SIZE \
    --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
fi

gcloud compute firewall-rules create "$NAME" --network=default --allow=tcp:22,tcp:5555,tcp:443 --target-tags="$NAME-cli"
