#!/usr/bin/env python3
"""RC2 package verifier.

For each staged package, before any copy/execute:
  1. schema-validate the v2 manifest;
  2. verify the manifest signature over the canonical bytes;
  3. verify every listed file's presence, size, and SHA-256;
  4. reject any unlisted protected file (any file under the package
     other than package.manifest.v2.json must be listed);
  5. reject duplicate / absolute / '..' paths.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SCHEMA_PATH = REPO / "deployment" / "schemas" / "package-manifest.v2.schema.json"
EVID = REPO / "build" / "evidence" / "rc2" / "packages"
PROFILES = ["developer", "edge", "disconnected", "fixed-site", "data-center", "private-cloud"]


def head() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def load_schema() -> dict:
    return json.loads(SCHEMA_PATH.read_text())


def schema_validate(schema, obj, path=""):
    import re
    errs = []
    def err(m): errs.append(f"{path or '/'}: {m}")
    t = schema.get("type")
    if "const" in schema and obj != schema["const"]:
        err(f"const {schema['const']} != {obj}")
    if t == "object":
        if not isinstance(obj, dict):
            err("expected object"); return errs
        for r in schema.get("required", []):
            if r not in obj: err(f"missing {r!r}")
        props = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            for k in obj:
                if k not in props: err(f"unexpected {k!r}")
        for k, v in obj.items():
            if k in props: errs.extend(schema_validate(props[k], v, f"{path}.{k}"))
    elif t == "array":
        if not isinstance(obj, list): err("expected array"); return errs
        if "minItems" in schema and len(obj) < schema["minItems"]:
            err(f"expected ≥ {schema['minItems']} items")
        for i, it in enumerate(obj):
            errs.extend(schema_validate(schema.get("items", {}), it, f"{path}[{i}]"))
    elif t == "string":
        if not isinstance(obj, str): err("expected string")
        if "pattern" in schema and isinstance(obj, str):
            if not re.match(schema["pattern"], obj):
                err(f"pattern mismatch {schema['pattern']}")
    elif t == "integer":
        if not isinstance(obj, int): err("expected int")
        if "minimum" in schema and isinstance(obj, int) and obj < schema["minimum"]:
            err(f"< min {schema['minimum']}")
    elif t == "boolean":
        if not isinstance(obj, bool): err("expected bool")
    if "enum" in schema and obj not in schema["enum"]:
        err(f"value {obj!r} not in enum")
    return errs


def verify_signature(manifest: dict) -> tuple[bool, str]:
    sig = manifest.get("signature", {})
    if sig.get("method") != "dev-hmac-sha256":
        return False, f"unsupported signature method: {sig.get('method')}"
    canonical = json.dumps({k: v for k, v in manifest.items() if k != "signature"},
                           sort_keys=True, separators=(",", ":"))
    digest = hashlib.sha256(canonical.encode()).hexdigest()
    key = os.environ.get("AEON_DEV_HMAC_KEY", "aeon-dev-signing-key-baseline")
    want = hashlib.sha256((key + digest).encode()).hexdigest()
    got = sig.get("signature_hex", "")
    if len(want) != len(got):
        return False, "signature length mismatch"
    diff = 0
    for a, b in zip(want.encode(), got.encode()):
        diff |= a ^ b
    return diff == 0, "signature verified" if diff == 0 else "signature mismatch"


def verify_package(pkg: Path) -> dict:
    problems = []
    mf_path = pkg / "package.manifest.v2.json"
    if not mf_path.exists():
        return {"path": str(pkg), "result": "FAIL", "problems": ["package.manifest.v2.json missing"]}
    try:
        manifest = json.loads(mf_path.read_text())
    except Exception as e:
        return {"path": str(pkg), "result": "FAIL", "problems": [f"manifest unreadable: {e}"]}

    # 1. Schema
    schema = load_schema()
    errs = schema_validate(schema, manifest)
    if errs:
        return {"path": str(pkg), "result": "FAIL", "problems": [f"schema: {e}" for e in errs]}

    # 2. Signature
    ok, why = verify_signature(manifest)
    if not ok:
        return {"path": str(pkg), "result": "FAIL", "problems": [f"signature: {why}"]}

    # 3. Every listed file present with correct size and digest.
    listed_paths = set()
    for f in manifest["files"]:
        rel = f["path"]
        if os.path.isabs(rel):    problems.append(f"absolute path: {rel}")
        if ".." in Path(rel).parts: problems.append(f"path traversal: {rel}")
        if rel in listed_paths:   problems.append(f"duplicate path: {rel}")
        listed_paths.add(rel)
        p = pkg / rel
        if not p.is_file() or p.is_symlink():
            problems.append(f"missing / not-a-file / symlink: {rel}"); continue
        b = p.read_bytes()
        if len(b) != f["size"]:
            problems.append(f"size mismatch: {rel} expected {f['size']} got {len(b)}")
        got = hashlib.sha256(b).hexdigest()
        if got != f["sha256"]:
            problems.append(f"digest mismatch: {rel} expected {f['sha256']} got {got}")

    # 4. No unlisted protected file.
    for root, _, files in os.walk(pkg):
        for name in files:
            rel = str(Path(root).joinpath(name).relative_to(pkg))
            if rel == "package.manifest.v2.json":
                continue
            if rel not in listed_paths:
                problems.append(f"unlisted protected file: {rel}")

    try:
        rel_path = str(pkg.relative_to(REPO))
    except ValueError:
        rel_path = str(pkg)
    return {
        "path": rel_path,
        "profile": manifest.get("profile"),
        "source_commit": manifest.get("source_commit"),
        "files_verified": len(listed_paths),
        "result": "PASS" if not problems else "FAIL",
        "problems": problems,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--pkg", action="append", default=[])
    ap.add_argument("--out-root", default="target/deploy")
    args = ap.parse_args()
    root = REPO / args.out_root
    pkgs = []
    if args.all:
        for p in PROFILES:
            d = root / p
            if d.is_dir(): pkgs.append(d)
    else:
        for p in args.pkg:
            pkgs.append(Path(p).resolve())
    if not pkgs:
        print("no packages found", file=sys.stderr); return 2

    results = {"source_commit": head(), "packages": []}
    fails = 0
    for pkg in pkgs:
        r = verify_package(pkg)
        results["packages"].append(r)
        print(f"[verify] {r['path']}: {r['result']}")
        if r["result"] != "PASS":
            fails += 1
            for p in r["problems"][:5]:
                print(f"    - {p}", file=sys.stderr)
    EVID.mkdir(parents=True, exist_ok=True)
    (EVID / "package-integrity-results.json").write_text(
        json.dumps(results, indent=2, sort_keys=True))
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
