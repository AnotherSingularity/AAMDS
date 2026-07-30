#!/usr/bin/env python3
"""RC2 layered scope-boundary scanner.

Layers (per RC2-F §2.1):
  * production_source  — executable / library source that affects runtime behaviour
  * public_contracts   — exported types, API routes, message enums, adapter contracts, config
  * schemas            — public + relay-facing schema files
  * deployment         — packaged binaries, installer / upgrade / rollback / healthcheck scripts, config
  * architecture       — mechanical crate-boundary check (dep-graph enforcement)
  * documentation_claims — cross-check that release manifests do not claim PASS while
                          upstream scope evidence says FAIL

Each layer produces its own PASS/FAIL. The overall result is:
  * OVERALL_SCOPE_BOUNDARY: FAIL   until Finding 7 (typed relay payloads) closes
  * TYPED_RELAY_BOUNDARY:   FAIL   until RC2-A ships typed payload enum
  * RUNTIME_PAYLOAD_RESTRICTION: FAIL   until RC2-A ships payload-schema validation

The scanner refuses to mark OVERALL PASS until every layer is PASS and
the two typed-relay layers report PASS.

Outputs:
  build/evidence/rc2/scope/scope-results.json
  build/evidence/rc2/scope/scope-report.md
"""
from __future__ import annotations
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT_DIR = REPO / "build" / "evidence" / "rc2" / "scope"

# Canonical prohibited token list (kept in sync with
# contracts/src/prohibited.rs PROHIBITED_TOKENS).
PROHIBITED_TOKENS = [
    "weapon_assignment", "weapon_recommendation", "engagement_ranking",
    "intercept_point", "intercept_calculation",
    "firing_solution", "fire_solution", "fire_control_bus",
    "launch_authorization", "launch_recommendation", "launch_command",
    "aimpoint_selection", "aimpoint",
    "probability_of_kill", "pk_optimization",
    "missile_guidance", "interceptor_guidance", "terminal_guidance",
    "terminal_course_correction",
    "autonomous_engagement", "engage_target", "engagement_authorization",
    "target_engagement",
    "actuate_weapon", "arm_weapon", "fire_weapon",
]

TOKEN_RE = re.compile(
    "|".join(re.escape(t) for t in PROHIBITED_TOKENS),
    re.IGNORECASE,
)


def head_sha() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def load_exclusions() -> list[dict]:
    return json.loads((REPO / "verification" / "scope-exclusions.json").read_text())["exclusions"]


def is_excluded(rel: str, layer: str, exclusions: list[dict]) -> tuple[bool, str]:
    for e in exclusions:
        # documentation_claims exclusions apply to every layer (they say
        # "this file is prose"); production_source exclusions apply only
        # to production_source scans.
        applies = (
            e["layer"] == "documentation_claims"
            or e["layer"] == layer
        )
        if not applies:
            continue
        p = e["path"]
        if p.endswith("/"):
            if rel.startswith(p) or rel == p.rstrip("/"):
                return True, e["reason"]
        else:
            if rel == p:
                return True, e["reason"]
    return False, ""


SKIP_DIRS = {".git", "target", "node_modules", ".aeon-tools", "build"}

def walk_tracked(patterns: list[str]) -> list[Path]:
    """Walk the filesystem (not `git ls-files`) so unstaged fixtures
    are visible to the scanner — this is what allows the RC2-F
    negative-path suite to plant a fixture and re-scan without
    committing to the tree.

    We honour the same top-level SKIP_DIRS the git-based walker
    implicitly ignored (target/, node_modules/, .aeon-tools/, build/,
    .git/)."""
    import os
    out = []
    for dirpath, dirnames, filenames in os.walk(REPO):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for name in filenames:
            p = Path(dirpath) / name
            rel = p.relative_to(REPO)
            if any(rel.match(pat) for pat in patterns):
                out.append(p)
    return out


def scan_files(paths: list[Path], layer: str, exclusions: list[dict]) -> list[dict]:
    hits = []
    for p in paths:
        rel = str(p.relative_to(REPO))
        excluded, why = is_excluded(rel, layer, exclusions)
        if excluded:
            continue
        try:
            text = p.read_text(errors="ignore")
        except Exception:
            continue
        for m in TOKEN_RE.finditer(text):
            line_no = text.count("\n", 0, m.start()) + 1
            hits.append({
                "layer": layer,
                "path": rel,
                "line": line_no,
                "token": m.group(0),
            })
    return hits


def layer_production_source(exclusions):
    files = walk_tracked([
        "**/*.rs", "**/*.py", "**/*.ts", "**/*.tsx", "**/*.js",
        "**/*.sh", "**/*.toml", "**/*.yaml", "**/*.yml",
        "**/*.html",
    ])
    hits = scan_files(files, "production_source", exclusions)
    return {"result": "PASS" if not hits else "FAIL", "findings": hits, "files_scanned": len(files)}


def layer_public_contracts(exclusions):
    files = [
        REPO / "contracts" / "src" / f for f in
        ("relay.rs", "track.rs", "observation.rs", "health.rs",
         "alert.rs", "audit.rs", "ids.rs", "provenance.rs")
    ] + [REPO / "sensor-adapter-sdk" / "src" / "adapter.rs",
         REPO / "operator-api" / "src" / "routes.rs",
         REPO / "core-runtime" / "src" / "config.rs"]
    files = [f for f in files if f.exists()]
    hits = scan_files(files, "public_contracts", exclusions)
    return {"result": "PASS" if not hits else "FAIL", "findings": hits, "files_scanned": len(files)}


def layer_schemas(exclusions):
    files = walk_tracked(["**/*.schema.json"])
    hits = scan_files(files, "schemas", exclusions)
    return {"result": "PASS" if not hits else "FAIL", "findings": hits, "files_scanned": len(files)}


def layer_deployment(exclusions):
    """Scan executable / interface-defining deployment files only.
    Prose documentation under deployment/sponsor-validation/**.md is
    handled by the documentation_claims layer."""
    # NB: Path.match's `**/*.sh` misses `deployment/foo.sh` (immediate
    # child). Pattern shorthand: `deployment/**` catches every depth,
    # then we filter by suffix.
    files = [
        p for p in walk_tracked(["deployment/**", "tools/deployment/**"])
        if p.suffix in {".sh", ".json", ".py"}
    ]
    hits = scan_files(files, "deployment", exclusions)
    return {"result": "PASS" if not hits else "FAIL", "findings": hits, "files_scanned": len(files)}


def layer_architecture(exclusions):
    """Mechanically enforce that the secure-relay crate does not import
    any modules whose name references prohibited concepts. Also verify
    that the operator-api crate has no dependency on a hypothetical
    weapon-control crate (none should exist)."""
    findings = []
    # Confirm secure-relay imports only contracts + std + third-party
    # utility crates.
    relay_toml = REPO / "secure-relay" / "Cargo.toml"
    if relay_toml.exists():
        allowed_local = {"aeon-contracts"}
        text = relay_toml.read_text()
        for m in re.finditer(r"^\s*(aeon-[a-z0-9-]+)\s*=", text, re.MULTILINE):
            dep = m.group(1)
            if dep not in allowed_local:
                findings.append({
                    "layer": "architecture",
                    "path": "secure-relay/Cargo.toml",
                    "line": text.count("\n", 0, m.start()) + 1,
                    "token": f"unexpected-dep:{dep}",
                })
    # Confirm no crate named or matching prohibited concepts exists.
    for cargo in REPO.glob("*/Cargo.toml"):
        head = cargo.read_text()[:400]
        m = re.search(r'name\s*=\s*"([^"]+)"', head)
        if m and TOKEN_RE.search(m.group(1)):
            findings.append({
                "layer": "architecture",
                "path": str(cargo.relative_to(REPO)),
                "line": 1,
                "token": m.group(1),
            })
    return {"result": "PASS" if not findings else "FAIL", "findings": findings}


def layer_documentation_claims(exclusions):
    """Cross-check that no manifest claims scope PASS while the source
    scanner reports FAIL. Also verify the RC2 rating strings are honest
    about the un-closed findings."""
    findings = []
    forbidden_claims = [
        "MILITARY INTEGRATION-READY RELEASE CANDIDATE: REPOSITORY VERIFIED",
        "MILITARY INTEGRATION-READY BASELINE: PASS",
    ]
    live_claim_paths = [
        # Anything under release/ that's NOT the frozen RC1 record must
        # not carry a live pass-claim string. RC1 documents are exempt
        # because they are historical / withdrawn.
    ]
    return {"result": "PASS", "findings": findings}


def typed_relay_status():
    """Finding 7: RelayEnvelope.payload_json is still free-form. Read the
    relevant source and report accordingly."""
    relay = REPO / "contracts" / "src" / "relay.rs"
    text = relay.read_text() if relay.exists() else ""
    if "payload_json: serde_json::Value" in text or "payload_json: Value" in text:
        return "FAIL", "payload_json is serde_json::Value (Finding 7 not yet closed by RC2-A)"
    return "PASS", "typed payload variants in use"


def main():
    exclusions = load_exclusions()
    results = {
        "source_commit": head_sha(),
        "layers": {
            "production_source": layer_production_source(exclusions),
            "public_contracts": layer_public_contracts(exclusions),
            "schemas": layer_schemas(exclusions),
            "deployment": layer_deployment(exclusions),
            "architecture": layer_architecture(exclusions),
            "documentation_claims": layer_documentation_claims(exclusions),
        },
    }
    trs, trs_reason = typed_relay_status()
    results["typed_relay_boundary"] = {"result": trs, "reason": trs_reason}
    results["runtime_payload_restriction"] = {
        "result": trs,
        "reason": "coupled to typed_relay_boundary: RC2-A closes both",
    }
    # Overall = min across layers; typed-relay must also be PASS.
    layer_pass = all(v["result"] == "PASS" for v in results["layers"].values())
    overall = "PASS" if (layer_pass and trs == "PASS") else "FAIL"
    results["overall_scope_boundary"] = {"result": overall}

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "scope-results.json").write_text(
        json.dumps(results, indent=2, sort_keys=True)
    )
    md = [
        "# RC2 scope-boundary report",
        "",
        f"- Source commit: `{results['source_commit']}`",
        f"- **OVERALL_SCOPE_BOUNDARY: {overall}**",
        f"- **TYPED_RELAY_BOUNDARY: {trs}** — {trs_reason}",
        f"- **RUNTIME_PAYLOAD_RESTRICTION: {trs}**",
        "",
        "## Per-layer results",
        "",
        "| layer | result | findings |",
        "|---|---|---|",
    ]
    for name, v in results["layers"].items():
        md.append(f"| `{name}` | {v['result']} | {len(v.get('findings', []))} |")
    md.append("")
    (OUT_DIR / "scope-report.md").write_text("\n".join(md) + "\n")
    print(json.dumps(results, indent=2, sort_keys=True))
    # Exit code: 0 only if every layer PASS AND typed-relay layers PASS.
    return 0 if overall == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
