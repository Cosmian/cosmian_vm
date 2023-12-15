#!/usr/bin/bash 

set -e

if [ "$EUID" -ne 0 ]
    then echo "Please run as root"
    exit
fi

usage() {
    echo "Usage: $0 <domain_name> <email> [--emulation]"
    echo ""
    echo "Using --emulation enables you to get the MR_ENCLAVE of the enclave server"
    echo "You don't need to use an SGX machine to use --emulation param"
    exit 1 
}

if [ $# -ne 2 ] && [ $# -ne 3 ]; then 
    usage
fi

DEBUG=0

if [ $# -eq 2 ]; then
    if ! [ -e "/dev/sgx_enclave" ]; then
            echo "You are not running on an sgx machine"
            echo "If you want to compute the MR_ENCLAVE, re-run with --emulation parameter"
            exit 1
        fi
        
        mkdir -p "$(dirname "$0")/var"

        # Start the enclave
        make clean && make DEBUG=$DEBUG DOMAIN_NAME="$1" EMAIL="$2" && gramine-sgx ./cosmian_vm
elif [ $# -eq 3 ] && [ "$3" = "--emulation" ]; then
    SGX_SIGNER_KEY=~/.config/gramine/enclave-key.pem
    if ! [ -f $SGX_SIGNER_KEY ]; then
        mkdir -p "$(dirname $SGX_SIGNER_KEY)"
        # Generate a dummy key. `MR_ENCLAVE` does not depend on it.
        openssl genrsa -3 -out $SGX_SIGNER_KEY 3072
    fi
    # Compile but don't start
    make DEBUG=$DEBUG DOMAIN_NAME="$1" EMAIL="$2"
else
    usage
fi
