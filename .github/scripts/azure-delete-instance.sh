#!/bin/bash

set -x


# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"
WHO="$(whoami)"
DEFAULT_NAME="$WHO-$TECHNO-$DISTRIB"
NAME="${3:-$DEFAULT_NAME}"


az vm delete -g packer-snp -n "$NAME" --yes
az network public-ip delete -g packer-snp -n "${NAME}PublicIP"
az network nsg delete --resource-group packer-snp -n "${NAME}NSG"
