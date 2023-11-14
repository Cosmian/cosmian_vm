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

# Configure IMA
## Policy
mkdir -p /etc/ima
cp data/ima-policy /etc/ima

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

update-grub

# Install deps
apt install nginx certbot -y


# Configure TLS and Nginx
service nginx stop
certbot certonly --standalone -d "$DN" -m  tech@cosmian.com -n --agree-tos
cp conf/nginx.conf "/etc/nginx/conf.d/$DN.conf"
sed -i "s/$DEFAULT_DN/DN/g" "/etc/nginx/conf.d/$DN.conf"
service nginx start

# If selinux is on
setsebool -P httpd_can_network_connect 1

# Rebooting
echo "You can now reboot"