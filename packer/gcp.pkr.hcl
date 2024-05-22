source "googlecompute" "TEMPLATE_GOOGLE_COMPUTE" {
  project_id              = "cosmian-dev"
  source_image            = "TEMPLATE_SOURCE_IMAGE"
  source_image_family     = "TEMPLATE_SOURCE_FAMILY"
  zone                    = "europe-west4-a"
  ssh_username            = "root"
  ssh_timeout             = "5m"
  image_name              = "TEMPLATE_IMAGE_NAME"
  image_guest_os_features = ["SEV_SNP_CAPABLE"] # Fix on TDX
  network                 = "default"
  subnetwork              = "default"
  tags                    = ["ssh"]
  use_os_login            = true
  wait_to_add_ssh_keys    = "60s"
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
