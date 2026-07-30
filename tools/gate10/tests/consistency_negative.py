#!/usr/bin/env python3
"""RC2-E negative-path regression: prove the Gate 10 consistency
validator rejects fixtures that reproduce the RC1 contradiction and
other summary/raw inconsistencies.

Uses isolated tempdirs; does NOT touch the real build/evidence tree.
"""
from __future__ import annotations
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
VALIDATOR = REPO / "tools" / "gate10" / "validate.py"


def with_temp_evidence(files: dict[str, dict], expect_rc_nonzero: bool) -> tuple[int, str]:
    """Populate build/evidence/rc2/gate-10/ with files, run the validator,
    then restore the real evidence from git."""
    out_dir = REPO / "build" / "evidence" / "rc2" / "gate-10"
    backup = Path(tempfile.mkdtemp(prefix="gate10-back-"))
    # Move existing content aside.
    if out_dir.exists():
        for p in out_dir.iterdir():
            (backup / p.name).mkdir(parents=True, exist_ok=True) if p.is_dir() else None
            p.rename(backup / p.name)
    try:
        out_dir.mkdir(parents=True, exist_ok=True)
        for name, content in files.items():
            (out_dir / name).write_text(json.dumps(content, indent=2, sort_keys=True))
        p = subprocess.run([sys.executable, str(VALIDATOR)], cwd=REPO,
                           capture_output=True, text=True)
        return p.returncode, p.stderr
    finally:
        # Wipe fixtures and restore.
        for p in list(out_dir.iterdir()):
            if p.is_dir():
                import shutil; shutil.rmtree(p)
            else:
                p.unlink()
        for p in backup.iterdir():
            p.rename(out_dir / p.name)
        import shutil; shutil.rmtree(backup, ignore_errors=True)


def sha() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def case_rc1_style_toolchain_contradiction() -> bool:
    """RC1 shipped an evidence file where cargo-audit expected=9.99.9,
    installed=0.22.2, status=version_mismatch — while the report claimed
    PASS. The RC2 validator MUST refuse to emit gate PASS on that state."""
    files = {
        "toolchain-versions.json": {
            "source_commit": sha(),
            "tools": [
                {"name": "cargo-audit", "expected_version": "9.99.9",
                 "installed_version": "0.22.2", "path": "/x", "status": "version_mismatch"}
            ],
            "result": "PASS",
        },
    }
    rc, err = with_temp_evidence(files, expect_rc_nonzero=True)
    ok = rc != 0 and "version_mismatch" in err
    print(f"  rc1_style_contradiction: rc={rc} {'PASS' if ok else 'FAIL'}")
    return ok


def case_sbom_index_lies_about_count() -> bool:
    """Index claims 5 SBOMs but the sbom/ dir contains 0."""
    files = {
        "toolchain-versions.json": {
            "source_commit": sha(),
            "tools": [],
            "result": "PASS",
        },
        "sbom-index.json": {
            "source_commit": sha(),
            "summary": {"total": 5, "valid": 5, "invalid": 0},
            "sboms": [],
            "result": "PASS",
        },
    }
    rc, err = with_temp_evidence(files, expect_rc_nonzero=True)
    ok = rc != 0 and "sbom-index lists 5" in err
    print(f"  sbom_index_lies:         rc={rc} {'PASS' if ok else 'FAIL'}")
    return ok


def case_source_commit_drift() -> bool:
    """Evidence file carrying a stale source_commit MUST fail."""
    files = {
        "toolchain-versions.json": {
            "source_commit": "0" * 40,  # not HEAD
            "tools": [],
            "result": "PASS",
        },
    }
    rc, err = with_temp_evidence(files, expect_rc_nonzero=True)
    ok = rc != 0 and "source_commit=" in err
    print(f"  source_commit_drift:     rc={rc} {'PASS' if ok else 'FAIL'}")
    return ok


def case_secret_scan_findings_but_pass() -> bool:
    """secret-scan.json with findings_count > 0 and result=PASS must be rejected."""
    files = {
        "toolchain-versions.json": {"source_commit": sha(), "tools": [], "result": "PASS"},
        "secret-scan.json": {
            "source_commit": sha(),
            "findings_count": 3,
            "raw_exit_code": 1,
            "result": "PASS",
        },
    }
    rc, err = with_temp_evidence(files, expect_rc_nonzero=True)
    ok = rc != 0 and "findings_count=3" in err
    print(f"  secret_scan_pass_with_findings: rc={rc} {'PASS' if ok else 'FAIL'}")
    return ok


def main() -> int:
    cases = [
        case_rc1_style_toolchain_contradiction,
        case_sbom_index_lies_about_count,
        case_source_commit_drift,
        case_secret_scan_findings_but_pass,
    ]
    fails = 0
    for c in cases:
        try:
            if not c():
                fails += 1
        except Exception as e:
            print(f"  {c.__name__}: EXCEPTION {e}")
            fails += 1
    print(f"\ngate-10 negative summary: {len(cases)-fails}/{len(cases)} PASS")
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
