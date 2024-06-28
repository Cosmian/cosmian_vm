#!/bin/bash

set -x

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"

WHO="$(whoami)"
NAME="$WHO-$TECHNO-$DISTRIB"

DURATION=240m

SSH_PUB_KEY=$(cat ~/.ssh/id_rsa.pub)

gcloud compute firewall-rules delete "$NAME" --quiet
gcloud beta compute instances delete --quiet "$NAME" --zone "us-central1-a" --project cosmian-dev
gcloud beta compute instances delete --quiet "$NAME" --zone "europe-west4-a" --project cosmian-dev

set -ex

if [ "$TECHNO" = "tdx" ]; then
  # Ubuntu TDX
  gcloud alpha compute instances create "$NAME" \
    --machine-type c3-standard-4 \
    --zone us-central1-a \
    --min-cpu-platform=AUTOMATIC \
    --confidential-compute-type=TDX \
    --shielded-secure-boot \
    --image=ubuntu-2204-tdx-v20240220 \
    --project cosmian-dev \
    --tags "$NAME-cli" \
    --maintenance-policy=TERMINATE \
    --max-run-duration=$DURATION \
    --instance-termination-action=DELETE \
    --boot-disk-size=20GB \
    --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
else
  if [ "$DISTRIB" = "ubuntu" ]; then
    # Ubuntu SEV
    IMAGE="ubuntu-2404-noble-amd64-v20240523a"
    IMAGE_PROJECT="ubuntu-os-cloud"
    # Cosmian Ubuntu SEV
    IMAGE="base-image-0-1-5-ubuntu-sev"
    IMAGE_PROJECT="cosmian-dev"
  else
    # RHEL SEV
    IMAGE="rhel-9-v20240515"
    IMAGE_PROJECT="rhel-cloud"
    # Cosmian Ubuntu SEV
    IMAGE="base-image-0-1-5-rhel-sev"
    IMAGE_PROJECT="cosmian-dev"
  fi
  gcloud beta compute instances create "$NAME" \
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
    --boot-disk-size=20GB \
    --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
fi

gcloud compute firewall-rules create "$NAME" --network=default --allow=tcp:22,tcp:5555,tcp:443 --target-tags="$NAME-cli"
