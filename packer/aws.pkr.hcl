variable "prefix" {}

locals {
  ubuntu_ami_name = "${var.prefix}-cosmian-vm-ubuntu-{{timestamp}}"
  redhat_ami_name = "${var.prefix}-cosmian-vm-redhat-{{timestamp}}"
}

variable "ubuntu_source_ami" {
  type    = string
  default = "ami-0905a3c97561e0b69"
}

variable "redhat_source_ami" {
  type    = string
  default = "ami-049b0abf844cab8d7"
}

variable "region" {
  type    = string
  default = "eu-west-1"
}

variable "ubuntu_ssh_username" {
  type    = string
  default = "ubuntu"
}

variable "redhat_ssh_username" {
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

variable "boot_mode" {
  type    = string
  default = "uefi"
}

variable "ami_virtualization_type" {
  type    = string
  default = "hvm"
}

variable "ena_support" {
  type    = bool
  default = true
}

variable "imds_support" {
  type    = string
  default = "v2.0"
}

source "amazon-ebssurrogate" "ubuntu" {
  source_ami             = var.ubuntu_source_ami
  region                 = var.region
  ssh_username           = var.ubuntu_ssh_username
  ami_name               = local.ubuntu_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
  boot_mode              = var.boot_mode
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  imds_support           = var.imds_support

  launch_block_device_mappings {
    volume_type = "gp2"
    device_name = "/dev/xvda" 
    delete_on_termination = true
    volume_size = 10
  }

  ami_root_device {
    source_device_name = "/dev/xvda"
    device_name = "/dev/xvda"
    delete_on_termination = true
    volume_size = 16
    volume_type = "gp2"
  }
}

source "amazon-ebssurrogate" "redhat" {
  source_ami             = var.redhat_source_ami
  region                 = var.region
  ssh_username           = var.redhat_ssh_username
  ami_name               = local.redhat_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
  boot_mode              = var.boot_mode
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  imds_support           = var.imds_support

  launch_block_device_mappings {
    volume_type = "gp2"
    device_name = "/dev/xvda" 
    delete_on_termination = true
    volume_size = 10
  }

  ami_root_device {
    source_device_name = "/dev/xvda"
    device_name = "/dev/xvda"
    delete_on_termination = true
    volume_size = 16
    volume_type = "gp2"
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

