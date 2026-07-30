#!/usr/bin/env bash
# Build a reproducible deployment package for a given profile.
#
# Usage:
#   tools/deployment/build-package.sh <profile> [--out DIR] [--version STRING]
#
# Emits:
#   <out>/<profile>/bin/aeon-operator-api
#   <out>/<profile>/ui/index.html
#   <out>/<profile>/config/*.json
#   <out>/<profile>/scripts/{install,uninstall,upgrade,rollback,healthcheck,backup,restore}.sh
#   <out>/<profile>/docs/{operations,integration,security,architecture}/**
#   <out>/<profile>/sbom/*.cdx.json
#   <out>/<profile>/manifest.sha256              (per-artifact sha256sum)
#   <out>/<profile>/package.manifest.json        (schema-validated + signed with dev-hmac)
#   <out>/<profile>/PROFILE, COMMIT, VERSION

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"

profile="${1:-}"
if [ -z "$profile" ]; then
  echo "usage: $0 <profile> [--out DIR] [--version STRING]" >&2
  exit 2
fi
shift
out="target/deploy"
version="0.1.0"
while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="$2"; shift 2 ;;
    --version) version="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

case "$profile" in
  developer|edge|fixed-site|disconnected|data-center|private-cloud) ;;
  *) echo "unknown profile: $profile" >&2; exit 2 ;;
esac

log() { printf '\033[1;34m[pkg]\033[0m %s\n' "$*"; }

pkg="$out/$profile"
rm -rf "$pkg"
mkdir -p "$pkg"/{bin,ui,config,scripts,docs,sbom}

# 1. release binaries
log "building release binaries"
cargo build --release --workspace
for b in aeon-operator-api; do
  cp -a "target/release/$b" "$pkg/bin/"
done

# 2. UI
cp -a operator-interface/index.html "$pkg/ui/"

# 3. profile config
cp -a "deployment/$profile/config/." "$pkg/config/"

# 4. profile scripts
cp -a deployment/_common/scripts/. "$pkg/scripts/"
if [ -d "deployment/$profile/scripts" ]; then
  cp -a "deployment/$profile/scripts/." "$pkg/scripts/"
fi
chmod +x "$pkg/scripts/"*.sh 2>/dev/null || true

# 5. docs
for d in operations integration security architecture models verification; do
  [ -d "docs/$d" ] && cp -a "docs/$d" "$pkg/docs/"
done

# 6. SBOM
if [ -d "docs/evidence/gate-10/sbom" ]; then
  cp -a docs/evidence/gate-10/sbom/. "$pkg/sbom/" 2>/dev/null || true
fi

# 7. per-artifact digests
(cd "$pkg" && find . -type f -not -name manifest.sha256 -not -name package.manifest.json -print0 \
   | xargs -0 sha256sum) > "$pkg/manifest.sha256"

# 8. package.manifest.json (schema-validated, dev-hmac signed)
commit="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
ts="deterministic-build"
python3 - "$pkg" "$profile" "$version" "$commit" "$ts" <<'EOF'
import hashlib, json, os, sys, glob, subprocess
pkg, profile, version, commit, ts = sys.argv[1:]

artifacts = []
for root, _, files in os.walk(pkg):
    for f in sorted(files):
        if f in ("manifest.sha256","package.manifest.json"):
            continue
        p = os.path.join(root, f)
        rel = os.path.relpath(p, pkg)
        b = open(p,"rb").read()
        artifacts.append({
            "path": rel.replace(os.sep,"/"),
            "sha256": hashlib.sha256(b).hexdigest(),
            "size_bytes": len(b),
        })
artifacts.sort(key=lambda a: a["path"])

sbom_refs = [a["path"] for a in artifacts if a["path"].startswith("sbom/")]
cfg_refs  = [a["path"] for a in artifacts if a["path"].startswith("config/")]

manifest = {
    "product": "aeon-air-defense-information-layer",
    "package_profile": profile,
    "version": version,
    "source_commit": commit,
    "build_timestamp": ts,
    "supported_architecture": ["x86_64","aarch64"],
    "supported_os": ["linux"],
    "artifacts": artifacts,
    "sbom_references": sbom_refs,
    "configuration_schema_references": cfg_refs,
    "installation_procedure_version": "1.0",
    "upgrade_procedure_version": "1.0",
    "rollback_procedure_version": "1.0",
    "minimum_resources": {
        "cpu_cores": 1,
        "memory_mb":  512 if profile in ("developer","edge","disconnected") else 2048,
        "disk_mb":    512 if profile in ("developer","edge","disconnected") else 4096,
    },
    "required_privileges": ["non_root_user","bind_local_tcp_port"],
    "known_limitations": [
        "Baseline signing method is dev-hmac-sha256; sponsor deployments substitute kms-hsm.",
        "Vendor sensor adapters are not shipped; SDK + conformance harness are.",
    ],
    "signature": {},
}

# Canonical JSON for digest (sorted keys, no whitespace)
canonical = json.dumps(
    {k: v for k, v in manifest.items() if k != "signature"},
    sort_keys=True, separators=(",",":"))
digest = hashlib.sha256(canonical.encode()).hexdigest()

key = os.environ.get("AEON_DEV_HMAC_KEY","aeon-dev-signing-key-baseline")
sig = hashlib.sha256((key + digest).encode()).hexdigest()

manifest["signature"] = {
    "method": "dev-hmac-sha256",
    "key_id": "dev-hmac-baseline",
    "signed_manifest_digest": sig,
    "note": "Non-production dev signature. Replace with kms-hsm before deployment."
}

open(os.path.join(pkg,"package.manifest.json"),"w").write(
    json.dumps(manifest, indent=2, sort_keys=True))
open(os.path.join(pkg,"PROFILE"),"w").write(profile+"\n")
open(os.path.join(pkg,"COMMIT"),"w").write(commit+"\n")
open(os.path.join(pkg,"VERSION"),"w").write(version+"\n")
print("wrote", os.path.join(pkg,"package.manifest.json"))
EOF

# 9. schema-validate the manifest
python3 tools/deployment/validate-manifest.py \
  --manifest "$pkg/package.manifest.json" \
  --schema   deployment/schemas/package-manifest.schema.json

log "package built: $pkg"
