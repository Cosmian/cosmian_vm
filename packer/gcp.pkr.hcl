packer {
  required_plugins {
    googlecompute = {
      version = "= 1.1.4"
      source  = "github.com/hashicorp/googlecompute"
    }
    ansible = {
      version = "= 1.1.1"
      source  = "github.com/hashicorp/ansible"
    }
  }
}

source "googlecompute" "TEMPLATE_GOOGLE_COMPUTE" {
  ssh_username              = "packer"
  ssh_timeout               = "5m"
  ssh_clear_authorized_keys = true
  project_id                = "cosmian-dev"
  source_image              = "TEMPLATE_SOURCE_IMAGE"
  source_image_family       = "TEMPLATE_SOURCE_FAMILY"
  zone                      = "europe-west4-a"
  image_name                = "TEMPLATE_IMAGE_NAME"
  image_guest_os_features   = ["TEMPLATE_OS_FEATURES"]
  network                   = "default"
  subnetwork                = "default"
  tags                      = ["ssh"]
  use_os_login              = true
  wait_to_add_ssh_keys      = "60s"
}

build {
  sources = ["sources.googlecompute.TEMPLATE_GOOGLE_COMPUTE"]

  provisioner "ansible" {
    playbook_file   = "../ansible/TEMPLATE_PRODUCT-packer-playbook.yml"
    local_port      = 22
    use_proxy       = false
    extra_arguments = ["-e", "cosmian_vm_version=TEMPLATE_COSMIAN_VM_VERSION", "-e", "cosmian_kms_version=TEMPLATE_COSMIAN_KMS_VERSION", "-e", "cosmian_ai_runner_version=TEMPLATE_COSMIAN_AI_RUNNER_VERSION"]
  }

  provisioner "shell" {
    skip_clean = true
    expect_disconnect = true
    inline = [
      "rm -f /etc/sudoers.d/90-cloud-init-users",
      "/usr/sbin/userdel -r -f packer ",
    ]
  }
}
