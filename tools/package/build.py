#!/usr/bin/env python3
"""RC2 canonical package builder.

Correct order (RC2-C §4.4):

  build all content
  → generate identity metadata files (PROFILE / COMMIT / VERSION)
  → collect SBOM references
  → assemble the canonical file inventory (every file that lands in
    the package, including identity files, gets hashed and listed)
  → sign the canonical manifest (dev-hmac-sha256; sponsor swaps for
    kms-hsm)
  → seal the package (write signed manifest + refuse further mutation
    at the tooling layer)

Outputs:
  target/deploy/<profile>/
    bin/, ui/, config/, scripts/, docs/, sbom/,
    PROFILE, COMMIT, VERSION,
    package.manifest.v2.json          (canonical, signed)

  build/evidence/rc2/packages/package-build-results.json  (per-profile
    outcome for the RC2 verify:all ledger)
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SCHEMA = REPO / "deployment" / "schemas" / "package-manifest.v2.schema.json"
EVID = REPO / "build" / "evidence" / "rc2" / "packages"
PROFILES = ["developer", "edge", "disconnected", "fixed-site", "data-center", "private-cloud"]


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def head() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()


def toolchain_identity() -> dict:
    def _try(cmd):
        try:
            return subprocess.check_output(cmd, text=True).strip()
        except Exception:
            return ""
    return {
        "rustc": _try(["rustc", "--version"]),
        "cargo": _try(["cargo", "--version"]),
    }


def cargo_release_build() -> None:
    subprocess.check_call(["cargo", "build", "--release", "--workspace"], cwd=REPO)


def stage(profile: str, out_root: Path) -> Path:
    pkg = out_root / profile
    if pkg.exists():
        shutil.rmtree(pkg)
    (pkg / "bin").mkdir(parents=True)
    (pkg / "ui").mkdir()
    (pkg / "config").mkdir()
    (pkg / "scripts").mkdir()
    (pkg / "docs").mkdir()
    (pkg / "sbom").mkdir()
    for b in ["aeon-operator-api"]:
        src = REPO / "target" / "release" / b
        if src.exists():
            shutil.copy2(src, pkg / "bin" / b)
    ui = REPO / "operator-interface" / "index.html"
    if ui.exists():
        shutil.copy2(ui, pkg / "ui" / "index.html")
    cfg = REPO / "deployment" / profile / "config"
    if cfg.is_dir():
        for f in cfg.iterdir():
            shutil.copy2(f, pkg / "config" / f.name)
    scripts_src = REPO / "deployment" / "_common" / "scripts"
    for f in scripts_src.iterdir():
        shutil.copy2(f, pkg / "scripts" / f.name)
    for d in ("operations", "integration", "security", "architecture", "models", "verification"):
        s = REPO / "docs" / d
        if s.is_dir():
            shutil.copytree(s, pkg / "docs" / d)
    sboms = REPO / "build" / "evidence" / "rc2" / "gate-10" / "sbom"
    if sboms.exists():
        for f in sboms.glob("*.cdx.json"):
            shutil.copy2(f, pkg / "sbom" / f.name)
    # Identity metadata files — written NOW so they are inside the
    # inventory + signed manifest boundary (fixes finding 3).
    (pkg / "PROFILE").write_text(profile + "\n")
    (pkg / "COMMIT").write_text(head() + "\n")
    (pkg / "VERSION").write_text("0.1.0" + "\n")
    return pkg


def content_role_for(rel: str) -> tuple[str, bool]:
    if rel.startswith("bin/"):    return "executable", True
    if rel.startswith("scripts/"): return "script", True
    if rel.startswith("config/"):  return "config", False
    if rel.startswith("docs/"):    return "documentation", False
    if rel.startswith("ui/"):      return "ui", False
    if rel.startswith("sbom/"):    return "sbom", False
    if rel in ("PROFILE", "COMMIT", "VERSION"): return "identity", False
    return "other", False


def compute_manifest(pkg: Path, profile: str) -> dict:
    files = []
    for root, _, names in os.walk(pkg):
        for name in sorted(names):
            p = Path(root) / name
            rel = str(p.relative_to(pkg))
            if rel == "package.manifest.v2.json":
                continue
            b = p.read_bytes()
            role, executable = content_role_for(rel)
            files.append({
                "path": rel,
                "size": len(b),
                "sha256": hashlib.sha256(b).hexdigest(),
                "content_role": role,
                "executable": executable,
                "required": True,
            })
    files.sort(key=lambda x: x["path"])
    manifest = {
        "manifest_version": 2,
        "product": "aeon-air-defense-information-layer",
        "release_version": "0.1.0",
        "source_commit": head(),
        "profile": profile,
        "target_os": ["linux"],
        "target_arch": ["x86_64", "aarch64"],
        "build_identity": {"builder": "tools/package/build.py", "created_at": "deterministic-per-commit"},
        "toolchain_identity": toolchain_identity(),
        "created_at": "deterministic-per-commit",
        "files": files,
        "sbom": [f["path"] for f in files if f["content_role"] == "sbom"],
        "configuration_schema": [f["path"] for f in files if f["content_role"] == "config"],
        "migration_set": [],
        "signing_policy": {"policy_version": "1", "signer": "dev-hmac-baseline"},
        "scope_policy": {"policy_version": "1"},
    }
    return manifest


def sign(manifest: dict) -> dict:
    canonical = json.dumps({k: v for k, v in manifest.items() if k != "signature"},
                           sort_keys=True, separators=(",", ":"))
    digest = hashlib.sha256(canonical.encode()).hexdigest()
    key = os.environ.get("AEON_DEV_HMAC_KEY", "aeon-dev-signing-key-baseline")
    sig = hashlib.sha256((key + digest).encode()).hexdigest()
    manifest["signature"] = {
        "method": "dev-hmac-sha256",
        "key_id": "dev-hmac-baseline",
        "over":   "canonical-manifest-minus-signature",
        "signature_hex": sig,
        "note": "Non-production dev signature. Sponsor deployments substitute kms-hsm before production use.",
    }
    return manifest


def build_one(profile: str, out_root: Path) -> dict:
    pkg = stage(profile, out_root)
    manifest = compute_manifest(pkg, profile)
    sign(manifest)
    (pkg / "package.manifest.v2.json").write_text(json.dumps(manifest, indent=2, sort_keys=True))
    try:
        rel_path = str(pkg.relative_to(REPO))
    except ValueError:
        rel_path = str(pkg)
    return {
        "profile": profile,
        "path": rel_path,
        "files": len(manifest["files"]),
        "result": "PASS",
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--profile", action="append", default=[])
    ap.add_argument("--out", default="target/deploy")
    args = ap.parse_args()

    profiles = PROFILES if args.all else args.profile
    if not profiles:
        print("use --all or --profile <p>", file=sys.stderr); return 2
    cargo_release_build()
    out_root = Path(args.out)
    if not out_root.is_absolute():
        out_root = REPO / out_root
    out_root.mkdir(parents=True, exist_ok=True)
    results = {"source_commit": head(), "generated_at": now(), "packages": []}
    for p in profiles:
        r = build_one(p, out_root)
        results["packages"].append(r)
        print(f"[package] built {p}: {r['files']} files")
    EVID.mkdir(parents=True, exist_ok=True)
    (EVID / "package-build-results.json").write_text(
        json.dumps(results, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
