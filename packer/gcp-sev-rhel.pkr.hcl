variable "prefix" {}

locals {
  redhat_ami_name = "${var.prefix}-cosmian-vm-sev-rhel"
}

variable "project_id" {
  type    = string
  default = "cosmian-dev"
}

variable "zone" {
  type    = string
  default = "europe-west4-a"
}

variable "ssh_username" {
  type    = string
  default = "root"
}

variable "ssh_timeout" {
  type    = string
  default = "10m"
}

variable "image_guest_os_features" {
  type    = list(string)
  default = ["SEV_SNP_CAPABLE"]
}

variable "network" {
  type    = string
  default = "default"
}

variable "subnetwork" {
  type    = string
  default = "default"
}

variable "tags" {
  type    = list(string)
  default = ["ssh"]
}

variable "use_os_login" {
  type    = bool
  default = true
}

variable "wait_to_add_ssh_keys" {
  type    = string
  default = "20s"
}

variable "redhat_source_image" {
  type    = string
  default = "rhel-9-v20231115"
}

variable "redhat_source_image_family" {
  type    = string
  default = "rhel-9"
}

source "googlecompute" "redhat" {
  project_id             = var.project_id
  source_image           = var.redhat_source_image
  source_image_family    = var.redhat_source_image_family
  zone                   = var.zone
  ssh_username           = var.ssh_username
  ssh_timeout            = var.ssh_timeout
  image_name             = local.redhat_ami_name
  image_guest_os_features = var.image_guest_os_features
  network                = var.network
  subnetwork             = var.subnetwork
  tags                   = var.tags
  use_os_login           = var.use_os_login
  wait_to_add_ssh_keys   = var.wait_to_add_ssh_keys
}

build {
  sources = ["sources.googlecompute.redhat"]

  provisioner "file" {
    source      = "../resources/conf/ima-policy-selinux"
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
    source      = "./../target/release/cosmian_vm_agent"
    destination = "/tmp/"
  }

  provisioner "file" {
    source      = "./../target/release/cosmian_certtool"
    destination = "/tmp/"
  }

  provisioner "ansible" {
    playbook_file = "../ansible/cosmian_vm_playbook.yml"
    local_port    = 22
    use_proxy     = false
  }
}
