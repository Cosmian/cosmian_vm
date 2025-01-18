#!/bin/bash

set -x

# Assign default values if parameters are not provided
TECHNO="${1:-sev}"
DISTRIB="${2:-ubuntu}"
WHO="$(whoami)"
DEFAULT_NAME="$WHO-$TECHNO-$DISTRIB"
NAME="${3:-$DEFAULT_NAME}"

az vm delete -g packer-snp -n "$NAME" --yes
az snapshot delete -g packer-snp --name "snapshot-$NAME"
az image delete -g packer-snp --name "$NAME-image"
az disk delete -g packer-snp --name "$NAME-OSDisk" --yes
az disk delete -g packer-snp --name "disk-$NAME" --yes
az network nsg delete --resource-group packer-snp -n "${NAME}-nsg"
az network nsg delete --resource-group packer-snp -n "${NAME}NSG"
az network nsg delete --resource-group packer-snp -n "${NAME}"
az network nic delete -g packer-snp --name "${NAME}"
az network public-ip delete -g packer-snp -n "${NAME}PublicIP"
az network public-ip delete -g packer-snp -n "${NAME}"
az network vnet delete -g packer-snp --name "${NAME}"
