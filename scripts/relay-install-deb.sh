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

# `apt install ./file.deb` doesn't reliably resolve local-file dependencies on this
# system (apt reports them as "not going to be installed" despite valid candidates).
# dpkg -i followed by apt --fix-broken install is the standard robust recipe instead.
dpkg -i "$DEB" || true
apt-get install -f -y
