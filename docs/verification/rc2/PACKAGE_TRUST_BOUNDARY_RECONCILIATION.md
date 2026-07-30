# Package Trust-Boundary Reconciliation

Audit finding 3: RC1's `tools/deployment/build-package.sh` computed
`manifest.sha256` **before** writing `PROFILE`, `COMMIT`, `VERSION`,
and `package.manifest.json`. Those identity files were therefore
outside the checksum boundary and not authenticated by
`install.sh`.

## RC1 order (rejected)

```
build      cargo release
copy       bin/ ui/ config/ scripts/ docs/ sbom/
checksum   find … -not -name manifest.sha256 -not -name package.manifest.json \
             → manifest.sha256                                        # ← boundary drawn
identity   PROFILE / COMMIT / VERSION written NOW                     # ← outside boundary
manifest   package.manifest.json written NOW                          # ← outside boundary
sign       dev-hmac over the JSON manifest                            # ← manifest self-signed but not covered
                                                                        by manifest.sha256
install    (cd pkg && sha256sum -c manifest.sha256)                   # ← misses PROFILE / COMMIT /
                                                                        VERSION / package.manifest.json
```

Files outside `manifest.sha256` at RC1: `PROFILE`, `COMMIT`,
`VERSION`, `package.manifest.json`. Any of them can be replaced
without failing `install.sh`.

## RC2 order (used by `tools/package/build.py`)

```
1.  cargo build --release --workspace
2.  stage: bin/ ui/ config/ scripts/ docs/ sbom/
3.  identity: PROFILE / COMMIT / VERSION written NOW
    (before any hashing — inside the boundary)
4.  compute canonical v2 manifest:
      every file under the package (except package.manifest.v2.json
      itself) is listed with {path, size, sha256, content_role,
      executable, required}, sorted by path.
5.  sign manifest:
      canonical = json.dumps(manifest_minus_signature,
                             sort_keys=True, separators=(",",":"))
      digest = SHA-256(canonical)
      signature_hex = SHA-256(KEY || digest)   # dev-hmac; sponsor swaps for kms-hsm
6.  emit package.manifest.v2.json (contains the signature)
```

Install (`tools/package/install.py`):

```
verify → temp staging dir → atomic promotion to $AEON_HOME
```

The verifier (`tools/package/verify.py`) is invoked **before** any
file is copied. Verification steps:

1. schema-validate the v2 manifest;
2. verify the signature over the canonical manifest bytes;
3. verify every listed file's presence + size + SHA-256;
4. reject any file present under the package that is not listed in the manifest;
5. reject `..`, absolute paths, duplicate paths;
6. reject symlinks (the tamper suite includes a symlink-escape case).

If any step fails, no file is copied to `$AEON_HOME`.

## Files now protected

Every one of the 63 shipped files per profile — including `PROFILE`,
`COMMIT`, `VERSION`, `bin/aeon-operator-api`, `scripts/*.sh`, every
`config/*.json`, every `sbom/*.cdx.json`, every doc file — is listed
in the v2 manifest and covered by the signature.

`package.manifest.v2.json` itself is not listed (a file cannot cover
itself); its integrity is instead protected by the signature it
carries.

## Adversarial coverage

21 tamper cases in `tools/package/negative.py` — 21/21 rejected by the
verifier or installer:

```
01 modified PROFILE            12 duplicate manifest path
02 modified COMMIT             13 absolute manifest path
03 modified VERSION            14 parent-directory traversal
04 modified manifest (unsigned edit)   15 symlink escape
05 modified executable         16 invalid signature (bit-flip)
06 modified installer script   17 wrong signer (different HMAC key)
07 modified upgrade script     18 wrong profile field
08 modified SBOM               19 wrong source_commit
09 added unlisted executable   20 installer refuses tampered pkg
10 added unlisted config       21 baseline positive control
11 removed listed file
```

Cross-profile: all six profiles (developer, edge, disconnected,
fixed-site, data-center, private-cloud) build + verify PASS.

## Sponsor-side substitution

`signature.method = dev-hmac-sha256` is a non-production baseline.
Sponsor deployments substitute `kms-hsm`; the manifest's
`signature.over` field is `canonical-manifest-minus-signature`, so
the signing algorithm can change while the signed bytes stay stable.
