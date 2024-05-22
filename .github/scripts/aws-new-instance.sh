#!/bin/bash

set -x

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"

WHO="$(whoami)"
NAME="$WHO-$TECHNO-$DISTRIB"

SSH_PUB_KEY=$(cat ~/.ssh/id_rsa.pub)

CI_INSTANCES=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=${NAME}" --query 'Reservations[].Instances[].[InstanceId]' --output text)
for instance in $CI_INSTANCES; do
  aws ec2 terminate-instances --instance-ids $instance
done
aws ec2 delete-security-group --group-name "$NAME-ansible-sg"

aws ec2 create-security-group --group-name "$NAME-ansible-sg" --description "Security group for ansible test"
aws ec2 authorize-security-group-ingress --group-name "$NAME-ansible-sg" --protocol tcp --port 22 --cidr 0.0.0.0/0
aws ec2 authorize-security-group-ingress --group-name "$NAME-ansible-sg" --protocol tcp --port 5555 --cidr 0.0.0.0/0

set -ex

if [ "$TECHNO" = "tdx" ]; then
  # Ubuntu TDX
else
  if [ "$DISTRIB" = "ubuntu" ]; then
    # Ubuntu SEV
    aws ec2 run-instances \
      --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$NAME}]" \
      --image-id ami-0f523b3f4402ef624 \
      --instance-type c6a.xlarge \
      --cpu-options AmdSevSnp=enabled \
      --key-name packer \
      --security-groups "$NAME-ansible-sg" \
      --user-data '#!/bin/bash
      mkdir -p /home/ec2-user/.ssh
      echo "$SSH_PUB_KEY" >> /home/ec2-user/.ssh/authorized_keys
      chmod 600 /home/ec2-user/.ssh/authorized_keys
      chown ec2-user:ec2-user /home/ec2-user/.ssh/authorized_keys'

    IP_ADDR=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=${NAME}" --query 'Reservations[*].instances[*].PublicIpAddress' --output text)
    echo $IP_ADDR
  else
    # Redhat SEV
  fi
fi
