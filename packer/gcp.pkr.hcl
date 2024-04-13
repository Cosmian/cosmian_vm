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
  default = "5m"
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

source "googlecompute" "TEMPLATE_GOOGLE_COMPUTE" {
  project_id              = var.project_id
  source_image            = "TEMPLATE_SOURCE_IMAGE"
  source_image_family     = "TEMPLATE_SOURCE_FAMILY"
  zone                    = var.zone
  ssh_username            = var.ssh_username
  ssh_timeout             = var.ssh_timeout
  image_name              = "TEMPLATE_IMAGE_NAME"
  image_guest_os_features = var.image_guest_os_features
  network                 = var.network
  subnetwork              = var.subnetwork
  tags                    = var.tags
  use_os_login            = var.use_os_login
  wait_to_add_ssh_keys    = var.wait_to_add_ssh_keys
}

build {
  sources = ["sources.googlecompute.TEMPLATE_GOOGLE_COMPUTE"]

  provisioner "ansible" {
    playbook_file   = "../ansible/TEMPLATE_PRODUCT-packer-playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=TEMPLATE_COSMIAN_VM_VERSION", "-e", "cosmian_kms_version=TEMPLATE_COSMIAN_KMS_VERSION"]
  }
}
