# Ansible script

Cosmian Ansible scripts provides roles and playbooks to install/reinstall the following products on [Cosmian Base Images](../CHANGELOG_BASE_IMAGES.md):

- Cosmian VM Images
- Cosmian KMS Images
- Cosmian AI Runner Images

## Example

```sh
bash .github/scripts/azure-new-instance.sh
# output HOST=X.X.X.X
export HOST=X.X.X.X
virtualenv env
source env/bin/activate
pip install -r python_modules.txt
ansible-playbook ai-runner-playbook.yml -i $HOST, -u cosmian --skip-tags role-cleanup
```
