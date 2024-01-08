variable "prefix" {}

locals {
  ubuntu_ami_name = "${var.prefix}-cosmian-vm-ubuntu-{{timestamp}}"
  redhat_ami_name = "${var.prefix}-cosmian-vm-redhat-{{timestamp}}"
}

variable "redhat_source_ami" {
  type    = string
  default = "ami-049b0abf844cab8d7"
}

variable "ubuntu_source_ami" {
  type    = string
  default = "ami-02d014f12327de757"
}

variable "region" {
  type    = string
  default = "eu-west-1"
}

variable "redhat_ssh_username" {
  type    = string
  default = "ec2-user"
}

variable "ubuntu_ssh_username" {
  type    = string
  default = "ubuntu"
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

variable "launch_block_device_mappings_device_name" {
  type    = string
  default = "/dev/sda1"
}

variable "source_device_name" {
  type    = string
  default = "/dev/sda1"
}

variable "ami_root_device_name" {
  type    = string
  default = "/dev/sda1"
}

variable "volume_size" {
  type    = number
  default = 12
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

source "amazon-ebssurrogate" "redhat" {
  source_ami             = var.redhat_source_ami
  region                 = var.region
  ssh_username           = var.redhat_ssh_username
  ami_name               = local.redhat_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  tpm_support            = var.tpm_support
  boot_mode              = var.boot_mode
  imds_support           = var.imds_support

  launch_block_device_mappings {
    volume_type = var.volume_type
    device_name = var.launch_block_device_mappings_device_name 
    volume_size = var.volume_size
    delete_on_termination = var.delete_on_termination
  }

  ami_root_device {
    source_device_name = var.source_device_name
    device_name = var.ami_root_device_name
    volume_size = var.volume_size
    volume_type = var.volume_type
    delete_on_termination = var.delete_on_termination
  }
}

source "amazon-ebssurrogate" "ubuntu" {
  source_ami             = var.ubuntu_source_ami
  region                 = var.region
  ssh_username           = var.ubuntu_ssh_username
  ami_name               = local.ubuntu_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  tpm_support            = var.tpm_support
  boot_mode              = var.boot_mode
  imds_support           = var.imds_support

  launch_block_device_mappings {
    volume_type = var.volume_type
    device_name = var.launch_block_device_mappings_device_name 
    volume_size = var.volume_size
    delete_on_termination = var.delete_on_termination
  }

  ami_root_device {
    source_device_name = var.source_device_name
    device_name = var.ami_root_device_name
    volume_size = var.volume_size
    volume_type = var.volume_type
    delete_on_termination = var.delete_on_termination
  }
}

build {
  sources = ["sources.amazon-ebssurrogate.redhat"]

  provisioner "file" {
    source      = "../resources/data/ima-policy"
    destination = "/tmp/ima-policy"
  }

  provisioner "file" {
    source      = "../resources/conf/agent.toml"
    destination = "/tmp/agent.toml"
  }

  provisioner "file" {
    source      = "./cosmian_vm_agent"
    destination = "/tmp/"
  }

  provisioner "ansible" {
    playbook_file = "../ansible/cosmian_vm_playbook.yml"
    local_port    = 22
    use_proxy     = false
  }
}

build {
  sources = ["sources.amazon-ebssurrogate.ubuntu"]

  provisioner "file" {
    source      = "../resources/data/ima-policy"
    destination = "/tmp/ima-policy"
  }

  provisioner "file" {
    source      = "../resources/conf/agent.toml"
    destination = "/tmp/agent.toml"
  }

  provisioner "file" {
    source      = "./cosmian_vm_agent"
    destination = "/tmp/"
  }

  provisioner "ansible" {
    playbook_file = "../ansible/cosmian_vm_playbook.yml"
    local_port    = 22
    use_proxy     = false
  }
}