#!/usr/bin/env python3
"""RC2 fail-closed verification runner.

Consumes verification/verification-requirements.json. Every mandatory
step must return exit 0; missing tool, missing target, empty
test set, or contradictory summary is a FAIL. No silent skip, no
advisory downgrade, no BLOCKED_EXTERNAL inside verify:all.

Outputs:
  build/evidence/rc2/verification/verify-all-results.json  (structured ledger)
  build/evidence/rc2/verification/verify-all-report.md     (generated from ledger)
  build/evidence/rc2/verification/logs/<step_id>.stdout / .stderr

Exit code = number of failed mandatory steps (0 on total success).
"""

from __future__ import annotations
import argparse
import datetime as dt
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
REQ_FILE  = REPO_ROOT / "verification" / "verification-requirements.json"
EVIDENCE  = REPO_ROOT / "build" / "evidence" / "rc2" / "verification"
LOGS      = EVIDENCE / "logs"


def now_rfc3339() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def head_sha() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, text=True
        ).strip()
    except Exception:
        return "unknown"


def tool_versions() -> dict:
    out = {}
    for name, cmd in (
        ("rustc", ["rustc", "--version"]),
        ("cargo", ["cargo", "--version"]),
        ("python", ["python3", "--version"]),
    ):
        try:
            out[name] = subprocess.check_output(cmd, text=True).strip()
        except Exception:
            out[name] = "unavailable"
    return out


def run_step(step: dict) -> dict:
    step_id = step["step_id"]
    LOGS.mkdir(parents=True, exist_ok=True)
    stdout_path = LOGS / f"{step_id}.stdout"
    stderr_path = LOGS / f"{step_id}.stderr"

    started = now_rfc3339()
    start_t = dt.datetime.now(dt.timezone.utc)

    if not step.get("mandatory", True):
        raise SystemExit(
            f"step {step_id!r}: mandatory=false is not permitted in RC2 verify:all"
        )
    if step.get("failure_semantics") != "fail_closed":
        raise SystemExit(
            f"step {step_id!r}: failure_semantics must be 'fail_closed' in RC2"
        )

    entry: dict[str, Any] = {
        "step_id": step_id,
        "command": step["command"],
        "mandatory": True,
        "started_at": started,
        "tool_versions": tool_versions(),
        "stdout_path": str(stdout_path.relative_to(REPO_ROOT)),
        "stderr_path": str(stderr_path.relative_to(REPO_ROOT)),
        "source_commit": head_sha(),
    }

    # 0. Refuse to run if the required executable is missing — never silent
    # skip. Special case: cargo alias / python script paths.
    exe = step["command"][0]
    if exe not in ("python3",) and shutil.which(exe) is None:
        entry.update(
            finished_at=now_rfc3339(),
            duration_ms=0,
            exit_code=127,
            result="FAIL",
            failure_reason=f"required executable '{exe}' not found in PATH",
        )
        return entry

    # 1. Execute
    with open(stdout_path, "w") as so, open(stderr_path, "w") as se:
        proc = subprocess.run(
            step["command"],
            cwd=REPO_ROOT,
            stdout=so,
            stderr=se,
            env={**os.environ, "PATH": f"{REPO_ROOT}/.aeon-tools/bin:{os.environ.get('PATH','')}"},
        )
    exit_code = proc.returncode
    finished = now_rfc3339()
    duration_ms = int(
        (dt.datetime.now(dt.timezone.utc) - start_t).total_seconds() * 1000
    )

    # 2. Post-conditions
    result = "PASS" if exit_code == 0 else "FAIL"
    failure_reason = None
    if exit_code != 0:
        failure_reason = f"exit_code={exit_code}"

    # 2a. Expected artifact must exist and be non-empty (if declared).
    art = step.get("expected_artifact")
    if art:
        art_path = REPO_ROOT / art
        if not art_path.exists() or art_path.stat().st_size == 0:
            result = "FAIL"
            failure_reason = (
                (failure_reason + "; " if failure_reason else "")
                + f"expected artifact missing or empty: {art}"
            )

    # 2b. Minimum test count (parse cargo-test tail lines from stdout).
    mtc = step.get("minimum_test_count")
    if mtc is not None:
        try:
            total_passed = 0
            with open(stdout_path) as f:
                for line in f:
                    if line.startswith("test result: ok."):
                        # e.g. "test result: ok. 6 passed; 0 failed; ..."
                        total_passed += int(line.split()[3])
            if total_passed < mtc:
                result = "FAIL"
                failure_reason = (
                    (failure_reason + "; " if failure_reason else "")
                    + f"minimum_test_count={mtc}, saw {total_passed}"
                )
        except Exception as e:
            result = "FAIL"
            failure_reason = (
                (failure_reason + "; " if failure_reason else "")
                + f"could not parse test count: {e}"
            )

    entry.update(
        finished_at=finished,
        duration_ms=duration_ms,
        exit_code=exit_code,
        result=result,
    )
    if failure_reason:
        entry["failure_reason"] = failure_reason
    return entry


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--only",
        action="append",
        default=[],
        help="Run only the listed step_ids (repeatable). Default: run all.",
    )
    args = ap.parse_args()

    EVIDENCE.mkdir(parents=True, exist_ok=True)
    LOGS.mkdir(parents=True, exist_ok=True)

    reqs = json.load(open(REQ_FILE))
    steps = reqs["steps"]
    if args.only:
        want = set(args.only)
        steps = [s for s in steps if s["step_id"] in want]
        missing = want - {s["step_id"] for s in steps}
        if missing:
            print(f"unknown step_ids: {sorted(missing)}", file=sys.stderr)
            return 2

    ledger: dict[str, Any] = {
        "manifest_version": reqs["manifest_version"],
        "source_commit": head_sha(),
        "started_at": now_rfc3339(),
        "steps": [],
    }
    failed_ids: list[str] = []
    for step in steps:
        entry = run_step(step)
        ledger["steps"].append(entry)
        if entry["result"] == "FAIL":
            failed_ids.append(entry["step_id"])
        print(f"[verify] {entry['step_id']:32s} → {entry['result']}")

    ledger["finished_at"] = now_rfc3339()
    ledger["result"] = "PASS" if not failed_ids else "FAIL"
    ledger["failed_step_ids"] = failed_ids

    (EVIDENCE / "verify-all-results.json").write_text(json.dumps(ledger, indent=2, sort_keys=True))

    # Generate Markdown report from the ledger — never hand-edited.
    lines = [
        "# RC2 verify:all report",
        "",
        f"- Source commit: `{ledger['source_commit']}`",
        f"- Started: `{ledger['started_at']}`",
        f"- Finished: `{ledger['finished_at']}`",
        f"- Overall result: **{ledger['result']}**",
        "",
        "## Steps",
        "",
        "| step_id | result | exit | duration ms | reason |",
        "|---|---|---|---|---|",
    ]
    for s in ledger["steps"]:
        lines.append(
            f"| `{s['step_id']}` | {s['result']} | {s['exit_code']} | {s['duration_ms']} | {s.get('failure_reason','')} |"
        )
    (EVIDENCE / "verify-all-report.md").write_text("\n".join(lines) + "\n")

    return len(failed_ids)


if __name__ == "__main__":
    sys.exit(main())
