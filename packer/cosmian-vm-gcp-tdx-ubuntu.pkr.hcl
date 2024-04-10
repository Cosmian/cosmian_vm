/*
Copy the tdx capable image into the intel-enclaves project before building the image with packer.
To do so, run the command below :
gcloud alpha compute --project=intel-enclaves images create ubuntu-2204-tdx-v20231011 --family=ubuntu-2204-lts  --source-image=ubuntu-2204-tdx-v20231011  --source-image-project=tdx-guest-image
*/

variable "prefix" {
  type    = string
  default = "alpha"
}

variable "cosmian_vm_version" {
  type    = string
  default = "X.Y.Z"
}

locals {
  ubuntu_ami_name = "${var.prefix}-cosmian-vm-ubuntu-tdx"
}

variable "project_id" {
  type    = string
  default = "cosmian-dev"
}

variable "ubuntu_source_image" {
  type    = string
  default = "ubuntu-2204-tdx-v20231011"
}

variable "ubuntu_source_image_family" {
  type    = string
  default = "ubuntu-2204-lts"
}

variable "zone" {
  type    = string
  default = "us-central1-a"
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
  default = ["UEFI_COMPATIBLE", "VIRTIO_SCSI_MULTIQUEUE", "GVNIC", "TDX_CAPABLE"]
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
  default = ["ssh-full"]
}

variable "use_os_login" {
  type    = bool
  default = true
}

variable "wait_to_add_ssh_keys" {
  type    = string
  default = "10m"
}

source "googlecompute" "ubuntu" {
  project_id              = var.project_id
  source_image            = var.ubuntu_source_image
  source_image_family     = var.ubuntu_source_image_family
  zone                    = var.zone
  ssh_username            = var.ssh_username
  ssh_timeout             = var.ssh_timeout
  image_name              = local.ubuntu_ami_name
  image_guest_os_features = var.image_guest_os_features
  network                 = var.network
  subnetwork              = var.subnetwork
  tags                    = var.tags
  use_os_login            = var.use_os_login
  wait_to_add_ssh_keys    = var.wait_to_add_ssh_keys
}

build {
  sources = ["sources.googlecompute.ubuntu"]
  provisioner "ansible" {
    playbook_file   = "../ansible/packer_tdx_playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=${var.cosmian_vm_version}"]
  }
}
