#!/bin/bash

set +x

RESOURCE_GROUP="packer-snp"

# List all VMs and extract VM names
vm_names=$(az vm list -g "$RESOURCE_GROUP" --query "[].name" -o tsv)
# Loop through each VM name and delete it
for vm_name in $vm_names; do
  if [[ $vm_name = *'pkr'* ]] || [[ $vm_name = *'gh-ci'* ]]; then
    echo "Deleting VM: $vm_name"
    az vm delete -g "$RESOURCE_GROUP" -n "$vm_name" --yes
    az network public-ip delete -g "$RESOURCE_GROUP" -n "${vm_name}PublicIP"
    az network nsg delete -g "$RESOURCE_GROUP" -n "${vm_name}NSG"
  fi
done

# List all network interfaces and extract their names
nic_names=$(az network nic list --query "[].name" -o tsv)
# Loop through each network interface name and delete it
for nic_name in $nic_names; do
  if [[ $nic_name = *'pkr'* ]] || [[ $nic_name = *'gh-ci'* ]]; then
    echo "Deleting Network Interface: $nic_name"
    az network nic delete -g "$RESOURCE_GROUP" --name "$nic_name"
  fi
done

# List all public IPs and extract their names
public_ip_names=$(az network public-ip list --query "[].name" -o tsv)
# Loop through each public IP name and delete it
for public_ip_name in $public_ip_names; do
  if [[ $public_ip_name = *'pkr'* ]] || [[ $public_ip_name = *'gh-ci'* ]]; then
    echo "Deleting Public IP: $public_ip_name"
    az network public-ip delete -g "$RESOURCE_GROUP" --name "$public_ip_name"
  fi
done

# List all virtual networks and extract their names
vnet_names=$(az network vnet list --query "[].name" -o tsv)
# Loop through each virtual network name and delete it
for vnet_name in $vnet_names; do
  if [[ $vnet_name = *'pkr'* ]] || [[ $vnet_name = *'gh-ci'* ]]; then
    echo "Deleting Virtual Network: $vnet_name"
    az network vnet delete -g "$RESOURCE_GROUP" --name "$vnet_name"
  fi
done

# List all disks and extract their names
disk_names=$(az disk list --query "[].name" -o tsv)
# Loop through each disk name and delete it
for disk_name in $disk_names; do
  if [[ $disk_name = *'pkr'* ]] || [[ $disk_name = *'gh-ci'* ]]; then
    echo "Deleting Disk: $disk_name"
    az disk delete -g "$RESOURCE_GROUP" --name "$disk_name" --yes
  fi
done

# List all network security groups and extract their names
nsg_names=$(az network nsg list --query "[].name" -o tsv)
# Loop through each network security group name and delete it
for nsg_name in $nsg_names; do
  if [[ $nsg_name = *'pkr'* ]] || [[ $nsg_name = *'gh-ci'* ]]; then
    echo "Deleting Network Security Group: $nsg_name"
    az network nsg delete -g "$RESOURCE_GROUP" --name "$nsg_name"
  fi
done
