variable "ubuntu_source_ami" {
  type    = string
  default = "ami-00983e8a26e4c9bd9"
}

variable "redhat_source_ami" {
  type    = string
  default = "ami-0bd23a7080ec75f4d"
}

variable "region" {
  type    = string
  default = "eu-west-3"
}

variable "ubuntu_ssh_username" {
  type    = string
  default = "ubuntu"
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
  default = "t2.micro"
}

source "amazon-ebs" "ubuntu" {
  source_ami             = var.ubuntu_source_ami
  region                 = var.region
  ssh_username           = var.ubuntu_ssh_username
  ami_name               = var.ubuntu_ami_name
  instance_type          = var.instance_type
}

build {
  sources = ["sources.amazon-ebs.ubuntu"]
  provisioner "file" {
    source      = "../resources/post-install.sh"
    destination = "/tmp/cosmian_vm_post_install.sh"
  }

  provisioner "file" {
    source      = "../resources/data/ima-policy"
    destination = "/tmp/ima-policy"
  }

  provisioner "file" {
    source      = "../resources/conf/nginx.conf"
    destination = "/tmp/cosmian_vm_agent.conf"
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
