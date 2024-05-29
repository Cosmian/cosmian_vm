#!/bin/sh

set -ex

SNAPSHOT="azure_${PRODUCT}_${DISTRIB}_${TECHNO}.snapshot"
NEW_SNAPSHOT="new_$SNAPSHOT"

echo "Waiting for Cosmian VM agent (${IP_ADDR}:5555)..."
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

echo "Cosmian VM app init"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app init -c ansible/roles/ai_runner/templates/config.json.j2

bash .github/scripts/azure-cosmian-vm-tests.sh
IP_ADDR=$(az vm show -d -g "$RESOURCE_GROUP" -n "$CI_INSTANCE" --query publicIps -o tsv)

echo "Checking Cosmian AI Runner HTTPS connection..."
timeout 5m bash -c "until curl --fail --insecure https://${IP_ADDR}/health; do sleep 3; done"
echo ""
echo "[ OK ] Cosmian AI Runner HTTPS connection"

echo "Rebooting instance..."
az vm restart -g "$RESOURCE_GROUP" -n "$CI_INSTANCE"
IP_ADDR=$(az vm show -d -g "$RESOURCE_GROUP" -n "$CI_INSTANCE" --query publicIps -o tsv)
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"

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
