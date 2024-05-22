#!/bin/sh

set -ex

CI_INSTANCE=$1
IP_ADDR=$2
ZONE=$3

sudo apt-get install -y jq moreutils

echo "Waiting for Cosmian VM agent (${IP_ADDR}:5555)..."
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls snapshot
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot cosmian_vm.snapshot

CI_INSTANCE_ID=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=${CI_INSTANCE}" --query Reservations[].Instances[].[InstanceId]' --output text)

echo "Rebooting instance..."
aws ec2 reboot-instances --instance-ids "$CI_INSTANCE_ID" --region "${ZONE}"

sleep 30
timeout 10m bash -c "until aws ec2 describe-instance-status --instance-ids $CI_INSTANCE_ID --query 'InstanceStatuses[].InstanceStatus[].Status[]' --output text | grep -q ok; do sleep 60; done"

IP_ADDR=$(aws ec2 describe-instances --instance-ids "$CI_INSTANCE_ID" --query 'Reservations[].Instances[].PublicIpAddress' --output text)
echo "IP_ADDR=${IP_ADDR}" >>"$GITHUB_OUTPUT"

timeout 15m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(cat cosmian_vm.snapshot | jq '.tpm_policy.reset_count')
NEW_RESET_COUNT=$(expr $RESET_COUNT + 1)
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' cosmian_vm.snapshot >new_cosmian_vm.snapshot
jq '.tpm_policy.reset_count |= tonumber' new_cosmian_vm.snapshot | sponge new_cosmian_vm.snapshot

./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot new_cosmian_vm.snapshot
echo "[ OK ] Integrity after reboot"
