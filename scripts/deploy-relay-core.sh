#!/usr/bin/env bash
set -euo pipefail

WORKFLOW="build-relay-core.yml"
ARTIFACT_NAME="relay-core-binary"
REMOTE_HOST="relay.local"
REMOTE_USER="relay"   # change to your actual EliteDesk username
REMOTE_PATH="/opt/relay/relay-core"
TMP_DIR="$(mktemp -d)"

echo "Finding latest successful build..."
RUN_ID=$(gh run list --workflow="$WORKFLOW" --branch=main --status=success --limit=1 --json databaseId --jq '.[0].databaseId')

if [ -z "$RUN_ID" ]; then
  echo "No successful run found."
  exit 1
fi

echo "Downloading artifact from run $RUN_ID..."
gh run download "$RUN_ID" -n "$ARTIFACT_NAME" -D "$TMP_DIR"

chmod +x "$TMP_DIR/relay-core"

echo "Copying to $REMOTE_HOST..."
scp "$TMP_DIR/relay-core" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH"

echo "Restarting service..."
ssh "$REMOTE_USER@$REMOTE_HOST" "sudo systemctl restart relay-core"

rm -rf "$TMP_DIR"
echo "Done."