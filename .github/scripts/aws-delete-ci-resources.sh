#!/bin/bash

set +e

# Set your AWS region
REGION="eu-west-1"

# List all EC2 instance IDs and extract instance IDs
instance_ids=$(aws ec2 describe-instances --query 'Reservations[].Instances[].InstanceId' --region "$REGION" --output text)
# Loop through each instance ID and terminate it
for instance_id in $instance_ids; do
  echo "Listing instance: $instance_name ($instance_id)"
  instance_name=$(aws ec2 describe-tags --filters "Name=resource-id,Values=$instance_id" "Name=key,Values=Name" --region "$REGION" --output=text | cut -f5)
  if [[ $instance_name == *"packer"* ]] || [[ $instance_name == *"gh-ci"* ]]; then
    echo "--> Terminating instance: $instance_name ($instance_id)"
    aws ec2 terminate-instances --instance-ids "$instance_id" --region "$REGION"
  fi
done

# List all EBS volume IDs and extract their IDs
volume_ids=$(aws ec2 describe-volumes --query 'Volumes[].VolumeId' --region "$REGION" --output text)
# Loop through each volume ID and delete it
for volume_id in $volume_ids; do
  echo "Listing volume: $volume_name ($volume_id)"
  volume_name=$(aws ec2 describe-tags --filters "Name=resource-id,Values=$volume_id" "Name=key,Values=Name" --region "$REGION" --output=text | cut -f5)
  if [[ $volume_name == *"packer"* ]] || [[ $volume_name == *"gh-ci"* ]]; then
    echo "--> Deleting volume: $volume_name ($volume_id)"
    aws ec2 delete-volume --volume-id "$volume_id" --region "$REGION"
  fi
done

# List all security group IDs and extract their IDs
security_group_ids=$(aws ec2 describe-security-groups --filters Name=group-name,Values=gh-ci-*,packer_* --query 'SecurityGroups[].GroupId' --region "$REGION" --output text)
# Loop through each security group ID and delete it
for security_group_id in $security_group_ids; do
  echo "--> Deleting security group: $security_group_id"
  aws ec2 delete-security-group --group-id "$security_group_id" --region "$REGION"
done

# Step 1: List all AMIs with names starting with "temp-" and get their ImageId
temp_ami_ids=$(aws ec2 describe-images --owners self --filters Name=name,Values=temp-* --query 'Images[*].ImageId' --region "$REGION" --output text)
for image_id in $temp_ami_ids; do
  echo "Listing image: $image_id"
  aws ec2 deregister-image --image-id "$image_id"
done

# Step 2: Get the Snapshot IDs associated with these AMIs
snapshot_ids=""
for ami_id in $temp_ami_ids; do
  ami_snapshot_ids=$(aws ec2 describe-images --image-ids "$ami_id" --query 'Images[*].BlockDeviceMappings[*].Ebs.SnapshotId' --region "$REGION" --output text)
  snapshot_ids="$snapshot_ids $ami_snapshot_ids"
done

# Step 3: Delete the snapshots
for snapshot_id in $snapshot_ids; do
  echo "Delete $snapshot_id related the images starting with temp-"
  aws ec2 delete-snapshot --snapshot-id "$snapshot_id" --region "$REGION" || true
done
