#!/usr/bin/env bash
# Health check: verify AEON_HOME contains a runnable installation and
# that its binaries can print --help (baseline, no HTTP round-trip).
# Extended check: hits the operator API if AEON_API_BASE is set.
set -euo pipefail
: "${AEON_HOME:=$HOME/.aeon}"
: "${AEON_API_BASE:=}"

need() { [ -e "$1" ] || { echo "healthcheck: missing $1" >&2; exit 1; }; }
need "$AEON_HOME/VERSION"
need "$AEON_HOME/PROFILE"
need "$AEON_HOME/bin/aeon-operator-api"
[ -x "$AEON_HOME/bin/aeon-operator-api" ] || { echo "aeon-operator-api not executable" >&2; exit 1; }

# manifest reverify
(cd "$AEON_HOME" && sha256sum -c manifest.sha256 --quiet) \
  || { echo "healthcheck: installed manifest.sha256 verification FAILED" >&2; exit 1; }

# API check (best-effort)
if [ -n "$AEON_API_BASE" ]; then
  curl -sSf "$AEON_API_BASE/api/v1/health" >/dev/null \
    || { echo "healthcheck: API /health unreachable at $AEON_API_BASE" >&2; exit 1; }
fi

cat <<EOF
{
  "aeon_home": "$AEON_HOME",
  "version": "$(cat "$AEON_HOME/VERSION")",
  "profile": "$(cat "$AEON_HOME/PROFILE")",
  "manifest_ok": true,
  "api_checked": ${AEON_API_BASE:+true}${AEON_API_BASE:-false}
}
EOF
