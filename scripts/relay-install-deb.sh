#!/usr/bin/env bash
# Installed on the device at /usr/local/sbin/relay-install-deb.sh, owned by root.
# Wrapping dpkg -i in a fixed-path script (rather than a sudoers wildcard rule)
# avoids the "wildcards are not allowed in command arguments" sudoers restriction.
set -euo pipefail

DEB=$(find /opt/relay/incoming -maxdepth 1 -name '*.deb' | head -1)
if [ -z "$DEB" ]; then
  echo "No .deb found in /opt/relay/incoming" >&2
  exit 1
fi

dpkg -i "$DEB"
