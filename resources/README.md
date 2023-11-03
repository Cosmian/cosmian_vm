Disable apparmor and install selinux: https://www.linode.com/docs/guides/how-to-install-selinux-on-ubuntu-22-04/
Check:

$ ls -Z /bin/bash
-rwxr-xr-x. root root system_u:object_r:shell_exec_t:s0 /bin/bash

Liste des magic: https://elixir.bootlin.com/linux/v3.18.103/source/include/uapi/linux/magic.h

ps -aeZ

getfattr -m - -d /sbin/init



sudo find / -type f ! -path "./output"

sudo find / -type f ! -path "./output" -exec sha1sum  "{}" +  > ./output

