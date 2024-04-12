#!/bin/bash

set -ex

supervisorctl start cosmian_vm_agent

# wait for cert and key to be generated by `cosmian_vm_agent` before starting nginx
until [ -f /var/lib/cosmian_vm/data/cert.pem ]; do sleep 1; done

if command -v restorecon &>/dev/null; then
  chown nginx /var/lib/cosmian_vm/data/*.pem
  restorecon /var/lib/cosmian_vm/data/*.pem
else
  chown www-data /var/lib/cosmian_vm/data/*.pem
fi
