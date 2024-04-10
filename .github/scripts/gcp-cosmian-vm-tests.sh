#!/bin/sh

set -exu

MODE=$1
CI_INSTANCE=$2
ZONE=$3
IP_ADDR=$4

sudo apt-get install -y jq moreutils

echo "Waiting for Cosmian VM agent (${IP_ADDR}:5355)..."
timeout 4m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5355/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready"
./cosmian_vm --url "https://${IP_ADDR}:5355" --allow-insecure-tls snapshot
./cosmian_vm --url "https://${IP_ADDR}:5355" --allow-insecure-tls verify --snapshot cosmian_vm.snapshot

echo "Rebooting instance..."
gcloud "${MODE}" compute instances stop "$CI_INSTANCE" --zone "${ZONE}" --project "$GCP_DEV_PROJECT"

gcloud "${MODE}" compute instances set-scheduling "$CI_INSTANCE" --zone "${ZONE}" --max-run-duration=10m --instance-termination-action=DELETE

sleep 30

gcloud "${MODE}" compute instances start "$CI_INSTANCE" --zone "${ZONE}" --project "$GCP_DEV_PROJECT"

IP_ADDR=$(gcloud "${MODE}" compute instances describe "$CI_INSTANCE" --format='get(networkInterfaces[0].accessConfigs[0].natIP)' --zone="${ZONE}")

timeout 4m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5355/ima/ascii; do sleep 3; done"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(cat cosmian_vm.snapshot | jq '.tpm_policy.reset_count')
NEW_RESET_COUNT=$(expr $RESET_COUNT + 1)
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' cosmian_vm.snapshot >new_cosmian_vm.snapshot
jq '.tpm_policy.reset_count |= tonumber' new_cosmian_vm.snapshot | sponge new_cosmian_vm.snapshot

./cosmian_vm --url "https://${IP_ADDR}:5355" --allow-insecure-tls verify --snapshot new_cosmian_vm.snapshot
echo "[ OK ] Integrity after reboot"