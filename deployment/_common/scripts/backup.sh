#!/usr/bin/env bash
# Backup the SQLite persistence file (baseline). Emits a manifest with
# a sha256 digest so restore can verify integrity.
set -euo pipefail
: "${AEON_HOME:=$HOME/.aeon}"
out=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done
[ -z "$out" ] && { echo "--out FILE required" >&2; exit 2; }
db="$AEON_HOME/data/aeon.sqlite"
[ -f "$db" ] || { echo "backup: no db at $db" >&2; exit 3; }
mkdir -p "$(dirname "$out")"
sqlite3 "$db" ".backup '$out'"
digest="$(sha256sum "$out" | awk '{print $1}')"
cat > "$out.manifest.json" <<EOF
{
  "path": "$(basename "$out")",
  "sha256": "$digest",
  "captured_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "source": "$db",
  "aeon_version": "$(cat "$AEON_HOME/VERSION" 2>/dev/null || echo unknown)",
  "aeon_profile": "$(cat "$AEON_HOME/PROFILE" 2>/dev/null || echo unknown)"
}
EOF
echo "backup: $out ($digest)"
