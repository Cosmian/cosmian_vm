# Release

## Table of contents

- [Release](#release)
  - [Table of contents](#table-of-contents)
  - [Step by step](#step-by-step)

## Step by step

To proceed a new release, please follow the steps below:

1. Create a new git-branch: `git checkout -b release/X.Y.Z`. Refer to the [CHANGELOG.md](CHANGELOG.md) for the version number.

2. (Possibly) Release the Cosmian Base VM image.

   - Increase the version number in the [CHANGELOG_BASE_IMAGES.md](CHANGELOG_BASE_IMAGES.md) file.
   - Increase the version number in the [README.md](README.md) file, Section `Versions correspondence`.
   - Edit the [CHANGELOG.md](CHANGELOG.md) file to add a new line describing the `Cosmian Base Image` changes.
   - Replace all old version numbers of the Cosmian Base VM image in:
     - `aws_base_image.yml`, `azure_base_image.yml` and `gcp_base_image.yml`
     - `nightly_aws_main.yml`, `nightly_azure_main.yml` and `nightly_gcp_main.yml`
   - Commit the changes: `git commit -am "chore: create new Cosmian Base Image version" && git push`
   - Eventually discard the pipelines related to the last commit.
   - Run manually the workflow [Manual base images recreation](https://github.com/Cosmian/cosmian_vm/actions/workflows/base_images_marketplaces.yml)
     - Run workflow on branch `release/X.Y.Z`

3. Release the Cosmian VM images.

- Increase the version number in the [CHANGELOG.md](CHANGELOG.md) file.
- Increase the version number in the [README.md](README.md) file, Section `Versions correspondence`.
- Increase the version number in the [cargo.toml root file](Cargo.toml).
- Do a cargo build a the root of the project to update the Cargo.lock file.
- Replace everywhere old version numbers of the Cosmian VM image.
- Once the CI pipeline is green, merge it into the `main` branch.
- Next, create a final tag to launch the release pipeline which will publish the Cosmian VM images on the marketplaces:
  - `git tag X.Y.Z -m "fix: this is why we did a new version"`
  - `git push --tags`
