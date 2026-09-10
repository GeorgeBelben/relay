#!/usr/bin/env bash
# Deploys the latest relay-core + relay-cli build (from GitHub Actions) to relay.local.
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
CORE_ARTIFACT_NAME="relay-core-binary"
CLI_ARTIFACT_NAME="relay-cli-binary"
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

# Uploads a binary under a temp name and renames it into place on the device,
# rather than overwriting the target directly -- relay-core may be running off
# the old binary while this runs, and Linux refuses to open() a running
# executable for writing (ETXTBSY). rename() has no such restriction: a
# running process keeps its old (now-unlinked) inode until it exits, and the
# next `restart` execs the new one at that path. Harmless, and just as safe,
# for relay-cli even though it's never long-running.
install_binary() {
  local local_path="$1" remote_name="$2"
  scp -q "$local_path" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_BIN_DIR/$remote_name.new"
  remote "mv $REMOTE_BIN_DIR/$remote_name.new $REMOTE_BIN_DIR/$remote_name"
}

deploy_run() {
  local run_id="$1"
  local tmp_dir
  tmp_dir="$(mktemp -d)"

  echo "Downloading artifacts from run $run_id..."
  gh run download "$run_id" -n "$CORE_ARTIFACT_NAME" -D "$tmp_dir"
  gh run download "$run_id" -n "$CLI_ARTIFACT_NAME" -D "$tmp_dir"
  chmod +x "$tmp_dir/relay-core" "$tmp_dir/relay-cli"

  echo "Installing systemd --user unit..."
  remote "mkdir -p $REMOTE_BIN_DIR $REMOTE_UNIT_DIR"
  scp -q "$LOCAL_UNIT_FILE" "$REMOTE_USER@$REMOTE_HOST:$REMOTE_UNIT_DIR/$SERVICE_NAME.service"

  echo "Copying binaries to $REMOTE_HOST..."
  install_binary "$tmp_dir/relay-core" relay-core
  install_binary "$tmp_dir/relay-cli" relay-cli

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
