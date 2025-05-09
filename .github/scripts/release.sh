#!/bin/bash

set -ex

OLD_VERSION="$1"
NEW_VERSION="$2"

sed -i "s/$OLD_VERSION/$NEW_VERSION/g" Cargo.toml
sed -i "s/$OLD_VERSION/$NEW_VERSION/g" .github/scripts/aws-packer-build.sh
sed -i "s/$OLD_VERSION/$NEW_VERSION/g" .github/scripts/azure-new-instance.sh
sed -i "s/$OLD_VERSION/$NEW_VERSION/g" .github/scripts/push_packages.sh
sed -i "s/$OLD_VERSION/$NEW_VERSION/g" ansible/group_vars/all.yml

# instead of replacing X.Y.Z, replace X-Y-Z
sed -i "s/${OLD_VERSION//./-}/${NEW_VERSION//./-}/g" .github/scripts/gcp-new-instance.sh

cargo build
git cliff -u -p CHANGELOG.md -t "$NEW_VERSION"
