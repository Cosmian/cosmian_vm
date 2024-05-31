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
  aws ec2 terminate-instances --instance-ids "$instance"
  aws ec2 wait instance-terminated --instance-ids "$instance"
done
aws ec2 delete-security-group --group-name "$NAME-ansible-sg" --output text

aws ec2 create-security-group --group-name "$NAME-ansible-sg" --description "Security group for ansible test"
aws ec2 authorize-security-group-ingress --group-name "$NAME-ansible-sg" --protocol tcp --port 22 --cidr 0.0.0.0/0
aws ec2 authorize-security-group-ingress --group-name "$NAME-ansible-sg" --protocol tcp --port 5555 --cidr 0.0.0.0/0
aws ec2 authorize-security-group-ingress --group-name "$NAME-ansible-sg" --protocol tcp --port 443 --cidr 0.0.0.0/0

set -ex

if [ "$TECHNO" = "tdx" ]; then
  # Ubuntu TDX
  true
else
  if [ "$DISTRIB" = "ubuntu" ]; then
    # Ubuntu SEV
    AMI_BASE=$(aws ec2 describe-images --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-20240523.1" --query "Images[*].{ID:ImageId}" --output text)
    AMI=$(aws ec2 run-instances \
      --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$NAME}]" \
      --image-id "$AMI_BASE" \
      --instance-type c6a.2xlarge \
      --block-device-mappings "DeviceName=/dev/sda1,Ebs={VolumeType=gp3,VolumeSize=20}" \
      --cpu-options AmdSevSnp=enabled \
      --key-name packer \
      --security-groups "$NAME-ansible-sg" \
      --query 'Instances[0].InstanceId' --output text \
      --metadata-options "InstanceMetadataTags=enabled, HttpTokens=optional, HttpEndpoint=enabled, HttpPutResponseHopLimit=2" \
      --user-data "#!/bin/bash
      mkdir -p /home/ubuntu/.ssh
      echo $SSH_PUB_KEY >> /home/ubuntu/.ssh/authorized_keys
      chmod 600 /home/ubuntu/.ssh/authorized_keys
      chown ubuntu:ubuntu /home/ubuntu/.ssh/authorized_keys")

    aws ec2 wait instance-running --instance-ids "$AMI"
    IP_ADDR=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=${NAME}" --query 'Reservations[*].instances[*].PublicIpAddress' --output text)
    echo "$IP_ADDR"
  else
    # Redhat SEV
    true
  fi
fi
