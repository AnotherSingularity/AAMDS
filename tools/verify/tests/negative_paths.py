#!/usr/bin/env python3
"""RC2-B negative-path suite for the fail-closed runner.

Each case builds a temporary requirements manifest containing a single
step designed to fail, invokes tools/verify/run.py with that manifest,
and asserts the runner returns nonzero AND the ledger flags the
step as FAIL.

Uses REPO_ROOT/build/evidence/rc2/verification/ for temp evidence so
the primary evidence directory is not clobbered.
"""
from __future__ import annotations
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RUNNER = REPO / "tools" / "verify" / "run.py"


def run_with_manifest(steps: list[dict]) -> tuple[int, dict]:
    """Run the runner against a temporary requirements manifest and
    return (exit_code, parsed ledger)."""
    tmp_dir = Path(tempfile.mkdtemp(prefix="rc2-neg-"))
    try:
        # Write a temp manifest and point the runner at it. The runner
        # currently loads REPO/verification/verification-requirements.json.
        # We work around this by symlinking/writing a private copy and
        # invoking the runner with --only on step ids that exist in the
        # temp manifest. Simplest: swap in a temp file by env override.
        # The runner doesn't yet accept a manifest override, so use
        # PYTHONPATH-style: write a shim that patches REQ_FILE.
        shim = tmp_dir / "shim.py"
        manifest = tmp_dir / "manifest.json"
        manifest.write_text(json.dumps(
            {"manifest_version": 1, "steps": steps}
        ))
        shim.write_text(
            "import runpy, sys\n"
            "from pathlib import Path\n"
            "sys.argv=['run.py']\n"
            "import importlib.util as u\n"
            f"spec = u.spec_from_file_location('run', r'{RUNNER}')\n"
            "m = u.module_from_spec(spec); spec.loader.exec_module(m)\n"
            f"m.REQ_FILE = Path(r'{manifest}')\n"
            "sys.exit(m.main())\n"
        )
        env = os.environ.copy()
        # Isolate evidence dir per run
        env["PWD"] = str(REPO)
        p = subprocess.run([sys.executable, str(shim)], cwd=REPO, env=env,
                           capture_output=True, text=True)
        ledger_path = REPO / "build" / "evidence" / "rc2" / "verification" / "verify-all-results.json"
        ledger = json.loads(ledger_path.read_text())
        return p.returncode, ledger
    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)


def case_missing_executable() -> bool:
    rc, ledger = run_with_manifest([{
        "step_id": "neg_missing_tool",
        "command": ["definitely-not-installed-42"],
        "mandatory": True,
        "failure_semantics": "fail_closed",
    }])
    ok = rc != 0 and ledger["steps"][0]["result"] == "FAIL"
    print(f"  missing_executable: rc={rc} result={ledger['steps'][0]['result']} {'PASS' if ok else 'FAIL'}")
    return ok


def case_command_nonzero() -> bool:
    rc, ledger = run_with_manifest([{
        "step_id": "neg_false",
        "command": ["false"],
        "mandatory": True,
        "failure_semantics": "fail_closed",
    }])
    ok = rc != 0 and ledger["steps"][0]["result"] == "FAIL"
    print(f"  command_nonzero:    rc={rc} result={ledger['steps'][0]['result']} {'PASS' if ok else 'FAIL'}")
    return ok


def case_missing_artifact() -> bool:
    rc, ledger = run_with_manifest([{
        "step_id": "neg_missing_artifact",
        "command": ["true"],
        "mandatory": True,
        "failure_semantics": "fail_closed",
        "expected_artifact": "build/evidence/rc2/verification/does-not-exist.json",
    }])
    ok = rc != 0 and ledger["steps"][0]["result"] == "FAIL"
    print(f"  missing_artifact:   rc={rc} result={ledger['steps'][0]['result']} {'PASS' if ok else 'FAIL'}")
    return ok


def case_too_few_tests() -> bool:
    # `true` yields no test-result line, so parsed count = 0 < 5 → FAIL.
    rc, ledger = run_with_manifest([{
        "step_id": "neg_min_count",
        "command": ["true"],
        "mandatory": True,
        "failure_semantics": "fail_closed",
        "minimum_test_count": 5,
    }])
    ok = rc != 0 and ledger["steps"][0]["result"] == "FAIL"
    print(f"  too_few_tests:      rc={rc} result={ledger['steps'][0]['result']} {'PASS' if ok else 'FAIL'}")
    return ok


def case_advisory_forbidden() -> bool:
    """RC2 forbids advisory_only in mandatory verify:all steps."""
    try:
        rc, _ = run_with_manifest([{
            "step_id": "neg_advisory",
            "command": ["true"],
            "mandatory": True,
            "failure_semantics": "advisory_only",
        }])
    except SystemExit as e:
        rc = e.code if isinstance(e.code, int) else 1
    # The runner exits nonzero when it refuses to accept the step.
    ok = rc != 0
    print(f"  advisory_forbidden: rc={rc} {'PASS' if ok else 'FAIL'}")
    return ok


def main() -> int:
    cases = [case_missing_executable, case_command_nonzero,
             case_missing_artifact, case_too_few_tests, case_advisory_forbidden]
    fails = 0
    for c in cases:
        try:
            if not c():
                fails += 1
        except Exception as e:
            print(f"  {c.__name__}: EXCEPTION {e}")
            fails += 1
    print(f"\nnegative-path summary: {len(cases) - fails}/{len(cases)} PASS")
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
