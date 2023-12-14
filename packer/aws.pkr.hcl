variable "ubuntu_source_ami" {
  type    = string
  default = "ami-0694d931cee176e7d"
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
  default = "10m"
}

variable "ubuntu_ami_name" {
  type    = string
  default = "cosmian-vm-ubuntu-{{timestamp}}"
}

variable "redhat_ami_name" {
  type    = string
  default = "cosmian-vm-redhat-{{timestamp}}"
}

variable "instance_type" {
  type    = string
  default = "c6a.large"
}


source "amazon-ebs" "ubuntu" {
  source_ami             = var.ubuntu_source_ami
  region                 = var.region
  ssh_username           = var.ubuntu_ssh_username
  ami_name               = var.ubuntu_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
}

source "amazon-ebs" "redhat" {
  source_ami             = var.redhat_source_ami
  region                 = var.region
  ssh_username           = var.redhat_ssh_username
  ami_name               = var.redhat_ami_name
  instance_type          = var.instance_type
  ssh_timeout            = var.ssh_timeout
}

build {
  sources = ["sources.amazon-ebs.ubuntu"]

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
  sources = ["sources.amazon-ebs.redhat"]

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