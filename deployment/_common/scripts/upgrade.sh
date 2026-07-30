#!/usr/bin/env bash
# Upgrade an Aeon installation to a newer package.
#
# Usage: upgrade.sh --from AEON_HOME --to NEW_PACKAGE_DIR
#
# Records a "before" state digest (VERSION + PROFILE + data digest),
# copies the new artifacts atop AEON_HOME, verifies the new manifest,
# and records an "after" state digest.

set -euo pipefail
from=""
to=""
while [ $# -gt 0 ]; do
  case "$1" in
    --from) from="$2"; shift 2 ;;
    --to)   to="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done
[ -z "$from" ] && { echo "--from AEON_HOME required" >&2; exit 2; }
[ -z "$to" ]   && { echo "--to NEW_PACKAGE_DIR required" >&2; exit 2; }

if [ ! -e "$from/VERSION" ]; then
  echo "upgrade: no existing installation at $from" >&2; exit 3
fi
if [ ! -f "$to/manifest.sha256" ]; then
  echo "upgrade: package $to has no manifest" >&2; exit 4
fi

# 1. verify new package integrity
(cd "$to" && sha256sum -c manifest.sha256 --quiet) \
  || { echo "upgrade: new package manifest.sha256 verify FAILED" >&2; exit 5; }

# 2. record before-state
before_state="$from/UPGRADE_BEFORE.json"
sqlite_digest="none"
if [ -f "$from/data/aeon.sqlite" ]; then
  sqlite_digest="$(sha256sum "$from/data/aeon.sqlite" | awk '{print $1}')"
fi
cat > "$before_state" <<EOF
{
  "captured_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "prior_version": "$(cat "$from/VERSION" 2>/dev/null || echo unknown)",
  "prior_commit":  "$(cat "$from/COMMIT"  2>/dev/null || echo unknown)",
  "sqlite_sha256": "$sqlite_digest"
}
EOF

# 3. copy new artifacts (do NOT touch data/)
for d in bin ui scripts docs sbom; do
  if [ -d "$to/$d" ]; then
    rm -rf "$from/$d"
    cp -a "$to/$d" "$from/"
  fi
done
# Config: keep existing files (operator may have customized), but drop
# in any file that does not already exist so new required knobs appear.
if [ -d "$to/config" ]; then
  mkdir -p "$from/config"
  for f in "$to/config"/*; do
    b="$(basename "$f")"
    [ -e "$from/config/$b" ] || cp -a "$f" "$from/config/$b"
  done
fi
for f in PROFILE COMMIT VERSION package.manifest.json manifest.sha256; do
  [ -f "$to/$f" ] && cp -a "$to/$f" "$from/"
done

# 4. record after-state
cat > "$from/UPGRADE_AFTER.json" <<EOF
{
  "upgraded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "new_version": "$(cat "$from/VERSION" 2>/dev/null || echo unknown)",
  "new_commit":  "$(cat "$from/COMMIT"  2>/dev/null || echo unknown)",
  "sqlite_sha256_preserved": "$sqlite_digest"
}
EOF

echo "upgrade: $from is now $(cat "$from/VERSION")"
