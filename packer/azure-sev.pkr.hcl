variable "prefix" {}
variable "client_id" {}
variable "tenant_id" {}
variable "subscription_id" {}
variable "client_secret" {}

locals {
  client_id = "${var.client_id}"
  client_secret = "${var.client_secret}"
  subscription_id = "${var.subscription_id}"
  tenant_id = "${var.tenant_id}"
  ubuntu_managed_image_name = "${var.prefix}-cosmian-vm-ubuntu-{{timestamp}}"
  ubuntu_managed_image_resource_group_name = "packer_tdx"
  ubuntu_build_resource_group_name = "packer_tdx"
  os_type = "Linux"
  image_publisher = "Canonical"
  image_offer = "0001-com-ubuntu-confidential-vm-jammy"
  image_sku = "22_04-lts-cvm"
  vm_size = "Standard_DC2ads_v5"
  vtpm_enabled = true
  secure_boot_enabled = true
}

source "azure-arm" "ubuntu" {
  client_id                 = local.client_id
  tenant_id                 = local.tenant_id
  subscription_id           = local.subscription_id
  client_secret             = local.client_secret
  managed_image_resource_group_name = local.ubuntu_managed_image_resource_group_name
  build_resource_group_name   = local.ubuntu_build_resource_group_name
  managed_image_name          = local.ubuntu_managed_image_name
  os_type                     = local.os_type
  image_publisher             = local.image_publisher
  image_offer                 = local.image_offer
  image_sku                   = local.image_sku
  vm_size                     = local.vm_size
  secure_boot_enabled         = local.secure_boot_enabled
  vtpm_enabled                = local.vtpm_enabled
  shared_image_gallery_destination {
    subscription = local.subscription_id
    resource_group = "packer_tdx"
    gallery_name = "cosmian"
    image_name = "cosmian-vm-ubuntu"
    image_version = "0.0.1"
    storage_account_type = "Standard_LRS"
  }
}

build {
  sources = ["source.azure-arm.ubuntu"]

  provisioner "file" {
    source      = "../resources/conf/ima-policy"
    destination = "/tmp/ima-policy"
  }

  provisioner "file" {
    source      = "../resources/conf/agent_no_tpm.toml"
    destination = "/tmp/agent.toml"
  }

  provisioner "file" {
    source      = "../resources/scripts/cosmian_fstool"
    destination = "/tmp/cosmian_fstool"
  }

  provisioner "file" {
    source      = "./target/release/cosmian_vm_agent"
    destination = "/tmp/"
  }

  provisioner "file" {
    source      = "./target/release/cosmian_certtool"
    destination = "/tmp/"
  }

  provisioner "ansible" {
    playbook_file = "../ansible/cosmian_vm_playbook.yml"
    local_port    = 22
    use_proxy     = false
  }
}

