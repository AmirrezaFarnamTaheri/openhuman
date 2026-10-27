#!/usr/bin/env bash
# CI must exercise a real native jail. Never turn sandbox enforcement off.
set -euo pipefail
if ! command -v bwrap >/dev/null 2>&1; then
  apt-get update
  apt-get install -y --no-install-recommends bubblewrap
fi
# Fail before compiling if the outer runner blocks nested namespaces.
bwrap --unshare-user --unshare-pid --unshare-net \
  --ro-bind / / --proc /proc --dev /dev -- /bin/true
