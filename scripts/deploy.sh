#!/usr/bin/env bash
# Runs on the dev Mac (via `bun run deploy`). Triggers the build-dev.yml workflow,
# waits for it to finish, then SSHes into the device to pull + install the result.
set -euo pipefail

REPO="GeorgeBelben/relay"
WORKFLOW="build-dev.yml"
DEVICE="relay@relay.local"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BRANCH=$(git rev-parse --abbrev-ref HEAD)

echo "Triggering $WORKFLOW on branch $BRANCH..."
gh workflow run "$WORKFLOW" --repo "$REPO" --ref "$BRANCH"

echo "Waiting for the run to register..."
RUN_ID=""
for _ in $(seq 1 15); do
  RUN_ID=$(gh run list --repo "$REPO" --workflow "$WORKFLOW" --branch "$BRANCH" --limit 1 --json databaseId,createdAt --jq '.[0].databaseId' 2>/dev/null || true)
  if [ -n "$RUN_ID" ]; then
    break
  fi
  sleep 2
done

if [ -z "$RUN_ID" ]; then
  echo "Timed out waiting for the workflow run to appear." >&2
  exit 1
fi

echo "Watching run $RUN_ID..."
gh run watch "$RUN_ID" --repo "$REPO" --exit-status

echo "Build succeeded. Installing on $DEVICE..."
ssh "$DEVICE" 'bash -s' < "$SCRIPT_DIR/update-device.sh"

echo "Deploy complete."
