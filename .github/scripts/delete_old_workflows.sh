#!/usr/bin/env bash

# Delete all the runs for a given workflow
# Usage: delete-runs.sh <repository> <workflow-name>

set -xoe pipefail

REPOSITORY=$1
WORKFLOW_NAME=$2

# Validate arguments
if [[ -z "$REPOSITORY" ]]; then
  echo "Repository is required"
  exit 1
fi

if [[ -z "$WORKFLOW_NAME" ]]; then
  echo "Workflow name is required"
  exit 1
fi

echo "Getting all completed runs for workflow $WORKFLOW_NAME in $REPOSITORY"

RUNS=$(
  gh api \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "/repos/$REPOSITORY/actions/workflows/$WORKFLOW_NAME/runs" \
    --paginate \
    --jq '.workflow_runs[] | select(.conclusion != "") | .id'
)

echo "Found $(echo "$RUNS" | wc -l) completed runs for workflow $WORKFLOW_NAME"

for RUN_ID in $RUNS; do
  gh api \
  --method DELETE \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "/repos/$REPOSITORY/actions/runs/$RUN_ID" --silent || echo "Failed to delete run $RUN_ID"
  sleep 0.1
done
