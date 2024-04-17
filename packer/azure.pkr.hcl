source "azure-arm" "TEMPLATE_DISTRIBUTION" {
  client_id                 = "TEMPLATE_CLIENT_ID"
  tenant_id                 = "TEMPLATE_TENANT_ID"
  subscription_id           = "TEMPLATE_SUBSCRIPTION_ID"
  client_secret             = "TEMPLATE_CLIENT_SECRET"
  build_resource_group_name = "TEMPLATE_RESOURCE_GROUP"
  os_type                   = "Linux"
  image_publisher           = "Canonical"
  image_offer               = "TEMPLATE_IMAGE_OFFER"
  image_sku                 = "TEMPLATE_IMAGE_SKU"
  vm_size                   = "TEMPLATE_VM_SIZE"
  secure_boot_enabled       = true
  vtpm_enabled              = true
  security_type             = "ConfidentialVM"

  shared_image_gallery_destination {
    subscription         = "TEMPLATE_SUBSCRIPTION_ID"
    resource_group       = "packer-snp"
    gallery_name         = "cosmian_packer"
    image_name           = "TEMPLATE_PRODUCT-TEMPLATE_DISTRIBUTION"
    image_version        = "0.0.0"
    storage_account_type = "Standard_LRS"
    target_region {
      name = "westeurope"
    }
    confidential_vm_image_encryption_type = "EncryptedVMGuestStateOnlyWithPmk"
  }
}

build {
  sources = ["source.azure-arm.TEMPLATE_DISTRIBUTION"]

  provisioner "ansible" {
    playbook_file   = "../ansible/TEMPLATE_PRODUCT-packer-playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=TEMPLATE_COSMIAN_VM_VERSION", "-e", "cosmian_kms_version=TEMPLATE_COSMIAN_KMS_VERSION"]
  }
}