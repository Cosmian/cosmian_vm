packer {
  required_plugins {
    amazon = {
      version = "= 1.3.9"
      source  = "github.com/hashicorp/amazon"
    }
    ansible = {
      version = "= 1.1.4"
      source  = "github.com/hashicorp/ansible"
    }
  }
}

source "amazon-ebssurrogate" "TEMPLATE_DISTRIBUTION" {
  ssh_username              = "TEMPLATE_SSH_USERNAME"
  ssh_timeout               = "60m"
  ssh_clear_authorized_keys = true
  source_ami                = "TEMPLATE_SOURCE_AMI"
  region                    = "eu-west-1"
  ami_name                  = "TEMPLATE_IMAGE_NAME"
  instance_type             = "c6a.2xlarge"
  ami_virtualization_type   = "hvm"
  ena_support               = true
  tpm_support               = "TEMPLATE_SUPPORT"
  boot_mode                 = "uefi"

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
    extra_arguments = ["-e", "cosmian_vm_version=TEMPLATE_COSMIAN_VM_VERSION", "-e", "cosmian_kms_version=TEMPLATE_COSMIAN_KMS_VERSION", "-e", "cosmian_ai_runner_version=TEMPLATE_COSMIAN_AI_RUNNER_VERSION"]
  }
}
