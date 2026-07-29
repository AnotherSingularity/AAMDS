#!/usr/bin/env bash
# Shared bits used by every profile's build.sh.
#
# Usage: source deployment/_common/build_common.sh
#
# Provides: pkg_stage, pkg_write_manifest, log

set -euo pipefail

log() { printf '\033[1;34m[deploy]\033[0m %s\n' "$*"; }

# Assemble a staging directory containing the release binaries.
pkg_stage() {
  local profile="$1" outdir="$2"
  log "building release binaries"
  cargo build --release --workspace
  mkdir -p "$outdir/bin" "$outdir/config" "$outdir/ui" "$outdir/docs"
  for b in aeon-operator-api; do
    if [ -f "target/release/$b" ]; then
      cp -a "target/release/$b" "$outdir/bin/"
    fi
  done
  # UI is a single html file
  if [ -f "operator-interface/index.html" ]; then
    cp -a operator-interface/index.html "$outdir/ui/"
  fi
  # Minimal doc set — operator + integration
  for d in operations integration security architecture; do
    if [ -d "docs/$d" ]; then
      cp -a "docs/$d" "$outdir/docs/"
    fi
  done
  # Profile config
  if [ -d "deployment/$profile/config" ]; then
    cp -a deployment/$profile/config/. "$outdir/config/"
  fi
  echo "$profile" > "$outdir/PROFILE"
  git rev-parse HEAD > "$outdir/COMMIT" 2>/dev/null || echo unknown > "$outdir/COMMIT"
}

# Write a sha256 manifest of everything in the staging dir.
pkg_write_manifest() {
  local outdir="$1"
  (cd "$outdir" && find . -type f -not -name manifest.sha256 -print0 \
     | xargs -0 sha256sum) > "$outdir/manifest.sha256"
  log "manifest written to $outdir/manifest.sha256"
}
