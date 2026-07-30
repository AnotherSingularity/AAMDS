#!/usr/bin/env python3
"""RC2 Gate 10 evidence generator (raw-first).

For each tool:
  1. Execute the pinned tool.
  2. Preserve raw stdout + stderr.
  3. Record exit status.
  4. Parse normalized results.
  5. Emit summary JSON.

Outputs live under build/evidence/rc2/gate-10/. Every emitted file
carries the current source_commit and generation_command so the
consistency validator can catch tampering / drift.
"""
from __future__ import annotations
import argparse
import glob
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from datetime import datetime, timezone

REPO = Path(__file__).resolve().parents[2]
OUT  = REPO / "build" / "evidence" / "rc2" / "gate-10"
SBOM_DIR = OUT / "sbom"


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def head() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def load_lock() -> dict:
    return json.loads((REPO / "security" / "toolchain.lock").read_text())


def _add_tool_path(env: dict) -> dict:
    env["PATH"] = f"{REPO}/.aeon-tools/bin:{env.get('PATH','')}"
    return env


def step_toolchain_versions() -> int:
    """Emit toolchain-versions.json from the lock + observed executables.
    Fails on any expected-vs-installed mismatch."""
    lock = load_lock()
    env = _add_tool_path(os.environ.copy())
    tools = []
    ok = True
    for entry in lock["tools"]:
        name = entry["name"]
        want = entry["version"]
        got = ""
        path = shutil.which(name)
        try:
            if name in ("cargo-audit", "cargo-cyclonedx"):
                sub = name[len("cargo-"):]
                r = subprocess.run(["cargo", sub, "--version"], env=env, capture_output=True, text=True)
                got = (r.stdout or r.stderr).strip().split()[-1].lstrip("v")
            elif name == "gitleaks":
                r = subprocess.run(["gitleaks", "version"], env=env, capture_output=True, text=True)
                got = (r.stdout or r.stderr).strip().split()[-1].lstrip("v")
            elif name == "rustfmt":
                r = subprocess.run(["rustfmt", "--version"], capture_output=True, text=True)
                got = r.stdout.split()[1] if r.stdout else ""
            elif name == "clippy":
                r = subprocess.run(["cargo", "clippy", "--version"], capture_output=True, text=True)
                got = r.stdout.split()[1] if r.stdout else ""
        except Exception as e:
            got = f"error:{e}"
        status = "ok"
        if want == "from-rust-toolchain":
            pass  # tracks toolchain — just record
        elif not got:
            status = "missing"; ok = False
        elif got != want:
            status = "version_mismatch"; ok = False
        tools.append({
            "name": name, "expected_version": want,
            "installed_version": got, "path": path or "", "status": status,
        })
    report = {
        "generated_at": now(),
        "source_commit": head(),
        "generation_command": "tools/gate10/run.py --step toolchain",
        "toolchain_lock": "security/toolchain.lock",
        "tools": tools,
        "result": "PASS" if ok else "FAIL",
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "toolchain-versions.json").write_text(json.dumps(report, indent=2, sort_keys=True))
    return 0 if ok else 1


def step_dependency_audit() -> int:
    env = _add_tool_path(os.environ.copy())
    OUT.mkdir(parents=True, exist_ok=True)
    raw_out = OUT / "dependency-audit.raw.stdout"
    raw_err = OUT / "dependency-audit.raw.stderr"
    with open(raw_out, "w") as so, open(raw_err, "w") as se:
        r = subprocess.run(["cargo", "audit", "--json"], env=env, cwd=REPO,
                           stdout=so, stderr=se)
    exit_code = r.returncode
    # cargo-audit exits 0 with findings=0, non-zero when it has findings.
    try:
        raw = json.loads(raw_out.read_text())
        vulns = (raw.get("vulnerabilities") or {}).get("list") or []
    except Exception:
        raw = None
        vulns = []
    # Dispositioning
    dispositions_path = REPO / "cybersecurity" / "vulnerability-dispositions.json"
    disposition_result = subprocess.run(
        [sys.executable, "tools/security/check-dispositions.py",
         "--audit", str(raw_out), "--ledger", str(dispositions_path),
         "--out", str(OUT / "dependency-audit-dispositioned.json")],
        cwd=REPO, capture_output=True, text=True,
    )
    normalized = {
        "generated_at": now(),
        "source_commit": head(),
        "generation_command": "tools/gate10/run.py --step dependency_audit",
        "raw_stdout": str(raw_out.relative_to(REPO)),
        "raw_stderr": str(raw_err.relative_to(REPO)),
        "raw_exit_code": exit_code,
        "vulnerabilities_count": len(vulns),
        "dispositions_result": "PASS" if disposition_result.returncode == 0 else "FAIL",
        "result": "PASS" if disposition_result.returncode == 0 else "FAIL",
    }
    (OUT / "dependency-audit.json").write_text(json.dumps(normalized, indent=2, sort_keys=True))
    return 0 if normalized["result"] == "PASS" else 1


def step_secret_scan() -> int:
    env = _add_tool_path(os.environ.copy())
    OUT.mkdir(parents=True, exist_ok=True)
    raw_out = OUT / "secret-scan.raw.stdout"
    raw_err = OUT / "secret-scan.raw.stderr"
    report_json = OUT / "secret-scan.raw.json"
    with open(raw_out, "w") as so, open(raw_err, "w") as se:
        r = subprocess.run([
            "gitleaks", "detect", "--no-banner", "--redact",
            "--report-format", "json",
            "--report-path", str(report_json),
            "--config", "cybersecurity/gitleaks.toml",
        ], env=env, cwd=REPO, stdout=so, stderr=se)
    # gitleaks: 0 clean, 1 findings, 2 error.
    exit_code = r.returncode
    findings = []
    if report_json.exists():
        try:
            findings = json.loads(report_json.read_text())
        except Exception:
            findings = []
    result = "PASS" if exit_code == 0 else ("FAIL" if exit_code == 1 else "FAIL")
    normalized = {
        "generated_at": now(),
        "source_commit": head(),
        "generation_command": "tools/gate10/run.py --step secret_scan",
        "raw_stdout": str(raw_out.relative_to(REPO)),
        "raw_stderr": str(raw_err.relative_to(REPO)),
        "raw_json": str(report_json.relative_to(REPO)),
        "raw_exit_code": exit_code,
        "findings_count": len(findings) if isinstance(findings, list) else 0,
        "result": result,
    }
    (OUT / "secret-scan.json").write_text(json.dumps(normalized, indent=2, sort_keys=True))
    return 0 if result == "PASS" else 1


def step_sbom() -> int:
    env = _add_tool_path(os.environ.copy())
    OUT.mkdir(parents=True, exist_ok=True)
    SBOM_DIR.mkdir(parents=True, exist_ok=True)
    # Ensure a clean working area for cargo-cyclonedx.
    for f in glob.glob(str(REPO / "*" / "*.cdx.json")):
        Path(f).unlink()
    raw_err = OUT / "sbom.raw.stderr"
    with open(raw_err, "w") as se:
        r = subprocess.run(["cargo", "cyclonedx", "--format", "json"],
                           env=env, cwd=REPO, stdout=subprocess.DEVNULL, stderr=se)
    exit_code = r.returncode
    # Relocate + validate SBOMs.
    for f in glob.glob(str(REPO / "*" / "*.cdx.json")):
        p = Path(f)
        rel = p.parent.name
        dest = SBOM_DIR / f"{rel}__{p.name}"
        p.replace(dest)
    entries = []
    valid = 0
    invalid = 0
    for f in sorted(SBOM_DIR.glob("*.cdx.json")):
        try:
            b = json.loads(f.read_text())
            fmt = b.get("bomFormat") == "CycloneDX"
            spec = bool(b.get("specVersion"))
            comps = len(b.get("components") or [])
            ok = fmt and spec and comps > 0
            entries.append({
                "path": str(f.relative_to(REPO)),
                "bomFormat": b.get("bomFormat"),
                "specVersion": b.get("specVersion"),
                "component_count": comps,
                "valid": ok,
            })
            valid += ok
            invalid += (not ok)
        except Exception as e:
            entries.append({"path": str(f.relative_to(REPO)), "valid": False, "error": str(e)})
            invalid += 1
    result = "PASS" if exit_code == 0 and invalid == 0 and valid > 0 else "FAIL"
    (OUT / "sbom-index.json").write_text(json.dumps({
        "generated_at": now(),
        "source_commit": head(),
        "generation_command": "tools/gate10/run.py --step sbom",
        "raw_exit_code": exit_code,
        "summary": {"total": len(entries), "valid": valid, "invalid": invalid},
        "sboms": entries,
        "result": result,
    }, indent=2, sort_keys=True))
    return 0 if result == "PASS" else 1


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--step", required=True,
                    choices=["toolchain", "dependency_audit", "secret_scan", "sbom", "all"])
    args = ap.parse_args()
    rc = 0
    if args.step in ("toolchain", "all"):    rc = step_toolchain_versions() or rc
    if args.step in ("dependency_audit", "all"): rc = step_dependency_audit() or rc
    if args.step in ("secret_scan", "all"):  rc = step_secret_scan() or rc
    if args.step in ("sbom", "all"):         rc = step_sbom() or rc
    return rc


if __name__ == "__main__":
    sys.exit(main())
