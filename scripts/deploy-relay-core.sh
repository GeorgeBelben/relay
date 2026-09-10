#!/usr/bin/env bash
# Deploys the latest relay-core build (from GitHub Actions) to relay.local.
#
# Runs relay-core as a systemd --user service on the device instead of a system
# service, specifically so this script never needs sudo: as long as
# `loginctl enable-linger relay` has been run once on the device (only sudo this
# ever needs), `systemctl --user` works over a plain SSH command with no
# interactive auth. See scripts/relay-core.service for the unit.
#
# Usage:
#   scripts/deploy-relay-core.sh           deploy the latest successful build once
#   scripts/deploy-relay-core.sh --watch   poll GitHub for new successful builds
#                                          and deploy each one automatically
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

WORKFLOW="build-relay-core.yml"
ARTIFACT_NAME="relay-core-binary"
REMOTE_HOST="relay.local"
REMOTE_USER="relay"
SERVICE_NAME="relay-core"
LOCAL_UNIT_FILE="scripts/relay-core.service"
STATE_FILE=".deploy-relay-core.state"
POLL_INTERVAL_SECS=30

remote() {
  ssh -o BatchMode=yes "$REMOTE_USER@$REMOTE_HOST" "$@"
}

# Resolve to an absolute path up front -- `~` isn't reliably expanded by scp's
# remote-path argument (depends on whether it's using the SFTP or SCP protocol),
# so every remote path below is built from this instead of relying on `~`.
REMOTE_HOME="$(remote 'echo $HOME')"
REMOTE_BIN_DIR="$REMOTE_HOME/.local/bin"
REMOTE_UNIT_DIR="$REMOTE_HOME/.config/systemd/user"

latest_successful_run() {
  gh run list --workflow="$WORKFLOW" --branch=main --status=success --limit=1 \
    --json databaseId --jq '.[0].databaseId // empty'
}

deploy_run() {
  local run_id="$1"
  local tmp_dir
  tmp_dir="$(mktemp -d)"

  echo "Downloading artifact from run $run_id..."
  gh run download "$run_id" -n "$ARTIFACT_NAME" -D "$tmp_dir"
  chmod +x "$tmp_dir/relay-core"

  echo "Installing systemd --user unit..."
  remote "mkdir -p $REMOTE_BIN_DIR $REMOTE_UNIT_DIR"
  scp -q "$LOCAL_UNIT_FILE" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_UNIT_DIR/$SERVICE_NAME.service"

  # Upload under a temp name and rename into place, rather than overwriting
  # relay-core directly -- the service is running off the old binary while
  # this runs, and Linux refuses to open() a running executable for writing
  # (ETXTBSY). rename() has no such restriction: the running process keeps
  # the old (now-unlinked) inode until it exits, and `restart` below execs
  # the new one at that path.
  echo "Copying binary to $REMOTE_HOST..."
  scp -q "$tmp_dir/relay-core" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_BIN_DIR/relay-core.new"
  remote "mv $REMOTE_BIN_DIR/relay-core.new $REMOTE_BIN_DIR/relay-core"

  echo "Restarting service..."
  remote "systemctl --user daemon-reload && systemctl --user enable --now $SERVICE_NAME && systemctl --user restart $SERVICE_NAME"

  rm -rf "$tmp_dir"

  echo "$run_id" > "$STATE_FILE"
  echo "Deployed run $run_id."
}

deploy_latest() {
  local run_id
  run_id="$(latest_successful_run)"
  if [[ -z "$run_id" ]]; then
    echo "No successful build-relay-core run found." >&2
    exit 1
  fi
  deploy_run "$run_id"
}

watch() {
  local last_deployed=""
  [[ -f "$STATE_FILE" ]] && last_deployed="$(cat "$STATE_FILE")"

  echo "Watching for new relay-core builds (checking every ${POLL_INTERVAL_SECS}s, Ctrl+C to stop)..."
  while true; do
    local run_id
    run_id="$(latest_successful_run || true)"
    if [[ -n "$run_id" && "$run_id" != "$last_deployed" ]]; then
      echo "New build detected: run $run_id"
      deploy_run "$run_id"
      last_deployed="$run_id"
    fi
    sleep "$POLL_INTERVAL_SECS"
  done
}

if [[ "${1:-}" == "--watch" ]]; then
  watch
else
  deploy_latest
fi
