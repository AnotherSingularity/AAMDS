# Deployment Verification Matrix

Per directive section 13 + RC1 freeze directive section 1.

## Allowed statuses (RC1 freeze taxonomy)

| Status | Meaning |
|---|---|
| `REPOSITORY_VERIFIED` | Complete lifecycle mechanically tested in an isolated repository-controlled environment. |
| `PACKAGE_VERIFIED` | Package structure, manifest, integrity, and scripts verified in the repository CI environment, but realistic target operation requires sponsor infrastructure (HA persistence, IdP, KMS/HSM, orchestrator). |
| `REFERENCE_ONLY` | Documentation or templates exist, but the profile does not yet produce a complete installable package. |
| `SPONSOR_VALIDATION_REQUIRED` | Repository verification is complete and only authorized environmental validation remains. |
| `FAILED` | A mandatory repository-controlled verification failed. |

## Per-profile matrix (RC1)

Legend: ✅ mechanically verified in the repository CI-representative environment; ⏳ requires sponsor infrastructure; — not applicable at this profile.

| Profile | Package built | Manifest validated | Checksums | SBOM | Config schema | Install proc | Upgrade proc | Rollback proc | Health-check | Backup / restore | Offline install | Host env | Remaining sponsor dep | **Status** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| developer     | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | none | **REPOSITORY_VERIFIED** |
| edge          | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | edge-hardware / watchdog integration | **REPOSITORY_VERIFIED** |
| disconnected  | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container, network-isolated shell env | true air-gap validation on sponsor hardware | **REPOSITORY_VERIFIED** |
| fixed-site    | ✅ | ✅ | ✅ | ✅ | ✅ | scripts + procedure | scripts + procedure | scripts + procedure | scripts | scripts | — | linux/x86_64 CI container (package build only) | redundancy, Postgres-compatible persistence, HA relay, sponsor cluster | **PACKAGE_VERIFIED** |
| data-center   | ✅ | ✅ | ✅ | ✅ | ✅ | scripts + procedure | scripts + procedure | scripts + procedure | scripts | scripts | — | linux/x86_64 CI container (package build only) | central IdP integration, central observability, database backup automation | **PACKAGE_VERIFIED** |
| private-cloud | ✅ | ✅ | ✅ | ✅ | ✅ | scripts + procedure | scripts + procedure | scripts + procedure | scripts | scripts | — | linux/x86_64 CI container (package build only) | infrastructure-as-code, network policy templates, KMS/HSM integration, horizontal scaling | **PACKAGE_VERIFIED** |

## What "PACKAGE_VERIFIED" means for fixed-site / data-center / private-cloud

The three server-class profiles share the same build machinery
(`tools/deployment/build-package.sh`) and the same install / upgrade /
rollback / backup / restore / healthcheck / uninstall scripts
(`deployment/_common/scripts/`) as the three REPOSITORY_VERIFIED
profiles. For those three, the repository CI:

- builds the package,
- validates its manifest against `deployment/schemas/package-manifest.schema.json`,
- verifies every artifact's SHA-256 against the shipped `manifest.sha256`,
- signs the manifest with `dev-hmac-sha256` (non-production; sponsor
  substitutes `kms-hsm` at target),
- runs the boundary scanner over the shipped scripts.

What the repository does **not** verify for these three profiles:

- end-to-end install / upgrade / rollback against a redundant or
  clustered runtime,
- Postgres-compatible persistence migration and integrity walk,
- IdP-fronted operator API,
- KMS/HSM-signed outbound relay,
- HA failover, backup automation, or DR restore in a target environment.

These require the sponsor's target infrastructure; the
`deployment/sponsor-validation/` package documents the acceptance
procedure for each.

## Reproduction

```
./tools/deployment/build-package.sh <profile>
./tools/deployment/test-profile.sh <profile> package-integrity
./tools/deployment/test-profile.sh <profile> full-cycle    # only supported for developer/edge/disconnected
```

## Evidence

`docs/evidence/gate-11/{developer,edge,disconnected}/SUMMARY.json` —
`all_pass=true` for the full-cycle profiles.
`docs/evidence/gate-11/{fixed-site,data-center,private-cloud}/package-integrity.json` —
`PASS` for the package-verified profiles.
