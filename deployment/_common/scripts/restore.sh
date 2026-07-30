#!/usr/bin/env bash
# Restore a SQLite backup into AEON_HOME/data/aeon.sqlite.
# Rejects backups whose digest does not match the accompanying manifest.
set -euo pipefail
: "${AEON_HOME:=$HOME/.aeon}"
from=""
while [ $# -gt 0 ]; do
  case "$1" in
    --from) from="$2"; shift 2 ;;
    --home) AEON_HOME="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done
[ -z "$from" ] && { echo "--from FILE required" >&2; exit 2; }
[ -f "$from" ] || { echo "restore: $from missing" >&2; exit 3; }

manifest="$from.manifest.json"
if [ -f "$manifest" ]; then
  want="$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))['sha256'])" "$manifest")"
  got="$(sha256sum "$from" | awk '{print $1}')"
  if [ "$want" != "$got" ]; then
    echo "restore: digest mismatch (want $want, got $got) — REFUSED" >&2
    exit 4
  fi
fi

mkdir -p "$AEON_HOME/data"
cp -a "$from" "$AEON_HOME/data/aeon.sqlite"
echo "restore: $AEON_HOME/data/aeon.sqlite restored from $from"
