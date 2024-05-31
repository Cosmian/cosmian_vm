#!/bin/sh

rm -f ./*.snapshot

set -ex

SNAPSHOT="aws_${PRODUCT}_${DISTRIB}_${TECHNO}.snapshot"
NEW_SNAPSHOT="new_$SNAPSHOT"

sudo apt-get install -y jq moreutils

echo "Waiting for Cosmian VM agent (${IP_ADDR}:5555)..."
timeout 20m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls snapshot --output "$SNAPSHOT"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot "$SNAPSHOT"

CI_INSTANCE_ID=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=${CI_INSTANCE}" --query 'Reservations[].Instances[].[InstanceId]' --output text)

echo "Rebooting instance..."
aws ec2 reboot-instances --instance-ids "$CI_INSTANCE_ID" # do not specify region. Should reuse Region and Placement from `run-instances` command
aws ec2 wait instance-running --instance-ids "$CI_INSTANCE_ID"

IP_ADDR=$(aws ec2 describe-instances --instance-ids "$CI_INSTANCE_ID" --query 'Reservations[].Instances[].PublicIpAddress' --output text)
echo "IP_ADDR=${IP_ADDR}" >>"$GITHUB_OUTPUT"

timeout 20m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(jq '.tpm_policy.reset_count' "$SNAPSHOT")
NEW_RESET_COUNT=$((RESET_COUNT + 1))
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' "$SNAPSHOT" >"$NEW_SNAPSHOT"
jq '.tpm_policy.reset_count |= tonumber' "$NEW_SNAPSHOT" | sponge "$NEW_SNAPSHOT"

./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot "$NEW_SNAPSHOT"
echo "[ OK ] Integrity after reboot"

rm -f "$NEW_SNAPSHOT"
