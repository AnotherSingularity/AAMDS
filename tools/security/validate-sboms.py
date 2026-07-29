#!/usr/bin/env python3
"""Validate each CycloneDX SBOM under the given directory and emit
an index JSON to stdout.

An SBOM is 'valid' at this baseline if it parses as JSON, declares
bomFormat=CycloneDX, has a specVersion, and lists at least one
component.
"""

from __future__ import annotations
import glob
import json
import os
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate-sboms.py <dir>", file=sys.stderr)
        return 2
    d = sys.argv[1]
    files = sorted(glob.glob(os.path.join(d, "*.cdx.json")))
    valid = 0
    invalid = 0
    entries = []
    for f in files:
        try:
            b = json.load(open(f))
            fmt = b.get("bomFormat") == "CycloneDX"
            spec = bool(b.get("specVersion"))
            comps = len(b.get("components") or [])
            ok = fmt and spec and comps > 0
            entries.append({
                "path": os.path.relpath(f),
                "bomFormat": b.get("bomFormat"),
                "specVersion": b.get("specVersion"),
                "component_count": comps,
                "valid": ok,
            })
            if ok:
                valid += 1
            else:
                invalid += 1
        except Exception as e:
            entries.append({"path": os.path.relpath(f), "valid": False, "error": str(e)})
            invalid += 1
    out = {"summary": {"total": len(files), "valid": valid, "invalid": invalid},
           "sboms": entries}
    print(json.dumps(out, indent=2, sort_keys=True))
    return 0 if invalid == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
