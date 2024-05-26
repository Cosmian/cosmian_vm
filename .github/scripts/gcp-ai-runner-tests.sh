#!/bin/sh

set -exu

MODE=$1
CI_INSTANCE=$2
ZONE=$3
IP_ADDR=$4

bash .github/scripts/gcp-cosmian-vm-tests.sh "$MODE" "$CI_INSTANCE" "$ZONE" "$IP_ADDR"

IP_ADDR=$(gcloud "${MODE}" compute instances describe "$CI_INSTANCE" --format='get(networkInterfaces[0].accessConfigs[0].natIP)' --zone="${ZONE}")

echo "Cosmian VM app init"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app init -c ansible/roles/ai_runner/templates/agent.toml.j2

echo "Checking Cosmian AI Runner HTTP connection..."
timeout 5m bash -c "until curl --fail http://${IP_ADDR}:5001/health; do sleep 3; done"
echo ""

echo "[ OK ] Cosmian AI Runner HTTP connection"
echo "Checking Cosmian AI Runner HTTPS connection..."
curl --insecure "https://${IP_ADDR}/health"
echo ""
echo "[ OK ] Cosmian AI Runner HTTPS connection"
echo "Checking Cosmian AI Runner HTTP to HTTPS redirect connection..."
curl --insecure "http://${IP_ADDR}/health"
echo ""
echo "[ OK ] Cosmian AI Runner HTTP to HTTPS redirect connection"

echo "Rebooting instance..."
gcloud "${MODE}" compute instances stop "$CI_INSTANCE" --zone "$ZONE" --project "$GCP_DEV_PROJECT"
gcloud "${MODE}" compute instances set-scheduling "$CI_INSTANCE" --zone "${ZONE}" --max-run-duration=20m --instance-termination-action=DELETE
sleep 30
gcloud "${MODE}" compute instances start "$CI_INSTANCE" --zone "$ZONE" --project "$GCP_DEV_PROJECT"
IP_ADDR=$(gcloud "${MODE}" compute instances describe "$CI_INSTANCE" --format='get(networkInterfaces[0].accessConfigs[0].natIP)' --zone="${ZONE}")
timeout 8m bash -c "until curl --insecure --output /dev/null --silent --fail https://${IP_ADDR}:5555/ima/ascii; do sleep 3; done"
echo "IP_ADDR=${IP_ADDR}" >>"$GITHUB_OUTPUT"

echo "[ OK ] Cosmian VM ready after reboot"
RESET_COUNT=$(jq '.tpm_policy.reset_count' cosmian_vm.snapshot)
NEW_RESET_COUNT=$((RESET_COUNT + 2))
jq --arg NEW_RESET_COUNT "$NEW_RESET_COUNT" '.tpm_policy.reset_count = $NEW_RESET_COUNT' cosmian_vm.snapshot >new_cosmian_vm.snapshot
jq '.tpm_policy.reset_count |= tonumber' new_cosmian_vm.snapshot | sponge new_cosmian_vm.snapshot
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls verify --snapshot new_cosmian_vm.snapshot
echo "[ OK ] Integrity after reboot"

echo "Starting the AI Runner"
./cosmian_vm --url "https://${IP_ADDR}:5555" --allow-insecure-tls app restart

echo "[ OK ] AI Runner is started"
echo "Checking Cosmian AI Runner HTTP connection..."
timeout 5m bash -c "until curl --fail http://${IP_ADDR}:5001/health; do sleep 3; done"
echo ""

echo "[ OK ] Cosmian AI Runner HTTP connection"
echo "Checking Cosmian AI Runner HTTPS connection..."
curl --insecure "https://${IP_ADDR}/health"
echo ""
echo "[ OK ] Cosmian AI Runner HTTPS connection"
echo "Checking Cosmian AI Runner HTTP to HTTPS redirect connection..."
curl --insecure "http://${IP_ADDR}/health"
echo ""
echo "[ OK ] Cosmian AI Runner HTTP to HTTPS redirect connection"
