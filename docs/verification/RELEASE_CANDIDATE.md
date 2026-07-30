# Aeon Air Defense Information Layer — Release-Candidate Report

> **⛔ WITHDRAWN — RC1 REJECTED BY INDEPENDENT AUDIT.**
> The rating recorded in this file is **superseded** by
> [`RC1_AUDIT_REJECTION.md`](RC1_AUDIT_REJECTION.md). The correct rating
> for `d1b8414` is `ENGINEERING PROTOTYPE / LAB BASELINE — REJECTED BY
> INDEPENDENT AUDIT`. This file is preserved verbatim for audit
> traceability. Do **not** cite the rating below as current.


## Repository state

- **Branch**: `claude/aeon-air-defense-layer-wico33`
- **Starting SHA** (this continuation): `0156190e9227a58ba612f8a3db8d2f9ef771bb75`
- **Ending SHA** (before this handoff commit): `782720417f8c36335777cc1ce20474a18ef6a4a3`
- **Commit range**: baseline 15 commits (A–N) + 7 continuation commits (O, O-fix, P/Q, R/S, T, U). The Phase V/W handoff commit is atop this file.
- **Working-tree status**: clean at write time.
- **Remote push status**: baseline commits already pushed; continuation commits pushed at end of session.

## Commits added in this continuation

```
c76ee4c O: pin and automate the security verification toolchain
e7fd5f2 O-fix: exclude locally-built security tool binaries from tracking
af022d9 P: add vulnerability policy, secret scanning, and validated SBOM
                evidence
                (co-committed with Q: integrate mandatory security checks into CI)
90bf40a R: versioned deployment manifests + reproducible package builds
                (co-committed with S: install / upgrade / rollback / integrity harnesses)
ccdbf4f T: add backup, restore, offline installation, and negative-path tests
7827204 U: publish sponsor-controlled deployment acceptance package
```

## Security

| Item | Value |
|---|---|
| Toolchain lock | `security/toolchain.lock` |
| cargo-audit | 0.22.2 (installed, version-verified) |
| cargo-cyclonedx | 0.5.7 (installed, version-verified) |
| gitleaks | 8.28.0 (installed, version-verified) |
| rustfmt | tracks rust-toolchain (1.8.0-stable) |
| clippy | tracks rust-toolchain (0.1.94) |
| Dependency-audit result | 0 vulnerabilities across 143 dependencies |
| Secret-scan result | 0 findings across full history (21 commits, ~1 MB) |
| Static-analysis result | `cargo clippy --workspace --all-targets -- -D warnings` clean |
| SBOM result | 10 CycloneDX 1.3 JSON files, 10/10 valid |
| Vulnerability dispositions | 0 open, 0 expired (ledger empty because audit is clean) |
| Gate 10 evidence | `docs/evidence/gate-10/GATE_10_REPORT.md` |
| **Gate 10 decision** | **PASS** |

## Deployment

For each of the three REPOSITORY_VERIFIED profiles the full harness
cycle (package build + integrity + fresh install + config validation +
upgrade + rollback + backup/restore + offline install + uninstall) ran
green in the CI-representative container:

| Profile | Package | Manifest | Fresh install | Upgrade | Rollback | Backup/restore | Offline install | Env | Remaining sponsor dep |
|---|---|---|---|---|---|---|---|---|---|
| developer      | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | none |
| edge           | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | edge-hardware, watchdog |
| disconnected   | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | linux/x86_64 CI container | true air-gap validation |

The other three profiles (fixed-site, data-center, private-cloud) ship
scripts + manifests + config but require sponsor-owned targets to
verify end-to-end (IdP, HA persistence, KMS/HSM). They are marked
**SPONSOR_VALIDATION_REQUIRED**, not PASS.

Evidence: `docs/evidence/gate-11/{developer,edge,disconnected}/SUMMARY.json`.
Matrix: `docs/verification/DEPLOYMENT_VERIFICATION_MATRIX.md`.

Sponsor-validation package (12 documents): `deployment/sponsor-validation/`.

## Tests

| Bucket | Count |
|---|---|
| Baseline unit + property + integration + adversarial | **81** |
| New deployment negative-path regressions (`deployment_regression.sh`) | **8** |
| Total mechanical checks | **89** |
| Failures | **0** |
| Skips | **0** |
| Sponsor-required (Gate 11B) | documented in `deployment/sponsor-validation/`, not counted here |

## Scope boundary

Mechanically re-confirmed on the release-candidate commit:

- **No weapon-control interface exists.**
- **No engagement-computation implementation exists.**
- **No firing-solution implementation exists.**
- **No launch, guidance, or aimpoint command exists.**
- **Relay output is restricted** to the four informational variants
  in `RelayMessageKind` (`TrackState`, `ObservationSummary`,
  `SystemHealth`, `Alert`); enum size test still asserts count == 4.
- **Boundary scan** (`tools/scope_boundary_scan.sh`) — PASS.
- **Runtime content scan** (`aeon_contracts::prohibited::scan_json`)
  is applied on every outbound envelope inside
  `secure-relay/src/gateway.rs::submit`.

## Final gate decisions

```
Gate 10:            PASS
Gate 11A:           PASS
Gate 11B:           SPONSOR VALIDATION REQUIRED
Gate 11 overall:    PARTIAL (Gate 11A PASS, Gate 11B outstanding)
```

## Release-candidate rating

```
MILITARY INTEGRATION-READY RELEASE CANDIDATE: REPOSITORY VERIFIED
SPONSOR PLATFORM VALIDATION: REQUIRED
```

**Not** `MILITARY INTEGRATION-READY BASELINE: PASS` — that phrase is
reserved for a state that includes sponsor-controlled Gate 11B
evidence, which cannot be obtained inside this repository.

No claim of accreditation, deployment approval, government
certification, combat readiness, plug-and-play compatibility, or
FIPS validation is made or implied.
