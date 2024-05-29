#!/bin/sh

rm -f ./*.snapshot

set -ex

SNAPSHOT="azure_${PRODUCT}_${DISTRIB}_${TECHNO}.snapshot"
NEW_SNAPSHOT="new_$SNAPSHOT"

sudo apt-get install -y jq moreutils

echo "Waiting for Cosmian VM agent (${IP_ADDR}:5555)..."
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls snapshot --output "$SNAPSHOT"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot "$SNAPSHOT"

echo "Rebooting instance..."
az vm restart -g "$RESOURCE_GROUP" -n "$CI_INSTANCE"

IP_ADDR=$(az vm show -d -g "$RESOURCE_GROUP" -n "$CI_INSTANCE" --query publicIps -o tsv)

timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(jq '.tpm_policy.reset_count' "$SNAPSHOT")
NEW_RESET_COUNT=$((RESET_COUNT + 1))
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' "$SNAPSHOT" >"$NEW_SNAPSHOT"
jq '.tpm_policy.reset_count |= tonumber' "$NEW_SNAPSHOT" | sponge "$NEW_SNAPSHOT"

./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot "$NEW_SNAPSHOT"
echo "[ OK ] Integrity after reboot"

rm -f "$NEW_SNAPSHOT"
