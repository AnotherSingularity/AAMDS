#!/usr/bin/env bash
# Install an Aeon deployment package into $AEON_HOME.
#
# Usage: install.sh [--home DIR] [--force]
#
# Steps:
#   1. Refuse if $AEON_HOME already contains an installation, unless --force.
#   2. Verify manifest.sha256 for every artifact in the package.
#   3. Copy artifacts into $AEON_HOME.
#   4. Copy default config, scripts, docs, sbom.
#   5. Emit $AEON_HOME/INSTALL_EVIDENCE.json.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${AEON_HOME:=$HOME/.aeon}"
force=0
while [ $# -gt 0 ]; do
  case "$1" in
    --home)  AEON_HOME="$2"; shift 2 ;;
    --force) force=1; shift ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

log() { printf '[install] %s\n' "$*"; }

if [ -e "$AEON_HOME/VERSION" ] && [ "$force" -ne 1 ]; then
  echo "install: $AEON_HOME already contains an installation. Use --force to overwrite." >&2
  exit 3
fi

# 1. verify manifest.sha256
(cd "$here" && sha256sum -c manifest.sha256 --quiet) \
  || { echo "install: manifest.sha256 verification FAILED" >&2; exit 4; }

# 2. install
mkdir -p "$AEON_HOME"
for d in bin ui config scripts docs sbom; do
  [ -d "$here/$d" ] && cp -a "$here/$d" "$AEON_HOME/"
done
for f in PROFILE COMMIT VERSION package.manifest.json manifest.sha256; do
  [ -f "$here/$f" ] && cp -a "$here/$f" "$AEON_HOME/"
done

# 3. evidence
cat > "$AEON_HOME/INSTALL_EVIDENCE.json" <<EOF
{
  "installed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "package_source": "$here",
  "aeon_home": "$AEON_HOME",
  "profile": "$(cat "$here/PROFILE" 2>/dev/null || echo unknown)",
  "commit":  "$(cat "$here/COMMIT" 2>/dev/null || echo unknown)",
  "version": "$(cat "$here/VERSION" 2>/dev/null || echo unknown)"
}
EOF

log "installed to $AEON_HOME"
