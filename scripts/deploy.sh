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
# gh workflow run prints the newly created run's URL on success; parse the run ID
# straight from that instead of polling `gh run list`, which can race and return a
# previous run if the new one hasn't registered yet.
TRIGGER_OUTPUT=$(gh workflow run "$WORKFLOW" --repo "$REPO" --ref "$BRANCH" 2>&1)
echo "$TRIGGER_OUTPUT"
RUN_ID=$(echo "$TRIGGER_OUTPUT" | grep -oE '/runs/[0-9]+' | grep -oE '[0-9]+' | tail -1)

if [ -z "$RUN_ID" ]; then
  echo "Could not parse run ID from trigger output; falling back to polling..."
  for _ in $(seq 1 15); do
    RUN_ID=$(gh run list --repo "$REPO" --workflow "$WORKFLOW" --branch "$BRANCH" --limit 1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || true)
    if [ -n "$RUN_ID" ]; then
      break
    fi
    sleep 2
  done
fi

if [ -z "$RUN_ID" ]; then
  echo "Timed out waiting for the workflow run to appear." >&2
  exit 1
fi

echo "Watching run $RUN_ID..."
gh run watch "$RUN_ID" --repo "$REPO" --exit-status

echo "Build succeeded. Installing on $DEVICE..."
ssh "$DEVICE" 'bash -s' < "$SCRIPT_DIR/update-device.sh"

echo "Deploy complete."
