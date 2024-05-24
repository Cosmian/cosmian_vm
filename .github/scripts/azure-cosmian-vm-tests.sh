#!/bin/sh

rm -f cosmian_vm.snapshot

set -ex

CI_INSTANCE=$1
IP_ADDR=$2

sudo apt-get install -y jq moreutils

echo "Waiting for Cosmian VM agent (${IP_ADDR}:5555)..."
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls snapshot
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot cosmian_vm.snapshot

echo "Rebooting instance..."
az vm restart -g "$RESOURCE_GROUP" -n "$CI_INSTANCE"

IP_ADDR=$(az vm show -d -g "$RESOURCE_GROUP" -n "$CI_INSTANCE" --query publicIps -o tsv)

timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(jq '.tpm_policy.reset_count' cosmian_vm.snapshot)
NEW_RESET_COUNT=$((RESET_COUNT + 1))
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' cosmian_vm.snapshot >new_cosmian_vm.snapshot
jq '.tpm_policy.reset_count |= tonumber' new_cosmian_vm.snapshot | sponge new_cosmian_vm.snapshot

./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot new_cosmian_vm.snapshot
echo "[ OK ] Integrity after reboot"
