#!/usr/bin/env python3
"""Fail if any required document is missing."""
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REQUIRED = [
    "README.md",
    "SECURITY.md",
    "LICENSE",
    "docs/architecture/SCOPE_BOUNDARY.md",
    "docs/architecture/SYSTEM_OVERVIEW.md",
    "docs/architecture/COMPONENT_MODEL.md",
    "docs/architecture/DATA_FLOW.md",
    "docs/architecture/DEPLOYMENT_MODEL.md",
    "docs/architecture/FAILURE_MODEL.md",
    "docs/verification/KNOWN_LIMITATIONS.md",
    "docs/verification/ACCEPTANCE_PLAN.md",
    "docs/verification/RC1_AUDIT_REJECTION.md",
    "docs/verification/rc2/RC2_STARTING_STATE.md",
    "docs/verification/rc2/RC2_AUDIT_FINDING_MATRIX.md",
    "docs/verification/rc2/VERIFICATION_PATH_INVENTORY.md",
    "verification/verification-requirements.json",
    "verification/verification-requirements.schema.json",
]


def main() -> int:
    missing = [p for p in REQUIRED if not (REPO / p).is_file()]
    if missing:
        for m in missing:
            print(f"missing required document: {m}", file=sys.stderr)
        return 1
    print(f"documentation presence: PASS ({len(REQUIRED)} required documents present)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
