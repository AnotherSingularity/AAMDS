#!/usr/bin/env bash
# Remove an Aeon installation from $AEON_HOME.
set -euo pipefail
: "${AEON_HOME:=$HOME/.aeon}"
force=0
for a in "$@"; do case "$a" in --force) force=1 ;; esac; done

if [ ! -e "$AEON_HOME/VERSION" ]; then
  echo "uninstall: nothing to remove at $AEON_HOME" >&2
  exit 0
fi

# Preserve persistence file unless --force.
if [ -f "$AEON_HOME/data/aeon.sqlite" ] && [ "$force" -ne 1 ]; then
  bkup="$AEON_HOME.data.$(date -u +%Y%m%dT%H%M%SZ)"
  mkdir -p "$bkup"
  mv "$AEON_HOME/data" "$bkup/"
  echo "uninstall: preserved data at $bkup"
fi

rm -rf "$AEON_HOME"
echo "uninstall: removed $AEON_HOME"
