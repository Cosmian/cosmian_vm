#!/usr/bin/bash

set -e 

if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root"
    exit 1
fi

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <domain_name>"
    echo -e "\nComplete the installation of the cosmian vm. Including:"
    echo -e "\t- TLS certificate generation"
    echo -e "\t- Nginx configuration adaptation and reloading"
    exit 1
fi

DN_PLACEHOLDER="COSMIAN_VM_DN_PLACEHOLDER"
DN=$1
NGINX_CONF_PATH="/etc/nginx/sites-available/cosmian_vm_agent.conf"

# Configure TLS and Nginx
systemctl stop nginx 
certbot certonly --standalone -d "$DN" -m  tech@cosmian.com -n --agree-tos
sed -i "s/$DN_PLACEHOLDER/$DN/g" "$NGINX_CONF_PATH"
ln -s "$NGINX_CONF_PATH" /etc/nginx/sites-enabled/cosmian_vm_agent.conf
echo '0 12 * * * certbot renew --nginx --post-hook "service nginx restart"' | crontab -
systemctl enable nginx
systemctl start nginx

COSMIAN_VM_AGENT_CERTIFICATE="/etc/letsencrypt/live/$DN/cert.pem"
SUPERVISOR_CONF_PATH="/etc/supervisor/conf.d/cosmian_vm_agent.conf"

sed -i "s/$DN_PLACEHOLDER/$COSMIAN_VM_AGENT_CERTIFICATE/g" "$SUPERVISOR_CONF_PATH"
supervisorctl reread
supervisorctl update