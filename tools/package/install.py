#!/usr/bin/env python3
"""RC2 installer with pre-copy verification + atomic staging.

Verifies the package BEFORE any file is copied to $AEON_HOME.
Uses a temporary staging directory and only promotes it to
$AEON_HOME on complete success (atomic mv). Failure leaves no
partially-trusted installation active.
"""
from __future__ import annotations
import argparse
import shutil
import sys
import subprocess
import tempfile
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]


def run_verify(pkg: Path) -> int:
    return subprocess.call(
        [sys.executable, str(REPO / "tools" / "package" / "verify.py"), "--pkg", str(pkg)]
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pkg", required=True, help="package staging dir (bin/, ui/, config/, scripts/, package.manifest.v2.json)")
    ap.add_argument("--home", required=True)
    ap.add_argument("--force", action="store_true")
    args = ap.parse_args()
    pkg = Path(args.pkg).resolve()
    home = Path(args.home).resolve()
    if home.exists() and (home / "package.manifest.v2.json").exists() and not args.force:
        print(f"install: {home} already contains an installation; --force to overwrite", file=sys.stderr)
        return 3
    rc = run_verify(pkg)
    if rc != 0:
        print("install: package verification FAILED; refusing to copy", file=sys.stderr)
        return 4
    # Stage into a sibling temp dir and rename atomically.
    home.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f"{home.name}.staging.", dir=str(home.parent)))
    try:
        for item in pkg.iterdir():
            dest = staging / item.name
            if item.is_dir():
                shutil.copytree(item, dest)
            else:
                shutil.copy2(item, dest)
        # Atomic promotion.
        if home.exists():
            trash = Path(tempfile.mkdtemp(prefix=f"{home.name}.trash.", dir=str(home.parent)))
            home.rename(trash / home.name)
        staging.rename(home)
    except Exception as e:
        shutil.rmtree(staging, ignore_errors=True)
        print(f"install: failed after verification: {e}", file=sys.stderr)
        return 5
    print(f"install: {pkg} → {home} (verified before copy, atomic promotion)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
