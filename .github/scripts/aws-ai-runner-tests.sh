#!/bin/sh

set -ex

SNAPSHOT="aws_${PRODUCT}_${DISTRIB}_${TECHNO}.snapshot"
NEW_SNAPSHOT="new_$SNAPSHOT"

bash .github/scripts/aws-cosmian-vm-tests.sh

AMI=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=$CI_INSTANCE" --query 'Reservations[].Instances[].[InstanceId]' --output text)

IP_ADDR=$(aws ec2 describe-instances --instance-ids "$AMI" --query 'Reservations[*].Instances[*].PublicIpAddress' --output text)

echo "Cosmian VM app init"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app init -c ansible/roles/ai_runner/templates/config.json.j2

echo "Checking Cosmian AI Runner HTTPS connection..."
timeout 5m bash -c "until curl --fail --insecure https://${IP_ADDR}/health; do sleep 3; done"
echo ""
echo "[ OK ] Cosmian AI Runner HTTPS connection"

echo "Rebooting instance..."
aws ec2 reboot-instances --instance-ids "$AMI" --region "${ZONE}"
IP_ADDR=$(aws ec2 describe-instances --instance-ids "$AMI" --query 'Reservations[*].Instances[*].PublicIpAddress' --output text)
aws ec2 wait instance-running --instance-ids "$AMI"
timeout 8m bash -c "until curl --fail --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"
echo "IP_ADDR=${IP_ADDR}" >>"$GITHUB_OUTPUT"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(jq '.tpm_policy.reset_count' "$SNAPSHOT")
NEW_RESET_COUNT=$((RESET_COUNT + 2))
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' "$SNAPSHOT" >"$NEW_SNAPSHOT"
jq '.tpm_policy.reset_count |= tonumber' "$NEW_SNAPSHOT" | sponge "$NEW_SNAPSHOT"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot "$NEW_SNAPSHOT"
echo "[ OK ] Integrity after reboot"

echo "Starting the AI Runner"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app restart

echo "Checking Cosmian AI Runner HTTPS connection..."
timeout 5m bash -c "until curl --fail --insecure https://${IP_ADDR}/health; do sleep 3; done"
echo ""
echo "[ OK ] Cosmian AI Runner HTTPS connection"
