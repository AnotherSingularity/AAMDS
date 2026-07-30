#!/usr/bin/env python3
"""Cross-artifact evidence consistency check.

Rules:
  * Every JSON evidence file under build/evidence/rc2/ that carries a
    `source_commit` field must match `git rev-parse HEAD`.
  * The verification report Markdown must not disagree with the
    verify-all-results.json PASS/FAIL decision.
  * The gate-10 report Markdown must not disagree with
    gate-10-results.json.
  * SBOM index counts must match the number of actual SBOM files.
"""
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
EVID = REPO / "build" / "evidence" / "rc2"


def head_sha() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def check_source_commits(sha: str) -> list[str]:
    problems = []
    for p in EVID.rglob("*.json"):
        try:
            d = json.loads(p.read_text())
        except Exception as e:
            problems.append(f"{p}: unreadable ({e})")
            continue
        commits = []
        def _walk(o):
            if isinstance(o, dict):
                for k, v in o.items():
                    if k == "source_commit" and isinstance(v, str):
                        commits.append(v)
                    _walk(v)
            elif isinstance(o, list):
                for x in o:
                    _walk(x)
        _walk(d)
        for c in commits:
            if c and c != sha:
                problems.append(f"{p.relative_to(REPO)}: source_commit={c[:10]}… ≠ HEAD={sha[:10]}…")
    return problems


def check_report_consistency() -> list[str]:
    problems = []
    for name in ("verification/verify-all-results.json", "gate-10/gate-10-results.json"):
        j = EVID / name
        if not j.exists():
            continue
        try:
            d = json.loads(j.read_text())
        except Exception:
            problems.append(f"{j}: unreadable")
            continue
        result = d.get("result")
        md = j.with_suffix("").parent / (j.stem.replace("-results", "-report") + ".md")
        if md.exists():
            body = md.read_text()
            if result == "PASS" and "**FAIL**" in body:
                problems.append(f"{md}: claims FAIL while ledger says PASS")
            if result == "FAIL" and "**PASS**" in body:
                problems.append(f"{md}: claims PASS while ledger says FAIL")
    return problems


def check_sbom_counts() -> list[str]:
    problems = []
    idx_path = EVID / "gate-10" / "sbom-index.json"
    sbom_dir = EVID / "gate-10" / "sbom"
    if not idx_path.exists():
        return problems
    idx = json.loads(idx_path.read_text())
    listed = len(idx.get("sboms", []))
    actual = len(list(sbom_dir.glob("*.cdx.json"))) if sbom_dir.exists() else 0
    if listed != actual:
        problems.append(f"sbom-index lists {listed} SBOMs but {actual} files present")
    return problems


def main() -> int:
    if not EVID.exists():
        print("no build/evidence/rc2/ present — nothing to check", file=sys.stderr)
        return 1
    sha = head_sha()
    problems: list[str] = []
    problems += check_source_commits(sha)
    problems += check_report_consistency()
    problems += check_sbom_counts()
    if problems:
        for p in problems:
            print(f"evidence-consistency: {p}", file=sys.stderr)
        return 1
    print("evidence-consistency: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
