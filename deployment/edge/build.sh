#!/usr/bin/env bash
# Build the 'edge' deployment profile.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"
source deployment/_common/build_common.sh
out="target/deploy/edge"
rm -rf "$out"
mkdir -p "$out"
pkg_stage "edge" "$out"
pkg_write_manifest "$out"
log "edge package ready at $out"
