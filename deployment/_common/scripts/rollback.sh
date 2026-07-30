#!/usr/bin/env bash
# Roll back an Aeon installation to a prior package.
#
# Usage: rollback.sh --home AEON_HOME --to PACKAGE_DIR [--restore-backup FILE]
#
# Preconditions:
#   * PACKAGE_DIR is the earlier package that this installation is
#     being rolled back to.
#   * If schema changes make rollback data-unsafe, --restore-backup
#     FILE must be supplied and the file must be a compatible backup.
#
# This script does NOT auto-detect data-incompatible rollback; the
# operator is responsible for supplying the backup when required.

set -euo pipefail
home=""
to=""
restore=""
while [ $# -gt 0 ]; do
  case "$1" in
    --home)           home="$2"; shift 2 ;;
    --to)             to="$2"; shift 2 ;;
    --restore-backup) restore="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done
[ -z "$home" ] || [ -z "$to" ] && { echo "--home and --to required" >&2; exit 2; }

if [ ! -f "$to/manifest.sha256" ]; then
  echo "rollback: package $to has no manifest" >&2; exit 3
fi
(cd "$to" && sha256sum -c manifest.sha256 --quiet) \
  || { echo "rollback: target package manifest verify FAILED" >&2; exit 4; }

# Preserve the current data by default.
if [ -f "$home/data/aeon.sqlite" ]; then
  bkup="$home/data/pre-rollback.$(date -u +%Y%m%dT%H%M%SZ).sqlite"
  cp -a "$home/data/aeon.sqlite" "$bkup"
fi

for d in bin ui scripts docs sbom config; do
  if [ -d "$to/$d" ]; then
    rm -rf "$home/$d"
    cp -a "$to/$d" "$home/"
  fi
done
for f in PROFILE COMMIT VERSION package.manifest.json manifest.sha256; do
  [ -f "$to/$f" ] && cp -a "$to/$f" "$home/"
done

if [ -n "$restore" ]; then
  bash "$home/scripts/restore.sh" --home "$home" --from "$restore"
fi

cat > "$home/ROLLBACK.json" <<EOF
{
  "rolled_back_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "to_version": "$(cat "$home/VERSION")",
  "restored_backup": "${restore:-none}"
}
EOF

echo "rollback: $home is now $(cat "$home/VERSION")"
