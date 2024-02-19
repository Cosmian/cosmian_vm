#!/usr/bin/bash

set -ex

sudo killall cosmian_vm_agent || true

CUR_DIR=$(pwd)
TMP_DIR="$(mktemp -d)"
RAND_PORT=$((5100 + RANDOM % 1000))
RAND_NAME=$(echo date +%s%N | sha256sum | head -c 20)

# Prerequisites: folder cosmian_vm should contain:
# - cosmian_vm_agent
# - cosmian_vm
# - cosmian_certtool
mkdir -p cosmian_vm
cp target/release/cosmian_vm target/release/cosmian_vm_agent cosmian_vm/
chmod u+x cosmian_vm/*

# Create template directory
mkdir -p "$TMP_DIR"
cp -r cosmian_vm "$TMP_DIR"

# Copy agent configuration template
cp resources/conf/agent.toml "$TMP_DIR"

# Set working directory
cd "$TMP_DIR"

###
# Customize Cosmian VM agent configuration
sed -i "s,5355,$RAND_PORT," agent.toml

###
# Run Cosmian VM agent
sudo chmod u+x "$CUR_DIR/resources/scripts/cosmian_fstool"
sudo COSMIAN_VM_FSTOOL="$CUR_DIR/resources/scripts/cosmian_fstool" COSMIAN_VM_AGENT_CONF="$TMP_DIR/agent.toml" ./cosmian_vm/cosmian_vm_agent &

# wait for the server to be started
sleep 3

###
# Run Cosmian VM cli
./cosmian_vm/cosmian_vm --url https://localhost:$RAND_PORT/ --allow-insecure-tls snapshot
./cosmian_vm/cosmian_vm --url https://localhost:$RAND_PORT/ --allow-insecure-tls verify --snapshot ./cosmian_vm.snapshot

###
# Run a fake malware!
echo -e "#!/usr/bin/bash\necho malware" >"$RAND_NAME.sh"
chmod +x "$RAND_NAME.sh"
./"$RAND_NAME.sh"

set +e
./cosmian_vm/cosmian_vm --url https://localhost:$RAND_PORT/ --allow-insecure-tls verify --snapshot ./cosmian_vm.snapshot
ret=$?
if [ $ret -eq 0 ]; then
  echo "MUST fail since new executable file has been run"
  exit 1
fi
