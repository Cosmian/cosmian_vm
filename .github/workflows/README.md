
# Github CI workflows

## GitHub Workflow: `release_main.yml`

This workflow is designed to automate the release process for the repository.

- **Name:** Release Cloud providers images
- **Trigger:**
  - Push to any tag
  - Manual dispatch
- **Jobs:**
  - **Build binaries:** Compile and build the project binaries.
  - **Azure:** Release images to Azure.
  - **GCP:** Release images to Google Cloud Platform.
  - **AWS:** Release images to Amazon Web Services.
  - **Release:** Create a new release and publish artifacts.

### Workflow Details

  The `release_main.yml` workflow is structured to handle the release process across multiple cloud providers. Here's a detailed breakdown:

- **Build binaries:**
  - **Name:** Build binaries
  - **Workflow:** `build_all.yml`
  - **Description:** This job compiles and builds the necessary project binaries.

- **Azure:**
  - **Name:** Release to Azure
  - **Workflow:** `release_azure_main.yml`
  - **Description:** This job handles the release of images to Microsoft Azure.

- **GCP:**
  - **Name:** Release to Google Cloud Platform
  - **Workflow:** `release_gcp_main.yml`
  - **Description:** This job manages the release of images to Google Cloud Platform.

- **AWS:**
  - **Name:** Release to Amazon Web Services
  - **Workflow:** `release_aws_main.yml`
  - **Description:** This job oversees the release of images to Amazon Web Services.

- **Release:**
  - **Name:** Release
  - **Condition:** Triggered when a tag is pushed
  - **Dependencies:**
    - `azure`
    - `gcp`
    - `aws`
  - **Workflow:** `github_release.yml`
  - **Description:** This job creates a new release and publishes the relevant artifacts.

  Each job inherits secrets for secure operations and ensures a streamlined release process across different cloud platforms.

## GitHub Workflow: `pull_request.yml`

This workflow is designed to automate the pull request process for the repository.
It only runs on pull requests and do a subset of `release_main.yml` workflow.
