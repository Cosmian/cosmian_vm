#!/usr/bin/sh

# Generate a certificate inside the enclave
/bin/cosmian_certtool acme --workspace /var/lib/cosmian_vm --output /var/lib/cosmian_vm --domain "$1" --email "$2" && 

(
# Start the Cosmian VM Agent
/bin/cosmian_vm_agent & 

# Run the main application
/bin/app &

# Run nginx to root the https to the http microservice (app)
/usr/sbin/nginx -p /etc/nginx -c nginx.conf 
)

# Note: never put that last executable in background using '&' 