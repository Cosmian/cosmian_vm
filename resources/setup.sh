#!/usr/bin/bash

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

DN="cosmianvm.cosmian.dev"

# Configure IMA
## Policy
mkdir -p /etc/ima
cp data/ima-policy /etc/ima
## Hash method (we assume GRUB_CMDLINE_LINUX has no value yet)
sed -i /GRUB_CMDLINE_LINUX=/s/\"\"$/\"ima_hash=sha256\"/ /etc/default/grub
update-grub

# Install deps
apt install nginx certbot

# Configure TLS and Nginx
service nginx stop
certbot certonly --standalone -d $DN -m  tech@cosmian.com -n --agree-tos
cp conf/nginx.conf /etc/nginx/sites-enabled/$DN
service nginx start

# Rebooting
reboot