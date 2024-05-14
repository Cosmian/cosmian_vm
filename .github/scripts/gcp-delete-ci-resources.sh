#!/bin/bash

set +e

# Set your GCP project ID
PROJECT_ID="cosmian-dev"
ZONE="europe-west4-a"

# List all instances and extract instance names
instance_names=$(gcloud compute instances list --project "$PROJECT_ID" --format="value(name)")
# Loop through each instance name and delete it
for instance_name in $instance_names; do
  if [[ $instance_name == *"packer"* ]] || [[ $instance_name == *"gh-ci"* ]]; then
    echo "Deleting instance: $instance_name"
    gcloud compute instances delete "$instance_name" --project "$PROJECT_ID" --zone "$ZONE" --quiet
  fi
done

# List all disks and extract their names
disk_names=$(gcloud compute disks list --project "$PROJECT_ID" --format="value(name)")
# Loop through each disk name and delete it
for disk_name in $disk_names; do
  if [[ $disk_name == *"packer"* ]] || [[ $disk_name == *"gh-ci"* ]]; then
    echo "Deleting disk: $disk_name"
    gcloud compute disks delete "$disk_name" --project "$PROJECT_ID" --zone "$ZONE" --quiet
  fi
done

# List all firewalls and extract their names
firewall_names=$(gcloud compute firewall-rules list --project "$PROJECT_ID" --format="value(name)")
# Loop through each firewall name and delete it
for firewall_name in $firewall_names; do
  if [[ $firewall_name == *"packer"* ]] || [[ $firewall_name == *"gh-ci"* ]]; then
    echo "Deleting firewall: $firewall_name"
    gcloud compute firewall-rules delete "$firewall_name" --project "$PROJECT_ID" --quiet
  fi
done

# List all networks and extract their names
network_names=$(gcloud compute networks list --project "$PROJECT_ID" --format="value(name)")
# Loop through each network name and delete it
for network_name in $network_names; do
  if [[ $network_name == *"packer"* ]] || [[ $network_name == *"gh-ci"* ]]; then
    echo "Deleting network: $network_name"
    gcloud compute networks delete "$network_name" --project "$PROJECT_ID" --quiet
  fi
done

# List all images and extract their names
image_names=$(gcloud compute images list --project "$PROJECT_ID" --format="value(name)")
# Loop through each image name and delete it if it starts with "temp"
for image_name in $image_names; do
  if [[ $image_name == "temp-"* ]]; then
    echo "Deleting image: $image_name"
    gcloud compute images delete "$image_name" --project "$PROJECT_ID" --quiet
  fi
done

for i in $(gcloud compute os-login ssh-keys list --format="table[no-heading](value.fingerprint)"); do
  echo $i;
  gcloud compute os-login ssh-keys remove --impersonate-service-account="packer@cosmian-dev.iam.gserviceaccount.com" --key $i || true;
done
