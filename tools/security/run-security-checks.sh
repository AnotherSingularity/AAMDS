#!/usr/bin/env bash
# Unified `verify:security` driver.
#
# Runs the real pinned tools and fails on any mandatory-check failure.
# Missing tool → fail (never silent skip). Optionally --advisory turns
# specific steps into warnings for pre-CI local runs.
#
# Emits machine-readable evidence under docs/evidence/gate-10/.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"

: "${AEON_TOOL_DIR:=$here/.aeon-tools/bin}"
export PATH="$AEON_TOOL_DIR:$PATH"

out=docs/evidence/gate-10
mkdir -p "$out" "$out/sbom"

advisory=0
for a in "$@"; do
  case "$a" in
    --advisory) advisory=1 ;;
    *) echo "unknown flag: $a" >&2; exit 2 ;;
  esac
done

log()  { printf '\033[1;34m[verify:security]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[verify:security]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[verify:security]\033[0m %s\n' "$*" >&2; exit 1; }

require_tool() {
  local name="$1" cmd="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    if [ "$advisory" -eq 1 ]; then
      warn "$name not found — advisory mode, continuing"
      return 1
    fi
    die "$name ($cmd) not found. Install: tools/security/install-security-tools.sh"
  fi
}

# ---- Dependency audit ----
step_dep_audit() {
  log "dependency audit (cargo-audit)"
  if require_tool "cargo-audit" cargo-audit; then
    if cargo audit --json > "$out/DEPENDENCY_AUDIT.json" 2> "$out/DEPENDENCY_AUDIT.stderr.txt"; then
      log "  cargo audit: no vulnerabilities"
    else
      warn "  cargo audit reported findings — cross-referencing dispositions"
    fi
    if ! python3 tools/security/check-dispositions.py \
        --audit  "$out/DEPENDENCY_AUDIT.json" \
        --ledger cybersecurity/vulnerability-dispositions.json \
        --out    "$out/DEPENDENCY_AUDIT_DISPOSITIONED.json"; then
      [ "$advisory" -eq 1 ] || die "unresolved / expired vulnerabilities"
    fi
  fi
}

# ---- Secret scan ----
step_secret_scan() {
  log "secret scan (gitleaks)"
  if require_tool "gitleaks" gitleaks; then
    # gitleaks exit code: 0 = clean, 1 = findings, 2 = error
    set +e
    gitleaks detect --no-banner --redact \
      --report-format json \
      --report-path "$out/SECRET_SCAN.json" \
      --config cybersecurity/gitleaks.toml
    rc=$?
    set -e
    if [ "$rc" -eq 0 ]; then
      log "  gitleaks: no findings"
    elif [ "$rc" -eq 1 ]; then
      [ "$advisory" -eq 1 ] || die "gitleaks reported findings — see $out/SECRET_SCAN.json"
    else
      die "gitleaks failed with rc=$rc"
    fi
  fi
}

# ---- Static analysis (clippy + fmt) ----
step_static_analysis() {
  log "static analysis (rustfmt + clippy)"
  cargo fmt --all -- --check
  # capture clippy JSON for evidence
  set +e
  cargo clippy --workspace --all-targets --message-format=json \
    -- -D warnings \
    1> "$out/STATIC_ANALYSIS.raw.json" 2> "$out/STATIC_ANALYSIS.stderr.txt"
  rc=$?
  set -e
  # Also produce a human-readable pass/fail summary
  cat > "$out/STATIC_ANALYSIS.json" <<EOF
{"tool":"clippy","exit_code":$rc,"raw_output":"$out/STATIC_ANALYSIS.raw.json"}
EOF
  if [ "$rc" -ne 0 ]; then
    [ "$advisory" -eq 1 ] || die "clippy failed (exit $rc)"
  fi
  # unsafe-code inventory: repo forbids unsafe via #![forbid(unsafe_code)]
  grep -RIn 'unsafe' --include='*.rs' contracts core-runtime persistence \
    sensor-adapter-sdk normalization track-management ml-correction \
    secure-relay operator-api simulation \
    | grep -v 'forbid(unsafe_code)' > "$out/UNSAFE_CODE_INVENTORY.txt" || true
}

# ---- SBOM ----
step_sbom() {
  log "SBOM (cargo-cyclonedx)"
  if require_tool "cargo-cyclonedx" cargo-cyclonedx; then
    # Emits *.cdx.json per package; move them into evidence sbom dir.
    cargo cyclonedx --format json 1> "$out/SBOM.stdout.log" 2> "$out/SBOM.stderr.log"
    # Move produced files into evidence dir
    find . -maxdepth 3 -name '*.cdx.json' -not -path './target/*' \
      -not -path "./$out/*" -print0 \
    | while IFS= read -r -d '' f; do
        rel=$(dirname "$f" | sed 's|^\./||')
        base=$(basename "$f")
        dest="$out/sbom/${rel//\//__}__$base"
        mv -f "$f" "$dest"
      done
    python3 tools/security/validate-sboms.py "$out/sbom" > "$out/SBOM_INDEX.json"
  fi
}

# ---- Policy summary ----
step_policy_summary() {
  log "policy summary"
  python3 - <<PY > "$out/SECURITY_POLICY_RESULTS.json"
import json, os, glob, sys
def read(p, default={}):
    try: return json.load(open(p))
    except Exception: return default
out=r"$out"
res={
 "dependency_audit": read(os.path.join(out,"DEPENDENCY_AUDIT_DISPOSITIONED.json")).get("summary", {}),
 "secret_scan":      {"path": os.path.join(out,"SECRET_SCAN.json"),
                       "exists": os.path.exists(os.path.join(out,"SECRET_SCAN.json"))},
 "static_analysis":  read(os.path.join(out,"STATIC_ANALYSIS.json")),
 "sbom_index":       read(os.path.join(out,"SBOM_INDEX.json")).get("summary", {}),
 "toolchain":        read(os.path.join(out,"TOOLCHAIN_VERSIONS.json")).get("tools", []),
}
print(json.dumps(res, indent=2, sort_keys=True))
PY
}

step_dep_audit
step_secret_scan
step_static_analysis
step_sbom
step_policy_summary
log "verify:security complete — evidence in $out"
