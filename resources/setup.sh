#!/usr/bin/bash

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

DN="cosmianvm.cosmian.dev"

# Configure IMA
mkdir -p /etc/ima
cp data/ima-policy /etc/ima

# Install deps
apt install nginx certbot

# Configure TLS and Nginx
service nginx stop
certbot certonly --standalone -d $DN -m  tech@cosmian.com -n --agree-tos
cp conf/nginx.conf /etc/nginx/sites-enabled/$DN
service nginx start

# Rebooting
reboot