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
  ubuntu_build_resource_group_name = "packer-snp"
  os_type = "Linux"
  image_publisher = "Canonical"
  image_offer = "0001-com-ubuntu-confidential-vm-jammy"
  image_sku = "22_04-lts-cvm"
  vm_size = "Standard_DC2ads_v5"
  vtpm_enabled = true
  secure_boot_enabled = true
  security_type = "ConfidentialVM"
}

source "azure-arm" "ubuntu" {
  client_id                 = local.client_id
  tenant_id                 = local.tenant_id
  subscription_id           = local.subscription_id
  client_secret             = local.client_secret
  build_resource_group_name   = local.ubuntu_build_resource_group_name
  os_type                     = local.os_type
  image_publisher             = local.image_publisher
  image_offer                 = local.image_offer
  image_sku                   = local.image_sku
  vm_size                     = local.vm_size
  secure_boot_enabled         = local.secure_boot_enabled
  vtpm_enabled                = local.vtpm_enabled
  security_type               = local.security_type
  shared_image_gallery_destination {
    subscription = "e04f52be-d51f-43fe-95f8-d63a8fc91464"
    resource_group = "packer-snp"
    gallery_name = "cosmian_packer"
    image_name = "cosmian_vm_ubuntu"
    image_version = "1.0.0"
    storage_account_type = "Standard_LRS"
    target_region {
      name = "westeurope"
    }
  }
}

build {
  sources = ["source.azure-arm.ubuntu"]

  provisioner "ansible" {
    playbook_file = "../ansible/packer_sev_playbook.yml"
    local_port    = 22
    use_proxy     = false
  }
}