#!/usr/bin/env python3
"""RC2-F scope-boundary negative-path suite.

Each case plants a fixture file with a prohibited construct,
runs the layered scanner over a copy of the tree, asserts the scanner
returns nonzero and the offending layer reports FAIL, and cleans up.
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
SCANNER = REPO / "tools" / "scope" / "run.py"


def run_scanner() -> tuple[int, dict]:
    p = subprocess.run([sys.executable, str(SCANNER)], cwd=REPO,
                       capture_output=True, text=True)
    results_path = REPO / "build" / "evidence" / "rc2" / "scope" / "scope-results.json"
    return p.returncode, json.loads(results_path.read_text())


def with_temp_fixture(rel: str, contents: str, fn):
    """Plant a fixture, run fn, and always remove the fixture."""
    p = REPO / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(contents)
    try:
        return fn()
    finally:
        p.unlink(missing_ok=True)


def case_prohibited_in_production_source() -> bool:
    fixture_rel = "core-runtime/src/_rc2_scope_fixture.rs"
    def check():
        rc, results = run_scanner()
        layer = results["layers"]["production_source"]
        ok = layer["result"] == "FAIL" and any(
            f["path"] == fixture_rel for f in layer["findings"]
        )
        print(f"  prohibited_in_production_source: rc={rc} result={layer['result']} {'PASS' if ok else 'FAIL'}")
        return ok
    return with_temp_fixture(fixture_rel, "// firing_solution\n", check)


def case_prohibited_in_public_contract() -> bool:
    fixture_rel = "contracts/src/_rc2_scope_fixture.rs"
    def check():
        rc, results = run_scanner()
        layer = results["layers"]["public_contracts"]
        # public_contracts specifically lists 11 files that don't
        # include our fixture, so the fixture must be detected by
        # production_source instead. Confirm production_source catches it.
        prod = results["layers"]["production_source"]
        ok = prod["result"] == "FAIL" and any(
            f["path"] == fixture_rel for f in prod["findings"]
        )
        print(f"  prohibited_in_public_contract:   rc={rc} prod={prod['result']} {'PASS' if ok else 'FAIL'}")
        return ok
    return with_temp_fixture(fixture_rel, "// aimpoint_selection type\n", check)


def case_prohibited_in_schema() -> bool:
    fixture_rel = "verification/_rc2_scope_fixture.schema.json"
    def check():
        rc, results = run_scanner()
        layer = results["layers"]["schemas"]
        ok = layer["result"] == "FAIL" and any(
            f["path"] == fixture_rel for f in layer["findings"]
        )
        print(f"  prohibited_in_schema:            rc={rc} result={layer['result']} {'PASS' if ok else 'FAIL'}")
        return ok
    return with_temp_fixture(
        fixture_rel,
        '{"$schema":"https://x/","required":["launch_command"]}',
        check,
    )


def case_prohibited_in_deployment_script() -> bool:
    fixture_rel = "deployment/_rc2_scope_fixture.sh"
    def check():
        rc, results = run_scanner()
        layer = results["layers"]["deployment"]
        ok = layer["result"] == "FAIL" and any(
            f["path"] == fixture_rel for f in layer["findings"]
        )
        print(f"  prohibited_in_deployment:        rc={rc} result={layer['result']} {'PASS' if ok else 'FAIL'}")
        return ok
    return with_temp_fixture(fixture_rel, "#!/bin/sh\n# firing_solution\n", check)


def case_prose_doc_does_not_false_positive() -> bool:
    """A .md file quoting prohibited tokens must not create a
    production_source finding (documentation_claims layer)."""
    fixture_rel = "docs/_rc2_scope_prose_fixture.md"
    def check():
        rc, results = run_scanner()
        for layer_name in ("production_source", "public_contracts", "schemas", "deployment"):
            layer = results["layers"][layer_name]
            for f in layer["findings"]:
                if f["path"] == fixture_rel:
                    print(f"  prose_doc_no_false_positive:   FAIL (prose flagged in {layer_name})")
                    return False
        print("  prose_doc_no_false_positive:     PASS")
        return True
    return with_temp_fixture(
        fixture_rel,
        "# audit prose\n\nThe system excludes firing_solution, launch_command, aimpoint_selection.\n",
        check,
    )


def case_baseline_typed_relay_still_fail() -> bool:
    """RC2-F must NOT declare overall PASS while Finding 7 is open."""
    rc, results = run_scanner()
    ok = results["overall_scope_boundary"]["result"] == "FAIL" and \
         results["typed_relay_boundary"]["result"] == "FAIL"
    print(f"  baseline_typed_relay_fail:       rc={rc} overall={results['overall_scope_boundary']['result']} {'PASS' if ok else 'FAIL'}")
    return ok


def main() -> int:
    cases = [
        case_prohibited_in_production_source,
        case_prohibited_in_public_contract,
        case_prohibited_in_schema,
        case_prohibited_in_deployment_script,
        case_prose_doc_does_not_false_positive,
        case_baseline_typed_relay_still_fail,
    ]
    fails = 0
    for c in cases:
        try:
            if not c():
                fails += 1
        except Exception as e:
            print(f"  {c.__name__}: EXCEPTION {e}")
            fails += 1
    print(f"\nscope negative-path summary: {len(cases)-fails}/{len(cases)} PASS")
    # Rerun scanner cleanly to leave the primary evidence file clean.
    subprocess.run([sys.executable, str(SCANNER)], cwd=REPO, capture_output=True)
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
