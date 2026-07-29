#!/usr/bin/env bash
# Build the 'disconnected' deployment profile.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"
source deployment/_common/build_common.sh
out="target/deploy/disconnected"
rm -rf "$out"
mkdir -p "$out"
pkg_stage "disconnected" "$out"
pkg_write_manifest "$out"
log "disconnected package ready at $out"
