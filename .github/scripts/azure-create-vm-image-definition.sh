#!/bin/bash

set +e

#!/bin/bash

# Define the lists
products=("base-image" "cosmian-vm" "kms" "kms-fips" "ai-runner")
distribs=("rhel" "ubuntu")
technos=("sev" "tdx")

# Outer loop: iterates over the first list
for product in "${products[@]}"; do
  # Middle loop: iterates over the second list
  for distrib in "${distribs[@]}"; do
    # Inner loop: iterates over the third list
    for techno in "${technos[@]}"; do
      # Perform an operation using items from all three lists
      echo "Processing combination: $product, $distrib, $techno"
      name="${product}-${distrib}-${techno}"

      if [ "${distrib}" = "ubuntu" ]; then
        offer=0001-com-ubuntu-confidential-vm-jammy
        sku=22_04-lts-cvm
      else
        offer=rhel-cvm
        sku=9_3_cvm_sev_snp
      fi

      # Create the image definition
      az sig image-definition create \
        --resource-group packer-snp \
        --gallery-name cosmian_packer \
        --gallery-image-definition "$name" \
        --publisher "$name" \
        --offer $offer \
        --sku $sku \
        --os-type linux \
        --os-state Generalized \
        --features SecurityType=ConfidentialVm \
        --minimum-cpu-core 1 \
        --maximum-cpu-core 128 \
        --minimum-memory 1 \
        --maximum-memory 512
    done
  done
done
