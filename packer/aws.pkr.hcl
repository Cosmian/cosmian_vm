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

variable "volume_type" {
  type    = string
  default = "gp3"
}

variable "launch_block_device_mappings_device_name" {
  type    = string
  default = "/dev/xvdf"
}

variable "source_device_name" {
  type    = string
  default = "/dev/xvdf"
}

variable "ami_root_device_name" {
  type    = string
  default = "/dev/xvda"
}

variable "volume_size" {
  type    = number
  default = 8
}

variable "delete_on_termination" {
  type    = bool
  default = true
}

variable "iops" {
  type    = number
  default = 3000
}

variable "throughput" {
  type    = number
  default = 125
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

  launch_block_device_mappings {
    volume_type = var.volume_type
    device_name = var.launch_block_device_mappings_device_name 
    volume_size = var.volume_size
    delete_on_termination = var.delete_on_termination
    iops = var.iops
    throughput = var.throughput
  }

  ami_root_device {
    source_device_name = var.source_device_name
    device_name = var.ami_root_device_name
    volume_size = var.volume_size
    volume_type = var.volume_type
    delete_on_termination = var.delete_on_termination
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

  launch_block_device_mappings {
    volume_type = var.volume_type
    device_name = var.launch_block_device_mappings_device_name 
    volume_size = var.volume_size
    delete_on_termination = var.delete_on_termination
    iops = var.iops
    throughput = var.throughput
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

