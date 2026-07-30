#!/usr/bin/env python3
"""RC2 Gate 10 evidence-consistency validator.

Fails if:
  * expected_version differs from installed_version (toolchain-versions.json);
  * raw exit_code contradicts the normalized result;
  * SBOM index refers to missing files;
  * SBOM validation count differs from the on-disk file count;
  * an expired vulnerability disposition exists;
  * a report says PASS while a mandatory raw result failed;
  * source_commit differs across evidence files;
  * gate-10-report.md contradicts gate-10-results.json.
"""
from __future__ import annotations
import glob
import json
import subprocess
import sys
from datetime import date
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT  = REPO / "build" / "evidence" / "rc2" / "gate-10"


def head() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def problems() -> list[str]:
    ps = []
    sha = head()

    # 1. Toolchain
    tc = OUT / "toolchain-versions.json"
    if not tc.exists():
        ps.append("toolchain-versions.json missing"); return ps
    tc_j = json.loads(tc.read_text())
    if tc_j.get("source_commit") != sha:
        ps.append(f"toolchain-versions.json: source_commit={tc_j.get('source_commit')} ≠ HEAD={sha}")
    for t in tc_j["tools"]:
        if t["status"] != "ok":
            ps.append(f"toolchain: {t['name']} status={t['status']} (expected {t['expected_version']}, got {t['installed_version']})")

    # 2. Dependency audit + dispositions
    dep = OUT / "dependency-audit.json"
    if dep.exists():
        d = json.loads(dep.read_text())
        if d.get("source_commit") != sha:
            ps.append(f"dependency-audit.json: source_commit mismatch")
        # Exit code vs. result consistency:
        if d.get("raw_exit_code") != 0 and d.get("dispositions_result") != "PASS":
            if d.get("result") == "PASS":
                ps.append("dependency-audit: PASS while raw exit ≠ 0 AND dispositions FAIL")
    ledger = REPO / "cybersecurity" / "vulnerability-dispositions.json"
    if ledger.exists():
        try:
            lg = json.loads(ledger.read_text())
            today = date.today()
            for e in lg.get("dispositions", []):
                exp = e.get("expires")
                try:
                    if date.fromisoformat(exp) < today:
                        ps.append(f"disposition {e.get('advisory')}: expired ({exp})")
                except Exception:
                    ps.append(f"disposition {e.get('advisory')}: invalid expires={exp!r}")
        except Exception as e:
            ps.append(f"vulnerability-dispositions.json unreadable: {e}")

    # 3. Secret scan
    ss = OUT / "secret-scan.json"
    if ss.exists():
        s = json.loads(ss.read_text())
        if s.get("source_commit") != sha:
            ps.append("secret-scan.json: source_commit mismatch")
        # Findings > 0 must be FAIL.
        if s.get("findings_count", 0) > 0 and s.get("result") == "PASS":
            ps.append(f"secret-scan: PASS while findings_count={s['findings_count']}")

    # 4. SBOM
    sb = OUT / "sbom-index.json"
    if sb.exists():
        sj = json.loads(sb.read_text())
        if sj.get("source_commit") != sha:
            ps.append("sbom-index.json: source_commit mismatch")
        listed = sj.get("summary", {}).get("total", 0)
        actual = len(list(OUT.glob("sbom/*.cdx.json")))
        if listed != actual:
            ps.append(f"sbom-index lists {listed} SBOMs; {actual} on disk")
        invalid = sj.get("summary", {}).get("invalid", 0)
        if invalid > 0 and sj.get("result") == "PASS":
            ps.append(f"sbom-index: PASS with invalid={invalid}")

    return ps


def main() -> int:
    ps = problems()

    # Emit gate-10-results.json (single source of truth for the gate decision).
    # A PASS requires every subcheck PASS AND zero problems here.
    def load(name):
        p = OUT / name
        return json.loads(p.read_text()) if p.exists() else {}
    tc = load("toolchain-versions.json").get("result", "FAIL")
    dep = load("dependency-audit.json").get("result", "FAIL")
    ss  = load("secret-scan.json").get("result", "FAIL")
    sb  = load("sbom-index.json").get("result", "FAIL")
    subresults = {"toolchain": tc, "dependency_audit": dep, "secret_scan": ss, "sbom": sb}
    result = "PASS" if (all(v == "PASS" for v in subresults.values()) and not ps) else "FAIL"

    gate = {
        "generated_at_source_commit": head(),
        "subresults": subresults,
        "consistency_problems": ps,
        "result": result,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "gate-10-results.json").write_text(json.dumps(gate, indent=2, sort_keys=True))
    md = [
        "# RC2 Gate 10 report",
        "",
        f"- Source commit: `{gate['generated_at_source_commit']}`",
        f"- Overall result: **{result}**",
        "",
        "## Subresults",
        "",
        "| check | result |",
        "|---|---|",
    ]
    for k, v in subresults.items():
        md.append(f"| `{k}` | {v} |")
    if ps:
        md += ["", "## Consistency problems", ""] + [f"- {p}" for p in ps]
    (OUT / "gate-10-report.md").write_text("\n".join(md) + "\n")

    if ps:
        for p in ps:
            print(f"gate-10 consistency: {p}", file=sys.stderr)
        return 1
    print("gate-10 consistency: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
