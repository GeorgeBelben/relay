#!/usr/bin/env bash
# Runs ON the Ubuntu device (via `ssh relay@relay.local 'bash -s' < this file`).
# Pulls the latest successful build-dev.yml artifact from GitHub and installs it.
set -euo pipefail

REPO="GeorgeBelben/relay"
WORKFLOW="build-dev.yml"
DEST_DIR="/opt/relay/incoming"

echo "Finding latest successful $WORKFLOW run..."
RUN_ID=$(gh run list --repo "$REPO" --workflow "$WORKFLOW" --status success --limit 1 --json databaseId --jq '.[0].databaseId')

if [ -z "$RUN_ID" ]; then
  echo "No successful $WORKFLOW run found." >&2
  exit 1
fi

echo "Downloading artifact from run $RUN_ID..."
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
gh run download "$RUN_ID" --repo "$REPO" --name relay-deb --dir "$TMP_DIR"

DEB_FILE=$(find "$TMP_DIR" -name '*.deb' | head -1)
if [ -z "$DEB_FILE" ]; then
  echo "No .deb file found in artifact." >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
rm -f "$DEST_DIR"/*.deb
cp "$DEB_FILE" "$DEST_DIR/"
FINAL_DEB="$DEST_DIR/$(basename "$DEB_FILE")"

echo "Installing $FINAL_DEB..."
sudo /usr/local/sbin/relay-install-deb.sh

echo "Done."
