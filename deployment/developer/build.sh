#!/usr/bin/env bash
# Build the 'developer' deployment profile.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"
source deployment/_common/build_common.sh
out="target/deploy/developer"
rm -rf "$out"
mkdir -p "$out"
pkg_stage "developer" "$out"
pkg_write_manifest "$out"
log "developer package ready at $out"
