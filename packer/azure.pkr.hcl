packer {
  required_plugins {
    azure = {
      version = "= 2.1.4"
      source  = "github.com/hashicorp/azure"
    }
    ansible = {
      version = "= 1.1.1"
      source  = "github.com/hashicorp/ansible"
    }
  }
}

source "azure-arm" "TEMPLATE_DISTRIBUTION" {
  ssh_username              = "packer"
  ssh_timeout               = "60m"
  ssh_clear_authorized_keys = true
  client_id                 = "TEMPLATE_CLIENT_ID"
  tenant_id                 = "TEMPLATE_TENANT_ID"
  subscription_id           = "TEMPLATE_SUBSCRIPTION_ID"
  client_secret             = "TEMPLATE_CLIENT_SECRET"
  build_resource_group_name = "TEMPLATE_RESOURCE_GROUP"
  os_type                   = "Linux"
  image_publisher           = "TEMPLATE_IMAGE_PUBLISHER"
  image_offer               = "TEMPLATE_IMAGE_OFFER"
  image_sku                 = "TEMPLATE_IMAGE_SKU"
  vm_size                   = "TEMPLATE_VM_SIZE"
  secure_boot_enabled       = true
  vtpm_enabled              = true
  security_type             = "ConfidentialVM"
  keep_os_disk              = TEMPLATE_KEEP_OS_DISK
  temp_os_disk_name         = "TEMPLATE_OS_DISK_NAME"

  # shared_image_gallery {
  #   subscription         = "TEMPLATE_SUBSCRIPTION_ID"
  #   resource_group       = "packer-snp"
  #   gallery_name         = "cosmian_packer"
  #   image_name           = "base-image-TEMPLATE_DISTRIBUTION-TEMPLATE_TECHNO"
  #   image_version        = "TEMPLATE_BASE_IMAGE_VERSION"
  # }

  shared_image_gallery_destination {
    subscription         = "TEMPLATE_SUBSCRIPTION_ID"
    resource_group       = "packer-snp"
    gallery_name         = "cosmian_packer"
    image_name           = "TEMPLATE_PRODUCT-TEMPLATE_DISTRIBUTION-TEMPLATE_TECHNO"
    image_version        = "TEMPLATE_AZURE_IMAGE_VERSION"
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
    extra_arguments = ["-e", "cosmian_vm_version=TEMPLATE_COSMIAN_VM_VERSION", "-e", "cosmian_kms_version=TEMPLATE_COSMIAN_KMS_VERSION", "-e", "cosmian_ai_runner_version=TEMPLATE_COSMIAN_AI_RUNNER_VERSION"]
  }
}
