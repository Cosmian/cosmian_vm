!!! info "Cosmian VM reminder"

    First, read detailed information about [Cosmian VM](./index.md) or about [Cosmian VM Agent and related software tools functioning](https://github.com/Cosmian/cosmian_vm).

    As a reminder, the Cosmian VM's goal is to verify Confidential VM trustworthiness and integrity at anytime. This verification runs on the operating system where one or more applications have been installed.

    First, a snapshot is generated, freezing the state of the system and all executable files.

    Then, at anytime, a remote verification of the VM can be done using the Cosmian VM CLI tool (`cosmian_vm`).

<p align="center">
  <img src="../images/confidential_vm_setup_flow.drawio.svg" alt="setup flow">
</p>

The Cosmian VM can be deployed on virtual machines that supports AMD SEV-SNP or Intel TDX technologies.

The following steps help to deploy a Cosmian VM instance on any [supported cloud provider](./index.md#cloud-providers-support).

The Cosmian VM contains a set of software to ensure trustworthiness of the executable environment of the VM.

Then, the deployment flow is the following:

- instantiate a **Cosmian Confidential VM**,
- connect on this VM and install everything required for your application to run correctly,
- create for once a VM snapshot remotely using Cosmian VM CLI,
- verify at anytime the integrity of the VM

## Instantiate Cosmian VM on your favorite cloud provider 🚚

Go the Cosmian marketplace webpage of the chosen cloud provider:

- [Cosmian VM/KMS/AI on AWS Marketplace](https://aws.amazon.com/marketplace/search/results?searchTerms=cosmian)
- [Cosmian VM/KMS/AI on Azure Marketplace](https://marketplace.microsoft.com/fr-fr/marketplace/apps?search=cosmian&page=1)
- [Cosmian VM/KMS/AI on GCP Marketplace](https://console.cloud.google.com/marketplace/browse?hl=fr&q=Cosmian)

Select an OS, set an external IP and continue until the Cosmian VM instance is spawned.

Here's the list of instance types by cloud provider

| Cloud provider | Azure             | GCP          | AWS           |
| -------------- | ----------------- | ------------ | ------------- |
| **AMD**        | **SNP**           | **SNP**      | **SNP**       |
|                | Standard_DCas_v5  | n2d-standard | M6a           |
|                | Standard_DCads_v5 |              | C6a           |
|                |                   |              | R6a           |
| **Intel**      | **TDX**           | **TDX**      | **TDX**       |
|                | DCes_v5-series    | c3-standard  | Not available |
|                | ECesv5-series     |              |               |
|                | (preview)         |              |               |

## Customize your VM 👩‍🔧

Connect to the spawned Cosmian VM using SSH and install whatever is required for application and services to run (installing software and dependencies, setting-up configurations and services etc.).

For example, deploy an app and [setup it as a Linux service](#deploy-your-application-as-a-service).

## Snapshot the VM remotely

Once the VM is configured as needed, Cosmian VM Agent can do a snapshot of the VM containing fingerprint of the executables and metadata related to TEE and TPM.

### Install the Cosmian VM CLI on your local machine

Install the Cosmian VM CLI on a local machine

=== "Ubuntu 22.04"

    Download the binary and allow it to be executed:

    ```console title="On the local machine"
    sudo apt update && sudo apt install -y wget
    wget https://package.cosmian.com/cosmian_vm/1.3.20/ubuntu-22.04/cosmian-vm_1.3.20-1_amd64.deb
    sudo apt install ./cosmian-vm_1.3.20-1_amd64.deb
    cosmian_vm --version
    ```

=== "Ubuntu 24.04"

    Download the binary and allow it to be executed:

    ```console title="On the local machine"
    sudo apt update && sudo apt install -y wget
    wget https://package.cosmian.com/cosmian_vm/1.3.20/ubuntu-24.04/cosmian-vm_1.3.20-1_amd64.deb
    sudo apt install ./cosmian-vm_1.3.20-1_amd64.deb
    cosmian_vm --version
    ```

=== "RHEL 9"

    Download the binary and allow it to be executed:

    ```console title="On the local machine"
    sudo dnf update && dnf install -y wget
    wget https://package.cosmian.com/cosmian_vm/1.3.20/rhel9/cosmian_vm-1.3.20-1.x86_64.rpm
    sudo dnf install ./cosmian_vm-1.3.20-1.x86_64.rpm
    cosmian_vm --version
    ```

=== "MacOS / Windows"

    Start a Ubuntu-based Docker container and enter it:

    ```console title="On the local machine"
    docker run -it ubuntu:22.04 /bin/bash
    ```

    Download the binary and allow it to be executed:

    ```console title="In Docker container (local machine)"
    apt update && apt install -y wget
    wget https://package.cosmian.com/cosmian_vm/1.3.20/ubuntu-22.04/cosmian-vm_1.3.20-1_amd64.deb
    apt install ./cosmian-vm_1.3.20-1_amd64.deb
    ```

Generate a snapshot of the Cosmian VM:

```console title="On the local machine"
cosmian_vm --url https://${COSMIAN_VM_IP_ADDR}:5555 --allow-insecure-tls snapshot
```

## Verify the VM snapshot

Take a look at the [global flow](./index.md#verification-of-the-remote-instance) to fully understand the whole verification process of a Cosmian VM.

Previous downloaded snapshot is stored as `cosmian_vm.snapshot` file (see the [previous step](#snapshot-the-vm-remotely)).

- At <u>anytime</u>, the Cosmian VM integrity can be verified by running:

```console title="On the local machine"
cosmian_vm --url https://${COSMIAN_VM_IP_ADDR}:5555 --allow-insecure-tls \
          verify --snapshot cosmian_vm.snapshot
```

!!! info "Why `--allow-insecure-tls` flag?"

    When the agent starts (see step [Snapshot the VM](#snapshot-the-vm-remotely)) self-signed certificate is created to enable HTTPS out of the box.

    These certificates must be replaced by trusted ones using tools like `cosmian_certtool` or Linux tools (`certbot` with **Let's Encrypt** for instance).

    See [how to setup trusted certificates](#configure-https-with-your-own-domain).

## Deploy your application as a service

The benefit of setting the deployed app as a service is that Cosmian VM Agent is able to:

- handle the lifetime cycle of the app (start, stop, restart)
- deploy safely (in encrypted folder) the config file

### Setup the systemd service

Connect to the Cosmian VM instance through SSH to perform this setup.

1. write a service file (here compatible with `systemctl`)

    ```toml title="my_app.service"
    [Unit]
    Description=My App
    Requires=multi-user.target
    After=multi-user.target mount_luks.service cosmian_vm_agent.service

    [Service]
    Type=simple
    User=root
    ExecStart=/usr/local/sbin/my_app
    Restart=on-failure
    RestartSec=3s
    Environment="MY_APP_CONF=/var/lib/cosmian_vm/data/app/app.conf"

    [Install]
    WantedBy=multi-user.target
    ```

    !!! info "About the paths..."

            Any provided config file will be renamed to `app.conf` and will be stored as filepath `/var/lib/cosmian_vm/data/app/app.conf`, so the app `my_app` may know the location of configuration file.

            Here the env variable `MY_APP_CONF` is used to forward this information to `my_app`.

2. register the new service to `systemctl`

Copy the `my_app.service` file into `/etc/systemd/system/my_app.service` and enable this service (without starting it, **Cosmian VM Agent** that will be responsible of starting it)

```bash
sudo systemctl enable my_app
sudo systemctl daemon-reload
```

**Note**: `my_app` and `my_app_svc` are indicative naming and can be changed, but don't forget to update the Cosmian VM Agent config file (`/etc/cosmian_vm/agent.toml`) as well.

### Configure the remote app safely

On the local machine, write the config file of the app, and then use the Cosmian VM CLI to remotely configure the app.

1. write the app config file

   ```toml title="my_app.toml"
   [my_cfg]
   key = "value"
   secret = "a98jfdol"
   ```

   The format (TOML, JSON, INI...) of this config file depends on the app but the Cosmian VM CLI doesn't care, as the config file is treated as a blob of bytes.

2. send the configuration using `cosmian_vm` CLI

   ```console
   cosmian_vm --url https://app.company.com app init -c my_app.toml
   ```

   The app conf is written in the encrypted folder.

   Cosmian VM Agent start/restart automatically the app
   after writing the config file when `init` is called.

### Control the remote app as a service

The `cosmian_vm` CLI also contains two subcommands designed to drive your application running inside the _Cosmian VM_.

It could be relevant if the personnel in charge of the application doesn't have the rights to connect to the _Cosmian VM_ through SSH for instance.

!!! warning "Security"

    If your *Cosmian VM* is reachable over Internet, be aware that anyone can control your application. Out of the box the access to the `cosmian_vm_agent` endpoints is not authenticated.

Before going any further, you need to add a paragraph `app` inside the _Cosmian VM_ configuration file, as follow:

```toml
[agent]
host = "0.0.0.0"
port = 5555
ssl_certificate = "data/cosmianvm.cosmian.dev/cert.pem"
ssl_private_key = "data/cosmianvm.cosmian.dev/key.pem"
tpm_device = "/dev/tpmrm0"

[app]
service_type = "systemd"
service_name = "my_app"
app_storage = "data/app"
```

The `service_type` could be `standalone`, `systemd` or `supervisor` and defines the way to start/stop the application.
The `service_name` field defines the name of the service or the binary to start/stop.
The `app_storage` field defines the root path where the application will store its data. If it's a relative path, the root path will be located inside `/var/lib/cosmian_vm/`. If you put it in the subfolder `data`, this directory is therefore encrypted and protected from the cloud provider.

!!! warning "only `data/` is encrypted"

    Only the `data/app/` part is customizable in the Cosmian VM Agent config file (`/etc/cosmian_vm/agent.toml`), but it remains relative to `/var/lib/cosmian_vm/`.

    Keep in mind that `/var/lib/cosmian_vm/data` is the mount point of the encrypted folder.
    Anything outside this folder is NOT encrypted!

Let's imagine, the application is installed in the _Cosmian VM_ but not configured and started. Then, you can provide a configuration file (containing secrets for instance) and start it using:

```console
cosmian_vm --url https://app.company.com app init --conf app.json
```

Also, if needed, the application can be restarted using:

```console
cosmian_vm --url https://app.company.com app restart
```

The configuration file can be anything the application expects. Here, a json file. It will be sent to the `cosmian_vm_agent` and stored in the LUKS container in `/var/lib/cosmian_vm/data/app/app.conf`.

If you call again `init` the previous configuration file is overwritten.

## Advanced settings

### Cosmian VM logs

The logs of _Cosmian VM_ are written in `journalctl` and can be accessed via `journalctl -exu cosmian_vm_agent`.

### Temporary folder

_Cosmian VM_ also contains an encrypted RAMFS in `/var/lib/cosmian_vm/tmp`.

The data stored inside is volatile and will be deleted when rebooting the VM. The size of this directory is 500MB.

### Data protection on the filesystem

Out of the box, _Cosmian VM_ filesystem is not entirely protected against the cloud provider.

However, there are two ways to store securely sensitive data:

- LUKS container
- RAMFS

### Encrypted folder

At the first start of `cosmian_vm_agent`, a [LUKS](https://doc.ubuntu-fr.org/cryptsetup) container is created at `/var/lib/cosmian_vm/container` (size=500MB).

This container is mounted into `/var/lib/cosmian_vm/data`. This directory can be used to store any sensitive data (ie: to store TLS certificate).

The password of the LUKS container is stored inside the LUKS itself, and can be used to migrate the container to another VM for example.

With _Cosmian VM_ the LUKS is enrolled on the current TPM therefore the password won't be asked again, even at reboot.

To change the size of the container, create it again:

```console
cosmian_fstool --size 100GB --location /data/container
```

Define your own container size and save the container to an additional disk for back-up (`/data/container` in this example).

A prompt will invite you to set the password. Save this password as it could be useful if you want to migrate the container to another VM.

If you need to manage the container (extend its size, revoke the password, etc.), you can use `cryptsetup`.

The fields `ssl_certificate` and `ssl_private_key` could be relative paths. In that case, this is always relative to `/var/lib/cosmian_vm/`.

### Add measurement of scripts and configuration files

When using _Cosmian VM_ with [RHEL](https://www.redhat.com/en/technologies/linux-platforms/enterprise-linux) distribution, [SELinux](https://www.redhat.com/en/topics/linux/what-is-selinux) is enabled and configured in enforced mode.
The SELinux module named `cosmiand` is present in _Cosmian VM_ with 4 custom labels measured by Integrity Measurement Architecture (IMA):

- `cosmiand_exec_t` for Cosmian VM related binaries which will transition to `cosmiand_t`
- `cosmiand_t` for daemons when the binary file is labeled with `cosmian_exec_t`
- `cosmiand_script_t` for any script file such as Python, Perl or shell scripts
- `cosmiand_conf_t` for Cosmian VM configuration files

Because IMA only tracks binaries loaded in memory, `cosmiand_script_t` and `cosmiand_conf_t` allow to label specific files read by interpreters or applications.
It will ensure that these files are not modified after the snapshot.

To label a new file, use `semanage` and `restorecon` commands:

```console
# change SELinux context (always use absolute path)
semanage fcontext -a -t cosmiand_script_t "$(realpath my_script.py)"
# restore default context to apply the change (always use absolute path)
restorecon -v "$(realpath my_script.py)"
```

then you can see the new context with:

```console
$ ls -Z
system_u:object_r:cosmiand_script_t:s0 my_script.py
```

See [RHEL SELinux documentation](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html-single/using_selinux/index#proc_providing-feedback-on-red-hat-documentation_using-selinux) for more details and advanced usage of SELinux.

### Cosmian VM Agent lifecycle

_Cosmian VM Agent_ can be started/restarted/stopped.

Start the agent:

```console
systemctl start cosmian_vm_agent
```

Restart the agent

```sh
systemctl restart cosmian_vm_agent
```

Stop the agent

```sh
systemctl stop cosmian_vm_agent
```

### Configure Cosmian VM Agent

_Cosmian VM Agent_ relies on a configuration file located at `/etc/cosmian_vm/agent.toml`.

A minimal configuration file is:

```toml title="/etc/cosmian_vm/agent.toml"
[agent]
host = "127.0.0.1"
port = 5555
ssl_certificate = "data/cert.pem"
ssl_private_key = "data/key.pem"
tpm_device = "/dev/tpmrm0"
```

By default, `cosmian_vm_agent` uses the port 5555 on localhost.

### Configure HTTPS with your own domain

On a fresh install, `cosmian_vm_agent` uses a self-signed certificate generated at the start of the service and set the `CommonName` of the certificate to the value of the machine hostname.

Therefore when using the CLI, `--allow-insecure-tls` must be added to ignore SSL errors (due to self-signed cert). This is not a good practice in production.

To enable HTTPS with trusted certs:

- Edit your DNS registry to point to that VM
- Create a trusted certificate using the method of your choice (_Let's encrypt_ for instance) or using `cosmian_certtool`
- Edit the `nginx` configuration file to point to the location of the TLS certificate and private key:

  ```conf title="/etc/nginx/conf.d/default.conf"
  server {
          ...

          ssl_certificate /var/lib/cosmian_vm/data/cert.pem;
          ssl_certificate_key /var/lib/cosmian_vm/data/key.pem;
  }
  ```

- Edit the `cosmian_vm_agent` configuration file to point to the location of the TLS certificate and private key:

  ```toml title="/etc/cosmian_vm/agent.toml"
  [agent]
  ...
  ssl_certificate = "data/cert.pem"
  ssl_private_key = "data/key.pem"
  ```

- Restart both services

  ```console
  sudo systemctl restart nginx
  sudo systemctl restart cosmian_vm_agent
  ```

### Configure HTTPS with another certs

This configuration example sets the certificate from another application running in _Cosmian VM_:

```toml title="/etc/cosmian_vm/agent.toml"
[agent]
host = "0.0.0.0"
port = 5555
ssl_certificate = "/etc/letsencrypt/live/app.company.com/fullchain.pem"
ssl_private_key = "/etc/letsencrypt/live/app.company.com/privkey.pem"
tpm_device = "/dev/tpmrm0"
```

**Note**: in this case, `fullchain.pem` and `privkey.pem` are not located in the LUKS (under `/var/lib/cosmian_vm/data/`) so they are stored in cleartext on the disk hence potentially accessible by the Cloud Provider.

If you want to store these certificate securely, move them into the LUKS and change the location in the `agent.toml` file (don't forget to do the same for Nginx if applicable) like this:

```toml title="/etc/cosmian_vm/agent.toml"
[agent]
...
ssl_certificate = "data/fullchain.pem" # equivalent to /var/lib/cosmian_vm/data/fullchain.pem
ssl_private_key = "data/privkey.pem"   # equivalent to /var/lib/cosmian_vm/data/privkey.pem
```

By setting a relative path, the agent will complete the path using the default mounted path for the LUKS, which is `/var/lib/cosmian_vm/data` ([read here](#data-protection-on-the-filesystem) for more information about LUKS setup).

### Snapshot deep dive

#### Take a snapshot of the remote system

```console title="On the local machine"
cosmian_vm --url https://app.company.com snapshot
```

The snapshot is performed by `cosmian_vm_agent` to produce a single file with the trusted state of the machine at the current time.
This file is returned to the _Cosmian VM CLI_ when using the subcommand `snapshot`.

The snapshot file is a JSON file containing:

- IMA measurement log at the current time
- List of all measured files and their corresponding hash (`sha256` by default)
- TEE metadata
- TPM metadata

Here is a sample:

```json
{
  "tee_policy":{
    "Sev":{
      "measurement":"73797518025d1d20e09efdf10e383cd0115e00562109b04ec577b5bd5d2ddab12f1f9ee22758f50a121355cf5aac6507",
      "report_data":"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
      "family_id":"00000000000000000000000000000000",
      "image_id":"00000000000000000000000000000000",
      "guest_svn":0,
      "id_key_digest":"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
      "author_key_digest":"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
      "policy":196608,
      "report_id":"b6db1e43f868a4823f708df92942a0448750fc5c37f7a724bc49292ffd111e1a",
      "report_id_ma":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    }
  },
  "tpm_policy":{
    "reset_count":15,
    "restart_count":0
  },
  "filehashes":[
    ["/snap/google-cloud-cli/187/platform/gsutil/third_party/mock/mock/__pycache__/__init__.cpython-39.pyc", "f708df92942a044875[...]abc"],
    ...
  ]
}
```

### Verification with multiple domains

Typically, the verification is done using the CLI as follow:

```console
cosmian_vm --url https://app.company.com verify \
        --snapshot cosmian_vm.snapshot
```

When verifying a Cosmian VM you can also check that the TLS certificate of your services installed inside this VM are the one used when querying the Cosmian VM Agent during the verification.

The goal is to verify your services currently running inside this Cosmian VM.

To do so, use `--application` as many times as needed:

```console
cosmian_vm --url https://app.company.com verify \
        --snapshot cosmian_vm.snapshot \
        --application service1.company.com:3655 \
        --application service2.company.com
```

### Verification failure after reboot

Due to the IMA design, the VM cannot be trusted after a reboot.
Indeed the IMA is reset at each restart.

_Cosmian VM_ uses the TPM to detect a VM reboot: if the reset counter differs from the one captured during the snapshot, the verification fails.

If this reboot is trusted (for instance it has been made by the system administrator), you just need to [ask for a new snapshot](#take-a-snapshot-of-the-remote-system).

If the reboot occurs without the system administrator consent, the VM should be investigated to detect any malicious modification.

### Cosmian VM diff with a Ubuntu/RHEL base image

The modifications are related to the installation and the configuration of _Cosmian VM_ software stack.
All the changes are performed using Packer and can be found in [Cosmian VM Github page](https://github.com/Cosmian/cosmian_vm).

_Cosmian VM_ image:

- contains the fully configured IMA
- contains the fully configured SELinux (RHEL only)
- disables the auto-update (see [Disabled auto-update](#disabled-auto-update))
- contains the fully configured `cosmian_vm_agent`

This is an abstract of the updated file tree:

```sh
.
├── etc
│   ├── apt
│   │    └── apt.conf.d
│   │       └── 10periodic
│   ├── cosmian_vm
│   │   └── agent.toml
│   ├── default
│   │   └── grub
│   ├── ima
│   │   └── ima-policy
│   └── systemd
│       └── system
│           └── cosmian_vm_agent.service
├── usr
│   └── sbin
│       ├── cosmian_certtool
│       ├── cosmian_fstool
│       └── cosmian_vm_agent
└── var
    └── lib
        └── cosmian_vm
            ├── container   <--- LUKS container
            ├── tmp
            └── data        <--- LUKS container mounted
                ├── cert.pem
                └── cert.key
```

### Disabled auto-update

The verification performed by the _Cosmian VM_ relies on the fact that once the snapshot has been made, the VM content shouldn't be altered. If any modification is detected, the VM is considered compromised.

An auto-update processing alters the VM and makes the comparison with the snapshot impossible.

You shall update the Cosmian VM manually and create a new snapshot afterwards.

### Use a proxy in front of the Cosmian VM Agent

Although it's technically possible to configure an HTTPS proxy in front of the _Cosmian VM Agent_, it will prevent you from proceeding the verification through the CLI if you configure the proxy as an SSL forward.

Indeed, the TLS certificate configured in the _Cosmian VM Agent_ is a part of the exchange with the TEE. To avoid any malicious software to send fake collaterals, the TLS certificate used to get these collaterals should be the same as the one configured in the _Cosmian VM Agent_. This certificate should stay inside the _Cosmian VM_.

If you use a proxy, the TLS tunnel, from the CLI to the agent, uses the certificate of the proxy and not the certificate of the agent. The verification will therefore fail, being not able to determine if it's a malicious or healthy behavior.

However, if you use a proxy as an SSL passthrough, it will work like a charm.

Here is an example with HAProxy:

```c
[...]

defaults
    mode                 tcp
    option               tcplog
    [...]

# Entrypoint of the ha_proxy listen on 443
frontend https-in
    # Do not decrypt ssl yet
    bind *:443

    tcp-request inspect-delay 60s
    tcp-request content accept if { req_ssl_hello_type 1 }
    use_backend 8d14ff4ac2d452c3.cosmian.io if { req_ssl_sni -i 8d14ff4ac2d452c3.cosmian.io }
    # No default backend: return an error as fallback

backend 8d14ff4ac2d452c3.cosmian.io
    server haproxy-app 162.19.91.151:27283 check
```

Other resources:

- [External Passthrough Load Balancers](https://cloud.google.com/load-balancing/docs/network)
- [AWS - Network Load Balancer](https://docs.aws.amazon.com/elasticloadbalancing/latest/network/introduction.html)
- [Azure Load Balancer](https://learn.microsoft.com/en-us/azure/load-balancer/)
