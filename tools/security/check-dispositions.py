#!/usr/bin/env python3
"""Cross-reference cargo-audit findings against the disposition ledger.

Usage:
  check-dispositions.py --audit <audit.json> --ledger <dispositions.json>
    [--out <report.json>]

Exit codes:
  0  every finding is covered by a non-expired, non-open disposition
  1  unmatched, open, or expired finding present
  2  usage / read error
"""

from __future__ import annotations
import argparse
import datetime as dt
import json
import sys


def load_json(path):
    try:
        return json.load(open(path))
    except Exception as e:
        print(f"cannot read {path}: {e}", file=sys.stderr)
        sys.exit(2)


def norm_advisory(a):
    return (a or "").strip().upper()


def _today():
    return dt.date.today()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--audit", required=True)
    ap.add_argument("--ledger", required=True)
    ap.add_argument("--out", default="/dev/stdout")
    args = ap.parse_args()

    audit = load_json(args.audit)
    ledger = load_json(args.ledger)

    findings = []
    # cargo-audit --json emits {"vulnerabilities":{"list":[{...}]}}
    for v in (audit.get("vulnerabilities") or {}).get("list") or []:
        adv = v.get("advisory") or {}
        findings.append({
            "advisory": norm_advisory(adv.get("id")),
            "package": (v.get("package") or {}).get("name"),
            "affected_version": (v.get("package") or {}).get("version"),
        })

    ledger_by_advisory = {}
    for d in ledger.get("dispositions", []):
        ledger_by_advisory[norm_advisory(d.get("advisory"))] = d

    today = _today()
    report = {"today": today.isoformat(), "results": [], "summary": {}}
    unresolved = 0
    resolved = 0
    for f in findings:
        entry = {"finding": f, "state": None, "disposition": None, "verdict": None}
        d = ledger_by_advisory.get(f["advisory"])
        if not d:
            entry.update(state="uncovered", verdict="fail",
                         reason="no disposition entry for advisory")
        else:
            exp = d.get("expires")
            expired = False
            try:
                expired = dt.date.fromisoformat(exp) < today
            except Exception:
                expired = True
            entry["disposition"] = d
            if d["state"] == "open":
                entry.update(state="open", verdict="fail")
            elif expired or d["state"] == "expired":
                entry.update(state="expired", verdict="fail")
            elif d["state"] in ("mitigated", "not_reachable", "dev_only", "remediated"):
                entry.update(state=d["state"], verdict="pass")
            else:
                entry.update(state=d["state"], verdict="fail",
                             reason=f"unknown state {d['state']}")
        if entry["verdict"] == "fail":
            unresolved += 1
        else:
            resolved += 1
        report["results"].append(entry)
    report["summary"] = {
        "total_findings": len(findings),
        "resolved": resolved,
        "unresolved": unresolved,
    }

    with open(args.out, "w") as fh:
        json.dump(report, fh, indent=2, sort_keys=True)

    if unresolved:
        print(f"vulnerability check: FAIL — {unresolved} unresolved finding(s)", file=sys.stderr)
        return 1
    print(f"vulnerability check: PASS — {len(findings)} finding(s), all dispositioned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
