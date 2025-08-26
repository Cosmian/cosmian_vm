variable "gcp_credentials_file" {
  type        = string
  description = "packer.json"
  default     = "packer.json"
}

source "googlecompute" "redhat" {
  credentials_json          = file(var.gcp_credentials_file)
  ssh_username              = "root"
  ssh_timeout               = "5m"
  ssh_clear_authorized_keys = true
  project_id                = "cosmian-dev"
  source_image              = "rhel-9-v20250709"
  source_image_family       = "rhel-9"
  zone                      = "europe-west4-a"
  image_name                = "base-image-0-0-0-rhel-sev"
  image_guest_os_features   = ["SEV_SNP_CAPABLE"]
  network                   = "default"
  subnetwork                = "default"
  tags                      = ["ssh"]
  use_os_login              = true
  wait_to_add_ssh_keys      = "60s"
}

build {
  sources = ["sources.googlecompute.redhat"]
  provisioner "ansible" {
    playbook_file   = "../ansible/base-image-packer-playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=last_build/update_packer_plugins", "-e", "cosmian_kms_version=", "-e", "cosmian_ai_runner_version="]
  }
}
