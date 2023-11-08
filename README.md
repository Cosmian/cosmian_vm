# Cosmian VM

## Compile

```sh
$ cargo build
```

## Usage

On a SEV host:

```sh  
$ sudo COSMIAN_VM_AGENT_CERTIFICATE=/etc/letsencrypt/live/cosmianvm.cosmian.dev/cert.pem ./cosmian_vm_agent
```

Then on your localhost:

1. Create a snapshot (once)
   
```sh
$ cosmian_vm snapshot --url http://34.147.16.242:8080 
```

2. Verify the current state of the machine

```sh
$ cosmian_vm verify --url http://34.147.16.242:8080 --snapshot cosmian_vm.snapshot  
```
