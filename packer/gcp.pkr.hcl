variable "project_id" {
  type    = string
  default = "amd-sev-snp"
}

variable "ubuntu_source_image" {
  type    = string
  default = "ubuntu-2204-jammy-v20231030"
}

variable "ubuntu_source_image_family" {
  type    = string
  default = "ubuntu-2204-lts"
}

variable "redhat_source_image" {
  type    = string
  default = "rhel-9-v20231115"
}

variable "redhat_source_image_family" {
  type    = string
  default = "rhel-9"
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

variable "ubuntu_image_name" {
  type    = string
  default = "cosmian-vm-ubuntu-{{timestamp}}"
}

variable "redhat_image_name" {
  type    = string
  default = "cosmian-vm-redhat-{{timestamp}}"
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
  default = ["ssh-full"]
}

variable "use_os_login" {
  type    = bool
  default = true
}

source "googlecompute" "ubuntu" {
  project_id             = var.project_id
  source_image           = var.ubuntu_source_image
  source_image_family    = var.ubuntu_source_image_family
  zone                   = var.zone
  ssh_username           = var.ssh_username
  ssh_timeout            = var.ssh_timeout
  image_name             = var.ubuntu_image_name
  image_guest_os_features = var.image_guest_os_features
  network                = var.network
  subnetwork             = var.subnetwork
  tags                   = var.tags
  use_os_login           = var.use_os_login
}

source "googlecompute" "redhat" {
  project_id             = var.project_id
  source_image           = var.redhat_source_image
  source_image_family    = var.redhat_source_image_family
  zone                   = var.zone
  ssh_username           = var.ssh_username
  ssh_timeout            = var.ssh_timeout
  image_name             = var.redhat_image_name
  image_guest_os_features = var.image_guest_os_features
  network                = var.network
  subnetwork             = var.subnetwork
  tags                   = var.tags
  use_os_login           = var.use_os_login
}

build {
  sources = ["sources.googlecompute.ubuntu", "sources.googlecompute.redhat"]
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

