variable "prefix" {}

locals {
  amazon_linux_ami_name = "${var.prefix}-cosmian-vm-amazon-linux-{{timestamp}}"
}

variable "amazon_linux_source_ami" {
  type    = string
  default = "ami-02cad064a29d4550c"
}

variable "region" {
  type    = string
  default = "eu-west-1"
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

variable "tpm_support" {
  type    = string
  default = "v2.0"
}

source "amazon-ebssurrogate" "amazon-linux" {
  source_ami             = var.amazon_linux_source_ami
  region                 = var.region
  ssh_username           = var.amazon_linux_ssh_username
  ami_name               = local.amazon_linux_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
  boot_mode              = var.boot_mode
  ami_virtualization_type = var.ami_virtualization_type
  ena_support            = var.ena_support
  tpm_support            = var.tpm_support

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
  sources = ["sources.amazon-ebssurrogate.amazon-linux"]

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