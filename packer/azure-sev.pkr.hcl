variable "prefix" {}

locals {
  ubuntu_managed_image_name = "${var.prefix}-cosmian-vm-ubuntu-{{timestamp}}"
  ubuntu_managed_image_resource_group_name = "packer"
  ubuntu_build_resource_group_name = "packer"
  os_type = "Linux"
  image_publisher = "Canonical"
  image_offer = "0001-com-ubuntu-server-jammy"
  image_sku = "22_04-lts-gen2"
  location = "East US"
  vm_size = "Standard_B1s"
}

source "azure-arm" "ubuntu" {
  build_resource_group_name = local.ubuntu_build_resource_group_name
  managed_image_resource_group_name = local.ubuntu_managed_image_resource_group_name
  managed_image_name          = local.ubuntu_managed_image_name
  os_type                     = local.os_type
  image_publisher             = local.image_publisher
  image_offer                 = local.image_offer
  image_sku                   = local.image_sku
  vm_size                     = local.vm_size
  use_azure_cli_auth          = false
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

