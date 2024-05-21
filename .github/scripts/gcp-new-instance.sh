#!/bin/bash

set -x

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"

WHO="$(whoami)"
NAME="$WHO-$TECHNO-$DISTRIB"

DURATION=240m
REGION="us-central1-a"

SSH_PUB_KEY=$(cat ~/.ssh/id_rsa.pub)

gcloud compute firewall-rules delete "$NAME"
gcloud beta compute instances delete --quiet "$NAME" --zone $REGION --project cosmian-dev

set -ex

if [ "$TECHNO" = "tdx" ]; then
  # Ubuntu TDX
  gcloud alpha compute instances create "$NAME" \
    --machine-type c3-standard-4 \
    --zone $REGION \
    --min-cpu-platform=AUTOMATIC \
    --confidential-compute-type=TDX \
    --shielded-secure-boot \
    --image=ubuntu-2204-tdx-v20240220 \
    --project cosmian-dev \
    --tags "$NAME-cli" \
    --maintenance-policy=TERMINATE \
    --max-run-duration=$DURATION \
    --instance-termination-action=DELETE \
    --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
else
  if [ "$DISTRIB" = "ubuntu" ]; then
    # Ubuntu SEV
    gcloud beta compute instances create "$NAME" \
      --machine-type n2d-standard-2 \
      --zone europe-west4-a \
      --min-cpu-platform='AMD Milan' \
      --confidential-compute-type=SEV_SNP \
      --shielded-secure-boot \
      --image=ubuntu-2204-jammy-v20240515 --image-project=ubuntu-os-cloud \
      --project cosmian-dev \
      --tags "$NAME-cli" \
      --maintenance-policy=TERMINATE \
      --instance-termination-action=DELETE \
      --max-run-duration=$DURATION \
      --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
  else
    # RHEL SEV
    gcloud beta compute instances create "$NAME" \
      --machine-type n2d-standard-2 \
      --zone europe-west4-a \
      --min-cpu-platform='AMD Milan' \
      --confidential-compute-type=SEV_SNP \
      --shielded-secure-boot \
      --image=rhel-9-v20240312 --image-project=rhel-cloud \
      --project cosmian-dev \
      --tags "$NAME-cli" \
      --maintenance-policy=TERMINATE \
      --instance-termination-action=DELETE \
      --max-run-duration=$DURATION \
      --metadata=ssh-keys="cosmian:$SSH_PUB_KEY"
  fi
fi

gcloud compute firewall-rules create "$NAME" --network=default --allow=tcp:22,tcp:5555 --target-tags="$NAME-cli"