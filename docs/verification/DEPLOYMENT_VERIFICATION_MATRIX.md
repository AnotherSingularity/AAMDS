# Deployment Verification Matrix

Per directive section 13. Every profile is honestly labelled; the split
between REPOSITORY_VERIFIED (mechanically tested in this repo's CI) and
SPONSOR_VALIDATION_REQUIRED (target-environment tests only the sponsor
can perform) is preserved rather than collapsed to a single "green".

## Allowed statuses

| Status | Meaning |
|---|---|
| `VERIFIED` | Package produced + manifest validated + full harness cycle passed in a target-representative environment. |
| `REPOSITORY_VERIFIED` | Package built, manifest validated, harness cycle passed **inside this repository's CI containers**. Target-hardware validation still required. |
| `REFERENCE_ONLY` | Profile scaffolding is present but end-to-end run not yet driven. |
| `SPONSOR_VALIDATION_REQUIRED` | Cannot be mechanically closed in this repository — requires authorized target equipment, security controls, or personnel. |
| `FAILED` | Harness run failed. Must be repaired before release. |

## Per-profile matrix

| Profile | Package built | Manifest validated | Fresh install | Health | Upgrade | Rollback | Backup | Restore | Offline install | Uninstall | Host env | Remaining sponsor dep | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| developer      | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | none | **REPOSITORY_VERIFIED** |
| edge           | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | edge-hardware validation, watchdog integration | **REPOSITORY_VERIFIED** |
| disconnected   | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container, network-isolated env branch | true air-gap validation on sponsor hardware | **REPOSITORY_VERIFIED** |
| fixed-site     | ✅ (scripts) | ✅ (scripts) | — | — | — | — | — | — | — | — | not exercised in CI | redundancy, Postgres-compatible persistence, HA relay | **SPONSOR_VALIDATION_REQUIRED** |
| data-center    | ✅ (scripts) | ✅ (scripts) | — | — | — | — | — | — | — | — | not exercised in CI | central IdP integration, central observability, DB backup automation | **SPONSOR_VALIDATION_REQUIRED** |
| private-cloud  | ✅ (scripts) | ✅ (scripts) | — | — | — | — | — | — | — | — | not exercised in CI | infrastructure-as-code, network policy, KMS/HSM, horizontal scaling | **SPONSOR_VALIDATION_REQUIRED** |

Evidence for the three REPOSITORY_VERIFIED profiles is under
`docs/evidence/gate-11/{developer,edge,disconnected}/SUMMARY.json` and the
per-op JSON files alongside.

## Reproduction

```
./tools/deployment/build-package.sh developer          # or edge / disconnected
./tools/deployment/test-profile.sh developer full-cycle
```
