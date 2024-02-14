variable "prefix" {}

locals {
 amazon_linux_ami_name = "${var.prefix}-cosmian-vm-amazon-linux-{{timestamp}}"
  redhat_ami_name = "${var.prefix}-cosmian-vm-redhat-{{timestamp}}"
}

variable "amazon_linux_source_ami" {
  type    = string
  default = "ami-0766b4b472db7e3b9"
}
variable "redhat_source_ami" {
  type    = string
  default = "ami-08e592fbb0f535224"
}

variable "region" {
  type    = string
  default = "eu-west-1"
}

variable "redhat_ssh_username" {
  type    = string
  default = "ec2-user"
}

variable "amazon_linux_ssh_username" {
  type    = string
  default = "ec2-user"
}

variable "ssh_timeout" {
  type    = string
  default = "20m"
}

variable "instance_type" {
  type    = string
  default = "c6a.large"
}

variable "ami_virtualization_type" {
  type    = string
  default = "hvm"
}

variable "ena_support" {
  type    = bool
  default = true
}

variable "volume_type" {
  type    = string
  default = "gp3"
}

variable "amazon_linux_launch_block_device_mappings_device_name" {
  type    = string
  default = "/dev/xvda"
}

variable "amazon_linux_source_device_name" {
  type    = string
  default = "/dev/xvda"
}

variable "amazon_linux_ami_root_device_name" {
  type    = string
  default = "/dev/xvda"
}

variable "redhat_launch_block_device_mappings_device_name" {
  type    = string
  default = "/dev/sda1"
}

variable "redhat_source_device_name" {
  type    = string
  default = "/dev/sda1"
}

variable "redhat_ami_root_device_name" {
  type    = string
  default = "/dev/sda1"
}

variable "redhat_volume_size" {
  type    = number
  default = 12
}

variable "amazon_linux_volume_size" {
  type    = number
  default = 8
}

variable "delete_on_termination" {
  type    = bool
  default = true
}

variable "tpm_support" {
  type    = string
  default = "v2.0"
}

variable "boot_mode" {
  type    = string
  default = "uefi"
}

variable "imds_support" {
  type    = string
  default = "v2.0"
}

source "amazon-chroot" "redhat" {
  source_ami             = var.redhat_source_ami
  region                 = var.region
  ami_name               = local.redhat_ami_name
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  tpm_support            = var.tpm_support
  boot_mode              = var.boot_mode
  imds_support           = var.imds_support
  instance_id            = "i-02f6db6b3f6939272"
}

source "amazon-chroot" "amazon-linux" {
  source_ami             = var.amazon_linux_source_ami
  region                 = var.region
  ami_name               = local.amazon_linux_ami_name
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  tpm_support            = var.tpm_support
  boot_mode              = var.boot_mode
  imds_support           = var.imds_support
  instance_id            = "i-01665d8b5a1a431f0"
}

build {
  sources = ["sources.amazon-chroot.redhat"]

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

build {
  sources = ["sources.amazon-chroot.amazon-linux"]

  provisioner "file" {
    source      = "../resources/conf/ima-policy"
    destination = "/tmp/ima-policy"
  }

  provisioner "file" {
    source      = "../resources/conf/agent.toml"
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
