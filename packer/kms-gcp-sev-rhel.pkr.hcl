variable "prefix" {
  type    = string
  default = "alpha"
}

variable "cosmian_vm_version" {
  type    = string
  default = "X.Y.Z"
}

variable "cosmian_kms_version" {
  type    = string
  default = "X.Y.Z"
}

locals {
  redhat_ami_name = "${var.prefix}-kms-rhel-sev"
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
  default = "30s"
}

variable "redhat_source_image" {
  type    = string
  default = "rhel-9-v20240312"
}

variable "redhat_source_image_family" {
  type    = string
  default = "rhel-9"
}

source "googlecompute" "redhat" {
  project_id              = var.project_id
  source_image            = var.redhat_source_image
  source_image_family     = var.redhat_source_image_family
  zone                    = var.zone
  ssh_username            = var.ssh_username
  ssh_timeout             = var.ssh_timeout
  image_name              = local.redhat_ami_name
  image_guest_os_features = var.image_guest_os_features
  network                 = var.network
  subnetwork              = var.subnetwork
  tags                    = var.tags
  use_os_login            = var.use_os_login
  wait_to_add_ssh_keys    = var.wait_to_add_ssh_keys
}

build {
  sources = ["sources.googlecompute.redhat"]

  provisioner "ansible" {
    playbook_file   = "../ansible/packer_sev_playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=${var.cosmian_vm_version}", "-e", "cosmian_kms_version=${var.cosmian_kms_version}"]
  }
}
