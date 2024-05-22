source "amazon-ebssurrogate" "TEMPLATE_DISTRIBUTION" {
  source_ami              = "TEMPLATE_SOURCE_AMI"
  region                  = "eu-west-1"
  ssh_username            = "TEMPLATE_SSH_USERNAME"
  ami_name                = "TEMPLATE_IMAGE_NAME"
  instance_type           = "c6a.8xlarge"
  ssh_timeout             = "5m"
  ami_virtualization_type = "hvm"
  ena_support             = true
  tpm_support             = "TEMPLATE_SUPPORT"
  boot_mode               = "uefi"

  launch_block_device_mappings {
    volume_type           = "gp3"
    device_name           = "TEMPLATE_DEVICE_NAME"
    volume_size           = "TEMPLATE_VOLUME_SIZE"
    delete_on_termination = true
  }

  ami_root_device {
    source_device_name    = "TEMPLATE_DEVICE_NAME"
    device_name           = "TEMPLATE_DEVICE_NAME"
    volume_size           = "TEMPLATE_VOLUME_SIZE"
    volume_type           = "TEMPLATE_VOLUME_TYPE"
    delete_on_termination = true
  }
}

build {
  sources = ["sources.amazon-ebssurrogate.TEMPLATE_DISTRIBUTION"]

  provisioner "ansible" {
    playbook_file   = "../ansible/TEMPLATE_PRODUCT-packer-playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=TEMPLATE_COSMIAN_VM_VERSION", "-e", "cosmian_kms_version=TEMPLATE_COSMIAN_KMS_VERSION"]
  }
}
