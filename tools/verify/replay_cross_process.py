#!/usr/bin/env python3
"""Cross-process determinism check: two invocations of emit_trace_digest
for each scenario must return byte-identical digests, and different
scenarios must return different digests.
"""
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BIN = REPO / "target" / "release" / "emit_trace_digest"
OUT = REPO / "build" / "evidence" / "rc2" / "verification" / "replay-cross-process.json"


def run(scn: str) -> str:
    p = subprocess.run([str(BIN), scn], capture_output=True, text=True, check=False, cwd=REPO)
    if p.returncode != 0:
        raise SystemExit(f"emit_trace_digest {scn} exit={p.returncode}: {p.stderr}")
    return p.stdout.strip()


def main() -> int:
    if not BIN.exists():
        subprocess.check_call(
            ["cargo", "build", "-p", "aeon-simulation", "--bin", "emit_trace_digest", "--release"],
            cwd=REPO,
        )
    scns = ["single_clean_track", "crossing_two_tracks"]
    OUT.parent.mkdir(parents=True, exist_ok=True)
    report = {"scenarios": {}}
    ok = True
    for s in scns:
        runs = [run(s) for _ in range(3)]
        stable = all(x == runs[0] for x in runs)
        report["scenarios"][s] = {"runs": runs, "stable": stable}
        if not stable:
            ok = False
    a = report["scenarios"][scns[0]]["runs"][0]
    b = report["scenarios"][scns[1]]["runs"][0]
    report["differs_across_scenarios"] = a != b
    if a == b:
        ok = False
    report["result"] = "PASS" if ok else "FAIL"
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True))
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
