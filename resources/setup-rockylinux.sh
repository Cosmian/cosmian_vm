#!/usr/bin/bash

set -e 

if [ "$EUID" -ne 0 ]; then 
  echo "Please run as root"
  exit 1
fi

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <domain_name>"
  exit 1
fi

DEFAULT_DN="cosmianvm.cosmian.dev"
DN=$1

# Configure SEV
# modprobe sev-guest
echo "sev-guest" > /etc/modules-load.d/sev-guest.conf 

# Configure IMA
## Policy
mkdir -p /etc/ima
cp data/ima-policy /etc/ima

if [ -e "/sys/kernel/security/ima/policy" ]; then
  # Some OS expects the ima policy to be written in /sys/kernel/security/ima/policy
  # in order to load them on reboot
  # Note if it fails there: it means that the ima-policy is malformed (use: dmesg)
  cat /etc/ima/ima-policy > /sys/kernel/security/ima/policy
fi

## Hash method
if grep -q ima_hash= /etc/default/grub ; then
  # Hash method already set
  sed -i 's/ima_hash=[^ "]\+/ima_hash=sha256/' /etc/default/grub
else
  # Hash method not set
  sed -i /GRUB_CMDLINE_LINUX=/s/\"$/\ ima_hash=sha256\"/ /etc/default/grub
fi

## Template format
if grep -q ima_template= /etc/default/grub ; then
  # Template format already set
  sed -i 's/ima_template=[^ "]\+/ima_template=ima-ng/' /etc/default/grub
else
  # Template format not set
  sed -i /GRUB_CMDLINE_LINUX=/s/\"$/\ ima_template=ima-ng\"/ /etc/default/grub
fi

grub2-mkconfig -o "$(readlink -e /etc/grub2.conf)"

# Install deps
dnf install epel-release mod_ssl -y 
dnf install certbot nginx -y

# Configure TLS and Nginx
systemctl stop nginx 
certbot certonly --standalone -d "$DN" -m  tech@cosmian.com -n --agree-tos
cp conf/nginx.conf "/etc/nginx/conf.d/$DN.conf"
sed -i "s/$DEFAULT_DN/$DN/g" "/etc/nginx/conf.d/$DN.conf"
systemctl enable nginx
systemctl start nginx

# If selinux is on
setsebool -P httpd_can_network_connect 1

# Rebooting
echo "You can now reboot"