#!/usr/bin/env python3
"""Validate a deployment package manifest against the JSON schema and
verify the dev-hmac signature. Returns non-zero on any failure."""

from __future__ import annotations
import argparse
import hashlib
import json
import os
import sys


def validate(schema, obj, path=""):
    """Minimal JSON-schema subset validator: type, required, additionalProperties=false, enum, minimum, minItems, pattern."""
    import re
    errors = []

    def err(msg):
        errors.append(f"{path or '/'}: {msg}")

    t = schema.get("type")
    if t == "object":
        if not isinstance(obj, dict):
            err(f"expected object, got {type(obj).__name__}")
            return errors
        for req in schema.get("required", []):
            if req not in obj:
                err(f"missing required field {req!r}")
        props = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            for k in obj:
                if k not in props:
                    err(f"unexpected field {k!r}")
        for k, v in obj.items():
            if k in props:
                errors.extend(validate(props[k], v, path + "." + k))
    elif t == "array":
        if not isinstance(obj, list):
            err(f"expected array, got {type(obj).__name__}")
            return errors
        if "minItems" in schema and len(obj) < schema["minItems"]:
            err(f"expected at least {schema['minItems']} items")
        for i, item in enumerate(obj):
            errors.extend(validate(schema.get("items", {}), item, path + f"[{i}]"))
    elif t == "string":
        if not isinstance(obj, str):
            err("expected string")
        if "pattern" in schema and isinstance(obj, str):
            if not re.match(schema["pattern"], obj):
                err(f"pattern mismatch: {schema['pattern']}")
    elif t == "integer":
        if not isinstance(obj, int):
            err("expected integer")
        if "minimum" in schema and isinstance(obj, int) and obj < schema["minimum"]:
            err(f"below minimum {schema['minimum']}")
    elif t == "number":
        if not isinstance(obj, (int, float)):
            err("expected number")

    if "enum" in schema and obj not in schema["enum"]:
        err(f"value not in enum {schema['enum']}")
    return errors


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--schema", required=True)
    args = ap.parse_args()

    schema = json.load(open(args.schema))
    manifest = json.load(open(args.manifest))
    errs = validate(schema, manifest)
    if errs:
        for e in errs:
            print(f"schema: {e}", file=sys.stderr)
        return 1
    # verify dev-hmac signature
    sig = manifest.get("signature", {})
    if sig.get("method") == "dev-hmac-sha256":
        canonical = json.dumps({k: v for k, v in manifest.items() if k != "signature"},
                               sort_keys=True, separators=(",", ":"))
        digest = hashlib.sha256(canonical.encode()).hexdigest()
        key = os.environ.get("AEON_DEV_HMAC_KEY", "aeon-dev-signing-key-baseline")
        want = hashlib.sha256((key + digest).encode()).hexdigest()
        if want != sig.get("signed_manifest_digest"):
            print("signature: dev-hmac verify FAILED", file=sys.stderr)
            return 1
        print(f"manifest: {args.manifest} ok (dev-hmac verified)")
    else:
        print(f"manifest: {args.manifest} ok ({sig.get('method')})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
