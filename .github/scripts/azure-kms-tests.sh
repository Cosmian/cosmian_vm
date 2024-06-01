#!/bin/sh

set -ex

SNAPSHOT="azure_${PRODUCT}_${DISTRIB}_${TECHNO}.snapshot"
NEW_SNAPSHOT="new_$SNAPSHOT"

test_opened_ports() {
  REMOTE_HOST=$1
  echo "Checking Cosmian KMS HTTPS connection..."
  timeout 5m bash -c "until curl --fail --insecure https://${REMOTE_HOST}/version; do sleep 3; done"
  echo ""
  echo "[ OK ] Cosmian KMS HTTPS connection"
}

bash .github/scripts/azure-cosmian-vm-tests.sh
IP_ADDR=$(az vm show -d -g "$RESOURCE_GROUP" -n "$CI_INSTANCE" --query publicIps -o tsv)

echo "Cosmian VM app init"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app init -c ansible/roles/kms/templates/kms.toml.j2

test_opened_ports "$IP_ADDR"

echo "Rebooting instance..."
az vm restart -g "$RESOURCE_GROUP" -n "$CI_INSTANCE"
IP_ADDR=$(az vm show -d -g "$RESOURCE_GROUP" -n "$CI_INSTANCE" --query publicIps -o tsv)
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 10; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(jq '.tpm_policy.reset_count' "$SNAPSHOT")
NEW_RESET_COUNT=$((RESET_COUNT + 2))
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' "$SNAPSHOT" >"$NEW_SNAPSHOT"
jq '.tpm_policy.reset_count |= tonumber' "$NEW_SNAPSHOT" | sponge "$NEW_SNAPSHOT"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot "$NEW_SNAPSHOT"
echo "[ OK ] Integrity after reboot"

echo "Starting the KMS"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app restart

test_opened_ports "$IP_ADDR"
