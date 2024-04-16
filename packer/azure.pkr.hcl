variable "image_version" {}
variable "client_id" {}
variable "tenant_id" {}
variable "subscription_id" {}
variable "client_secret" {}


source "azure-arm" "ubuntu" {
  client_id                 = var.client_id
  tenant_id                 = var.tenant_id
  subscription_id           = var.subscription_id
  client_secret             = var.client_secret
  build_resource_group_name = "packer-snp"
  os_type                   = "Linux"
  image_publisher           = "Canonical"
  image_offer               = "0001-com-ubuntu-confidential-vm-jammy"
  image_sku                 = "22_04-lts-cvm"
  vm_size                   = "Standard_DC2ads_v5"
  secure_boot_enabled       = true
  vtpm_enabled              = true
  security_type             = "ConfidentialVM"

  shared_image_gallery_destination {
    subscription         = var.subscription_id
    resource_group       = "packer-snp"
    gallery_name         = "cosmian_packer"
    image_name           = "cosmian_vm_ubuntu"
    image_version        = "1.0.0"
    storage_account_type = "Standard_LRS"
    target_region {
      name = "westeurope"
    }
    confidential_vm_image_encryption_type = "EncryptedVMGuestStateOnlyWithPmk"
  }
}

build {
  sources = ["source.azure-arm.ubuntu"]

  provisioner "ansible" {
    playbook_file = "../ansible/cosmian-vm-packer-playbook.yml"
    local_port    = 22
    use_proxy     = false
  }
}
