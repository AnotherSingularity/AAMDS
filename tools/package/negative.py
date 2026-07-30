#!/usr/bin/env python3
"""RC2-C package-tamper adversarial suite.

Builds a developer package, runs a suite of tamper mutations against
copies of it, and asserts the RC2 verifier rejects every one. Also
proves the RC2 installer refuses to copy before verify has passed.

Twenty-one cases (RC2-C §4.7, extended):
  1  modified PROFILE
  2  modified COMMIT
  3  modified VERSION
  4  modified package.manifest.v2.json (unsigned edit)
  5  modified executable (bin/aeon-operator-api)
  6  modified installer script (scripts/install.sh)
  7  modified upgrade script (scripts/upgrade.sh)
  8  modified SBOM contents
  9  added unlisted executable
 10  added unlisted configuration
 11  removed listed file
 12  duplicate manifest path
 13  absolute manifest path
 14  parent-directory traversal
 15  symlink escape
 16  invalid signature (bit-flipped signature_hex)
 17  wrong signer (different HMAC key)
 18  wrong profile in manifest (mismatched with declared)
 19  wrong source_commit (all zeros)
 20  install without pre-verify — installer must refuse
 21  ensure a clean baseline package still verifies (positive control)
"""
from __future__ import annotations
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BUILDER = REPO / "tools" / "package" / "build.py"
VERIFIER = REPO / "tools" / "package" / "verify.py"
INSTALLER = REPO / "tools" / "package" / "install.py"
EVID = REPO / "build" / "evidence" / "rc2" / "packages"


def build_baseline(out_root: Path) -> Path:
    subprocess.check_call(
        [sys.executable, str(BUILDER), "--profile", "developer", "--out", str(out_root)],
        cwd=REPO,
    )
    return out_root / "developer"


def verify(pkg: Path) -> int:
    return subprocess.call([sys.executable, str(VERIFIER), "--pkg", str(pkg)])


def copy_pkg(src: Path, sbox: Path, name: str) -> Path:
    dst = sbox / name
    shutil.copytree(src, dst)
    return dst


def flip(path: Path) -> None:
    b = bytearray(path.read_bytes())
    if not b:
        b = bytearray(b"x")
    b[0] ^= 0x01
    path.write_bytes(bytes(b))


def result(name: str, rc: int, expect_nonzero: bool) -> tuple[bool, dict]:
    ok = (rc != 0) if expect_nonzero else (rc == 0)
    return ok, {"case": name, "verifier_rc": rc, "expected": "nonzero" if expect_nonzero else "zero", "result": "PASS" if ok else "FAIL"}


def main() -> int:
    sbox = Path(tempfile.mkdtemp(prefix="rc2c-neg-"))
    try:
        base = build_baseline(sbox / "base")
        cases_report = []

        # 21. positive control
        rc = verify(base)
        _, e = result("21_baseline_positive_control", rc, expect_nonzero=False)
        cases_report.append(e)

        mutations = [
            ("01_modified_PROFILE",   lambda p: flip(p / "PROFILE")),
            ("02_modified_COMMIT",    lambda p: flip(p / "COMMIT")),
            ("03_modified_VERSION",   lambda p: flip(p / "VERSION")),
            ("04_modified_manifest",  lambda p: (p / "package.manifest.v2.json").write_text(
                (p / "package.manifest.v2.json").read_text().replace("developer", "eDvelopar"))),
            ("05_modified_executable",lambda p: flip(next((p/"bin").iterdir()))),
            ("06_modified_installer", lambda p: flip(p/"scripts"/"install.sh")),
            ("07_modified_upgrade",   lambda p: flip(p/"scripts"/"upgrade.sh")),
            ("08_modified_sbom",      lambda p: flip(next(iter((p/"sbom").glob("*.cdx.json"))))),
            ("09_add_unlisted_exec",  lambda p: (p/"bin"/"backdoor").write_text("#!/bin/sh\nexit 0\n")),
            ("10_add_unlisted_cfg",   lambda p: (p/"config"/"secret.json").write_text("{}")),
            ("11_remove_listed_file", lambda p: (p/"VERSION").unlink()),
            ("15_symlink_escape",     lambda p: (p/"bin"/"lnk").symlink_to("/etc/passwd")),
            ("16_invalid_signature",  lambda p: mutate_signature(p, "flip")),
            ("17_wrong_signer",       lambda p: mutate_signature(p, "wrong-key")),
            ("18_wrong_profile",      lambda p: mutate_manifest_field(p, "profile", "edge")),
            ("19_wrong_commit",       lambda p: mutate_manifest_field(p, "source_commit", "0"*40)),
        ]
        for name, mut in mutations:
            pkg = copy_pkg(base, sbox, name)
            try:
                mut(pkg)
            except Exception as e:
                cases_report.append({"case": name, "result": "FAIL", "detail": f"mutation error: {e}"})
                continue
            rc = verify(pkg)
            _, e = result(name, rc, expect_nonzero=True)
            cases_report.append(e)

        # 12: duplicate path — mutate the manifest's files list to include a duplicate
        pkg = copy_pkg(base, sbox, "12_duplicate_path")
        j = json.loads((pkg/"package.manifest.v2.json").read_text())
        j["files"].append(j["files"][0])
        (pkg/"package.manifest.v2.json").write_text(json.dumps(j, indent=2, sort_keys=True))
        _, e = result("12_duplicate_path", verify(pkg), expect_nonzero=True); cases_report.append(e)

        # 13: absolute path
        pkg = copy_pkg(base, sbox, "13_absolute_path")
        j = json.loads((pkg/"package.manifest.v2.json").read_text())
        j["files"][0]["path"] = "/etc/passwd"
        (pkg/"package.manifest.v2.json").write_text(json.dumps(j, indent=2, sort_keys=True))
        _, e = result("13_absolute_path", verify(pkg), expect_nonzero=True); cases_report.append(e)

        # 14: parent-dir traversal
        pkg = copy_pkg(base, sbox, "14_parent_traversal")
        j = json.loads((pkg/"package.manifest.v2.json").read_text())
        j["files"][0]["path"] = "../etc/passwd"
        (pkg/"package.manifest.v2.json").write_text(json.dumps(j, indent=2, sort_keys=True))
        _, e = result("14_parent_traversal", verify(pkg), expect_nonzero=True); cases_report.append(e)

        # 20: installer without pre-verify (install a tampered pkg, expect refusal)
        pkg = copy_pkg(base, sbox, "20_installer_refuses_tampered")
        flip(pkg/"bin"/"aeon-operator-api")
        home = sbox / "aeon-home-20"
        rc_install = subprocess.call([sys.executable, str(INSTALLER),
                                      "--pkg", str(pkg), "--home", str(home)])
        _, e = result("20_installer_refuses_tampered", rc_install, expect_nonzero=True); cases_report.append(e)

        # Emit results
        EVID.mkdir(parents=True, exist_ok=True)
        out = {
            "source_commit": subprocess.check_output(["git","rev-parse","HEAD"], cwd=REPO, text=True).strip(),
            "cases": cases_report,
            "summary": {
                "total":   len(cases_report),
                "passed": sum(1 for c in cases_report if c["result"] == "PASS"),
                "failed": sum(1 for c in cases_report if c["result"] == "FAIL"),
            },
        }
        (EVID / "package-negative-tests.json").write_text(json.dumps(out, indent=2, sort_keys=True))
        for c in cases_report:
            mark = "\033[1;32mPASS\033[0m" if c["result"] == "PASS" else "\033[1;31mFAIL\033[0m"
            print(f"  {mark} {c['case']}")
        print(f"\n{out['summary']['passed']}/{out['summary']['total']} PASS")
        return 0 if out["summary"]["failed"] == 0 else 1
    finally:
        shutil.rmtree(sbox, ignore_errors=True)


def mutate_signature(pkg: Path, mode: str) -> None:
    p = pkg / "package.manifest.v2.json"
    j = json.loads(p.read_text())
    if mode == "flip":
        s = j["signature"]["signature_hex"]
        # flip nibble
        first = "1" if s[0] == "0" else "0"
        j["signature"]["signature_hex"] = first + s[1:]
    elif mode == "wrong-key":
        # re-sign with a different key so signature length is right but wrong.
        canonical = json.dumps({k: v for k, v in j.items() if k != "signature"},
                               sort_keys=True, separators=(",", ":"))
        digest = hashlib.sha256(canonical.encode()).hexdigest()
        j["signature"]["signature_hex"] = hashlib.sha256(("BAD-KEY" + digest).encode()).hexdigest()
    p.write_text(json.dumps(j, indent=2, sort_keys=True))


def mutate_manifest_field(pkg: Path, field: str, value: str) -> None:
    p = pkg / "package.manifest.v2.json"
    j = json.loads(p.read_text())
    j[field] = value
    p.write_text(json.dumps(j, indent=2, sort_keys=True))


if __name__ == "__main__":
    sys.exit(main())
