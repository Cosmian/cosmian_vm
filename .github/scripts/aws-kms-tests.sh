#!/bin/sh

set -exu

CI_INSTANCE=$1
IP_ADDR=$2
ZONE=$3

bash .github/scripts/aws-cosmian-vm-tests.sh "$CI_INSTANCE" "$IP_ADDR" "$ZONE"

AMI=$(aws ec2 describe-instances --filters "Name=tag:Name,Values=$CI_INSTANCE" --query 'Reservations[].Instances[].[InstanceId]' --output text)

IP_ADDR=$(aws ec2 describe-instances --instance-ids "$AMI" --query 'Reservations[*].Instances[*].PublicIpAddress' --output text)

echo "Cosmian VM app init"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app init -c ansible/roles/start_kms/templates/kms.toml.j2

echo "Checking Cosmian KMS HTTP connection..."
timeout 1m bash -c "until curl http://${IP_ADDR}:8080/version; do sleep 3; done"
echo ""

echo "[ OK ] Cosmian KMS HTTP connection"
echo "Checking Cosmian KMS HTTPS connection..."
curl --insecure "https://${IP_ADDR}/version"
echo ""
echo "[ OK ] Cosmian KMS HTTPS connection"
echo "Checking Cosmian KMS HTTP to HTTPS redirect connection..."
curl --insecure "http://${IP_ADDR}/version"
echo ""
echo "[ OK ] Cosmian KMS HTTP to HTTPS redirect connection"

echo "Rebooting instance..."
aws ec2 reboot-instances --instance-ids "$AMI" --region "${ZONE}"
IP_ADDR=$(aws ec2 describe-instances --instance-ids "$AMI" --query 'Reservations[*].Instances[*].PublicIpAddress' --output text)
aws ec2 wait instance-running --instance-ids "$AMI"
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"
echo "IP_ADDR=${IP_ADDR}" >> "$GITHUB_OUTPUT"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(cat cosmian_vm.snapshot | jq '.tpm_policy.reset_count')
NEW_RESET_COUNT=$(expr $RESET_COUNT + 2)
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' cosmian_vm.snapshot >new_cosmian_vm.snapshot
jq '.tpm_policy.reset_count |= tonumber' new_cosmian_vm.snapshot | sponge new_cosmian_vm.snapshot
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot new_cosmian_vm.snapshot
echo "[ OK ] Integrity after reboot"

echo "Starting the KMS"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app restart

# Wait KMS to be started
sleep 30

echo "[ OK ] KMS is started"
echo "Checking Cosmian KMS HTTP connection..."
curl "http://${IP_ADDR}:8080/version"
echo ""

echo "[ OK ] Cosmian KMS HTTP connection"
echo "Checking Cosmian KMS HTTPS connection..."
curl --insecure "https://${IP_ADDR}/version"
echo ""
echo "[ OK ] Cosmian KMS HTTPS connection"
echo "Checking Cosmian KMS HTTP to HTTPS redirect connection..."
curl --insecure "http://${IP_ADDR}/version"
echo ""
echo "[ OK ] Cosmian KMS HTTP to HTTPS redirect connection"
