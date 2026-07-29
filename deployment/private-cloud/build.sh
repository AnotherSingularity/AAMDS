#!/usr/bin/env bash
# Build the 'private-cloud' deployment profile.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"
source deployment/_common/build_common.sh
out="target/deploy/private-cloud"
rm -rf "$out"
mkdir -p "$out"
pkg_stage "private-cloud" "$out"
pkg_write_manifest "$out"
log "private-cloud package ready at $out"
