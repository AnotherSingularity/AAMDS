#!/usr/bin/env bash
# Install the security utilities pinned in security/toolchain.lock.
#
# The script is idempotent: running it a second time is a no-op if the
# expected versions are already present. It writes installation
# evidence to docs/evidence/gate-10/TOOLCHAIN_VERSIONS.json.
#
# The script installs tools into $AEON_TOOL_DIR (default: repo-local
# .aeon-tools/bin) rather than globally where possible, so that a
# developer machine is not mutated by running verify.
#
# Usage:
#   tools/security/install-security-tools.sh [--check] [--offline]
#
#   --check     do not install; verify installed versions match the lock
#   --offline   skip any step that requires network access; fail-close
#               any tool that is not already present with a clear error

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$here"

check_only=0
offline=0
for a in "$@"; do
  case "$a" in
    --check)   check_only=1 ;;
    --offline) offline=1 ;;
    *) echo "unknown flag: $a" >&2; exit 2 ;;
  esac
done

: "${AEON_TOOL_DIR:=$here/.aeon-tools/bin}"
mkdir -p "$AEON_TOOL_DIR"
export PATH="$AEON_TOOL_DIR:$PATH"

log()  { printf '\033[1;34m[sec-tools]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[sec-tools]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[sec-tools]\033[0m %s\n' "$*" >&2; exit 1; }

# Extract a JSON value from toolchain.lock using python (no jq required).
lock_get() {
  local expr="$1"
  python3 - "$here/security/toolchain.lock" <<EOF
import json, sys
d=json.load(open(sys.argv[1]))
print($expr)
EOF
}

verify_version() {
  local name="$1" want="$2"
  local got
  case "$name" in
    cargo-audit|cargo-cyclonedx)
      # invoke as cargo subcommand; strips the "cargo-" prefix
      sub="${name#cargo-}"
      got="$( cargo "$sub" --version 2>/dev/null | awk '{print $NF}' | tr -d 'v' )" || return 2 ;;
    gitleaks)
      got="$( "$name" version 2>/dev/null | tr -d 'v' | awk '{print $NF}' )" || return 2 ;;
    rustfmt)
      got="$( rustfmt --version 2>/dev/null | awk '{print $2}' )" || return 2 ;;
    clippy)
      got="$( cargo clippy --version 2>/dev/null | awk '{print $2}' )" || return 2 ;;
    *) return 3 ;;
  esac
  if [ -z "$got" ]; then return 2; fi
  if [ "$want" = "from-rust-toolchain" ]; then
    log "  $name: got=$got (tracks rust-toolchain)"
    return 0
  fi
  if [ "$got" != "$want" ]; then
    warn "  $name: installed $got != required $want"
    return 4
  fi
  log "  $name: $got ✓"
}

install_cargo_tool() {
  local name="$1" version="$2"
  if [ "$offline" -eq 1 ]; then
    verify_version "$name" "$version" || die "$name absent + --offline set"
    return
  fi
  # cargo install honours --root for isolated install.
  cargo install --locked --version "$version" "$name" \
    --root "$here/.aeon-tools" 2>&1 | tail -5
}

install_gitleaks() {
  local version="$1"
  if command -v gitleaks >/dev/null 2>&1 && verify_version gitleaks "$version"; then
    return
  fi
  if [ "$offline" -eq 1 ]; then
    die "gitleaks absent + --offline set"
  fi
  local url arch os
  os="$(uname | tr '[:upper:]' '[:lower:]')"
  case "$(uname -m)" in
    x86_64|amd64) arch="x64" ;;
    aarch64|arm64) arch="arm64" ;;
    *) die "unsupported arch $(uname -m) for gitleaks download" ;;
  esac
  url="https://github.com/gitleaks/gitleaks/releases/download/v${version}/gitleaks_${version}_${os}_${arch}.tar.gz"
  log "downloading $url"
  local tmp
  tmp="$(mktemp -d)"
  curl -fsSL "$url" -o "$tmp/gl.tgz" || die "gitleaks download failed"
  tar -C "$tmp" -xzf "$tmp/gl.tgz"
  install -m 0755 "$tmp/gitleaks" "$AEON_TOOL_DIR/gitleaks"
  rm -rf "$tmp"
}

emit_evidence() {
  local out="docs/evidence/gate-10/TOOLCHAIN_VERSIONS.json"
  mkdir -p "$(dirname "$out")"
  python3 - "$here/security/toolchain.lock" "$out" <<'EOF'
import json, os, subprocess, sys, shutil, datetime
lock=json.load(open(sys.argv[1]))
out_path=sys.argv[2]
def which(n):
    return shutil.which(n) or ""
def ver(n):
    try:
        if n in ("cargo-audit","cargo-cyclonedx"):
            sub=n[len("cargo-"):]
            r=subprocess.run(["cargo",sub,"--version"],capture_output=True,text=True,check=False)
            return (r.stdout or r.stderr).strip().split()[-1].lstrip("v")
        if n=="gitleaks":
            r=subprocess.run([n,"version"],capture_output=True,text=True,check=False)
            return (r.stdout or r.stderr).strip().split()[-1].lstrip("v")
        if n=="rustfmt":
            r=subprocess.run(["rustfmt","--version"],capture_output=True,text=True,check=False)
            return r.stdout.split()[1] if r.stdout else ""
        if n=="clippy":
            r=subprocess.run(["cargo","clippy","--version"],capture_output=True,text=True,check=False)
            return r.stdout.split()[1] if r.stdout else ""
    except Exception as e:
        return f"error:{e}"
    return ""
report={
    "generated_at": "harness-controlled",
    "toolchain_lock": "security/toolchain.lock",
    "tools": []
}
for t in lock["tools"]:
    n=t["name"]; want=t["version"]
    got=ver(n)
    status="ok"
    if not got: status="missing"
    elif want!="from-rust-toolchain" and got!=want: status="version_mismatch"
    report["tools"].append({
        "name": n, "expected_version": want, "installed_version": got,
        "path": which(n if n!="clippy" else "cargo"),
        "status": status,
    })
open(out_path,"w").write(json.dumps(report, indent=2, sort_keys=True))
print("evidence:", out_path)
EOF
}

log "toolchain lock: security/toolchain.lock"
if [ "$check_only" -eq 0 ]; then
  # install cargo tools
  for tool_entry in "cargo-audit 0.22.2" "cargo-cyclonedx 0.5.7"; do
    read -r name version <<<"$tool_entry"
    if verify_version "$name" "$version" >/dev/null 2>&1; then
      log "  $name $version already present"
    else
      log "installing $name $version"
      install_cargo_tool "$name" "$version"
    fi
  done
  # rustup components
  for comp in rustfmt clippy; do
    if ! rustup component list --installed 2>/dev/null | grep -q "$comp"; then
      if [ "$offline" -eq 1 ]; then
        die "$comp missing + --offline"
      fi
      rustup component add "$comp"
    fi
  done
  # gitleaks
  install_gitleaks 8.28.0
fi

log "verifying versions match lock:"
overall=0
for tool_entry in "cargo-audit 0.22.2" "cargo-cyclonedx 0.5.7" "gitleaks 8.28.0" "rustfmt from-rust-toolchain" "clippy from-rust-toolchain"; do
  read -r name version <<<"$tool_entry"
  verify_version "$name" "$version" || overall=1
done

emit_evidence
if [ "$overall" -ne 0 ]; then
  die "one or more security tools failed version verification"
fi
log "all pinned security tools installed and version-verified"
