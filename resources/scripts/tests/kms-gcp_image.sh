#!/bin/sh

set -exu

MODE=$1
CI_INSTANCE=$2
ZONE=$3
IP_ADDR=$4
GCP_DEV_PROJECT=cosmian-dev

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

echo "Rebooting instance..."
gcloud beta compute instances stop "$CI_INSTANCE" --zone "$ZONE" --project "$GCP_DEV_PROJECT"
gcloud beta compute instances start "$CI_INSTANCE" --zone "$ZONE" --project "$GCP_DEV_PROJECT"
IP_ADDR=$(gcloud beta compute instances describe "$CI_INSTANCE" --format='get(networkInterfaces[0].accessConfigs[0].natIP)' --zone="$ZONE")
timeout 4m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5355/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(cat cosmian_vm.snapshot | jq '.tpm_policy.reset_count')
NEW_RESET_COUNT=$(expr $RESET_COUNT + 1)
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' cosmian_vm.snapshot >new_cosmian_vm.snapshot
jq '.tpm_policy.reset_count |= tonumber' new_cosmian_vm.snapshot | sponge new_cosmian_vm.snapshot
./cosmian_vm --url "https://${IP_ADDR}:5355" --allow-insecure-tls verify --snapshot new_cosmian_vm.snapshot
echo "[ OK ] Integrity after reboot"

echo "Starting the KMS"
./cosmian_vm --url "https://${IP_ADDR}:5355" --allow-insecure-tls app restart

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