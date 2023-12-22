# How to use Cosmian VM on SGX

Working with Cosmian VM on SGX enables an application to be executed in zero trust environment: 
- The memory is fully encrypted
- The filesystem is fully encrypted
- The network is fully encrypted (the certificate is generated right inside the enclave)

## Pre-requesites

1. Setup the SGX host
2. Compile `cosmian_vm_certtool` and move it to `./enclave/bin/cosmian_vm_certtool`
3. Compile `cosmian_vm_agent` and move it to `./enclave/bin/cosmian_vm_agent`
4. Compile your application and move it to `./enclave/bin/app`. You can find an example here: [`cosmian_helloworld`](https://github.com/Cosmian/helloworld-service)

Also, install some extra packages:

```sh
$ sudo apt install nginx
```
5. Edit the `etc/app/` as you need. This directory is designed to contains all configurations of the application. 

## Usage

Four enclaves will be generated:
- One to generate the ssl certificate using `cosmian_vm_certtool`. This enclave is shutdown after the certificate is generated
- One for `cosmian_vm_agent` binary
- One for `app` binary (your application)
- One for the nginx redirecting the https data to the http `app`

If one enclave raises an error, the whole program stops. 

The Cosmian VM Agent & App certificate is written in `./cosmian_vm_data` which is readable outside the enclave but can't be decrypted. 
The user application data are written in `./app_data` which is readable outside the enclave but can only be decrypted by other enclaves signed by the same key. 
By default, the nginx is configured to redirect HTTPS from 443 to 3000. 

```sh
$ cd enclave
$ sudo ./entrypoint.bash sgx.cosmian.dev tech@cosmian.com
```

It starts the four enclaves. 

For testing, you can add `--staging` as an argument of `cosmian_vm_certtool` in `run.sh`. It will remove ACME API limitations.

You can query the application by doing:

```sh
$ curl https://sgx.cosmian.dev
Hello, World!
```

You can verify the enclave by running:

```sh
$ cosmian_vm --url https://sgx.cosmian.dev:5355 snapshot 
Proceeding the snapshot...
The snapshot has been saved at ./cosmian_vm.snapshot

$ cosmian_vm --url https://sgx.cosmian.dev:5355 verify --snapshot ./cosmian_vm.snapshot
Reading the snapshot...
Fetching the collaterals...
[ WARNING ] No files hash in the snapshot
[ SKIP ] Verifying VM integrity
[ SKIP ] Verifying TPM attestation
[ OK ] Verifying TEE attestation
```

On SGX the snapshot won't contain the filehashes due to the fact that:
- IMA is not part of SGX technology
- The `mr_enclave` measurement is designed (and sufficient) to prove the integrity of the code running inside the enclave

Therefore, the verification step will only rely on the last one called `Verifying TEE attestation`.

## Docker

You can build a Cosmian VM docker for SGX by doing:

```sh
$ # From the root of the project
$ sudo docker build -f resources/sgx/Dockerfile.sgx -t cosmian-vm-sgx .
```

Then run it as follow:

```sh
$ docker run --device /dev/sgx_provision \
             --device /dev/sgx/enclave \
             --device /dev/sgx/provision \
             --device /dev/sgx_enclave \
             --rm \
             -v /var/run/aesmd:/var/run/aesmd \
             -v /root/.config/gramine/enclave-key.pem:/root/.config/gramine/enclave-key.pem \
             -p5355:5355 \
             -p443:443 \
             -p80:80 \
             -v "$(realpath ../helloworld-service/target/debug/helloworld)":/root/bin/app \
             --name cosmian_vm \
             cosmian-vm-sgx \
             sgx.cosmian.dev tech@cosmian.com
```

Note:
- Replace `helloworld` by your own application binary to run
- Replace the domain name and the email by your own.