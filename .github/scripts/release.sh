#!/bin/bash

set -ex

OLD_VERSION="$1"
NEW_VERSION="$2"

# BSD sed (macOS) requires empty string after -i; GNU sed (Linux) does not
if [[ "$(uname)" == "Darwin" ]]; then
  SED_INPLACE=(sed -i '')
else
  SED_INPLACE=(sed -i)
fi

"${SED_INPLACE[@]}" "s/$OLD_VERSION/$NEW_VERSION/g" Cargo.toml
"${SED_INPLACE[@]}" "s/$OLD_VERSION/$NEW_VERSION/g" .github/scripts/aws-packer-build.sh
"${SED_INPLACE[@]}" "s/$OLD_VERSION/$NEW_VERSION/g" .github/scripts/azure-new-instance.sh
"${SED_INPLACE[@]}" "s/$OLD_VERSION/$NEW_VERSION/g" .github/scripts/push_packages.sh
"${SED_INPLACE[@]}" "s/$OLD_VERSION/$NEW_VERSION/g" ansible/group_vars/all.yml
"${SED_INPLACE[@]}" "s/$OLD_VERSION/$NEW_VERSION/g" documentation/docs/deployment_guide.md

# instead of replacing X.Y.Z, replace X-Y-Z
"${SED_INPLACE[@]}" "s/${OLD_VERSION//./-}/${NEW_VERSION//./-}/g" .github/scripts/gcp-new-instance.sh

cargo build
git cliff -u -p CHANGELOG.md -t "$NEW_VERSION"
