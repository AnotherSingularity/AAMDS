#!/usr/bin/env bash
# Build the 'fixed-site' deployment profile.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"
source deployment/_common/build_common.sh
out="target/deploy/fixed-site"
rm -rf "$out"
mkdir -p "$out"
pkg_stage "fixed-site" "$out"
pkg_write_manifest "$out"
log "fixed-site package ready at $out"
