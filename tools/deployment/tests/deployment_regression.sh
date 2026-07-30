#!/usr/bin/env bash
# Negative-path regression suite for the deployment harness.
#
# Verifies that the harness fails closed under the conditions listed
# in directive section 15.

set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$here"

pass=0; fail=0
check() {
  local name="$1" want="$2" got="$3"
  if [ "$want" = "$got" ]; then
    printf '  \033[1;32mPASS\033[0m %s\n' "$name"; pass=$((pass+1))
  else
    printf '  \033[1;31mFAIL\033[0m %s (want %s got %s)\n' "$name" "$want" "$got"; fail=$((fail+1))
  fi
}

sbox="$(mktemp -d)"
./tools/deployment/build-package.sh developer --out "$sbox/pkg-a" --version 0.1.0 >/dev/null

echo "1) invalid package digest is rejected"
cp -r "$sbox/pkg-a/developer" "$sbox/tampered"
printf 'x' >> "$sbox/tampered/bin/aeon-operator-api"
rc=0
AEON_HOME="$sbox/home" "$sbox/tampered/scripts/install.sh" >/dev/null 2>&1 || rc=$?
check "install refuses tampered package" "4" "$rc"

echo "2) fresh install rejects a repeat install unless --force"
AEON_HOME="$sbox/home" "$sbox/pkg-a/developer/scripts/install.sh" >/dev/null
rc=0
AEON_HOME="$sbox/home" "$sbox/pkg-a/developer/scripts/install.sh" >/dev/null 2>&1 || rc=$?
check "double-install rejected without --force" "3" "$rc"

echo "3) restore rejects a backup with wrong digest"
mkdir -p "$sbox/home/data"
sqlite3 "$sbox/home/data/aeon.sqlite" "CREATE TABLE t(x INT); INSERT INTO t VALUES(1);"
AEON_HOME="$sbox/home" "$sbox/home/scripts/backup.sh" --out "$sbox/backup.sqlite" >/dev/null
printf 'x' >> "$sbox/backup.sqlite"
rc=0
AEON_HOME="$sbox/home" "$sbox/home/scripts/restore.sh" --from "$sbox/backup.sqlite" >/dev/null 2>&1 || rc=$?
check "restore rejects digest-mismatched backup" "4" "$rc"

echo "4) manifest schema-validator rejects an invalid manifest"
bad="$sbox/bad.manifest.json"
python3 -c "import json;json.dump({'product':'x'},open('$bad','w'))"
rc=0
python3 tools/deployment/validate-manifest.py \
  --manifest "$bad" --schema deployment/schemas/package-manifest.schema.json >/dev/null 2>&1 || rc=$?
check "invalid manifest rejected" "1" "$rc"

echo "5) toolchain checker fails-closed on wrong version"
# Temporarily replace the lock with one that expects an impossible
# version, run --check, then always restore the original.
cp security/toolchain.lock "$sbox/toolchain.lock.orig"
sed 's/"0.22.2"/"9.99.9"/' security/toolchain.lock > "$sbox/toolchain.lock.bad"
cp "$sbox/toolchain.lock.bad" security/toolchain.lock
rc=0
./tools/security/install-security-tools.sh --check >/dev/null 2>&1 || rc=$?
cp "$sbox/toolchain.lock.orig" security/toolchain.lock
check "toolchain --check fails on version mismatch" "1" "$rc"

echo "6) expired vulnerability disposition fails the check"
tmp_audit="$sbox/audit.json"
tmp_ledger="$sbox/ledger.json"
python3 - "$tmp_audit" "$tmp_ledger" <<'PY'
import json,sys
audit={"vulnerabilities":{"list":[{
  "advisory":{"id":"RUSTSEC-2099-0001"},
  "package":{"name":"fake","version":"1.0.0"}
}]}}
open(sys.argv[1],"w").write(json.dumps(audit))
ledger={"dispositions":[{
  "advisory":"RUSTSEC-2099-0001","package":"fake","affected_version":"1.0.0",
  "technical_impact":"none","reachability":"unreachable",
  "compensating_controls":"none","owner":"tester","state":"mitigated",
  "created":"2020-01-01","expires":"2020-06-01","required_remediation":"n/a"
}]}
open(sys.argv[2],"w").write(json.dumps(ledger))
PY
rc=0
python3 tools/security/check-dispositions.py --audit "$tmp_audit" --ledger "$tmp_ledger" --out "$sbox/out.json" >/dev/null 2>&1 || rc=$?
check "expired disposition rejected" "1" "$rc"

echo "7) uncovered vulnerability fails the check"
python3 - "$sbox/audit2.json" <<'PY'
import json,sys
audit={"vulnerabilities":{"list":[{
  "advisory":{"id":"RUSTSEC-2099-0002"},
  "package":{"name":"other","version":"1.0.0"}
}]}}
open(sys.argv[1],"w").write(json.dumps(audit))
PY
rc=0
python3 tools/security/check-dispositions.py --audit "$sbox/audit2.json" --ledger cybersecurity/vulnerability-dispositions.json --out "$sbox/out2.json" >/dev/null 2>&1 || rc=$?
check "uncovered advisory rejected" "1" "$rc"

echo "8) sbom validator rejects an empty CycloneDX file"
bad_sbom_dir="$sbox/sbom"
mkdir -p "$bad_sbom_dir"
echo '{"bomFormat":"CycloneDX","specVersion":"1.3","components":[]}' > "$bad_sbom_dir/x.cdx.json"
rc=0
python3 tools/security/validate-sboms.py "$bad_sbom_dir" >/dev/null 2>&1 || rc=$?
check "empty sbom rejected" "1" "$rc"

echo
echo "Result: $pass PASS, $fail FAIL"
if [ "$fail" -gt 0 ]; then exit 1; fi
