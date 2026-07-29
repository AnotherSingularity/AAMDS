#!/usr/bin/env bash
# Build the 'data-center' deployment profile.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"
source deployment/_common/build_common.sh
out="target/deploy/data-center"
rm -rf "$out"
mkdir -p "$out"
pkg_stage "data-center" "$out"
pkg_write_manifest "$out"
log "data-center package ready at $out"
