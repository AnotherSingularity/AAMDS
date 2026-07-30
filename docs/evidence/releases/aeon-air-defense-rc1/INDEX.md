# Aeon Air Defense — Release-Candidate Evidence (RC1)

Every file below is a snapshot from the RC1 verification run. Every
JSON file in this tree is stamped with the `source_commit` and
`release_id` fields for traceability. The source commit and full
release manifest are at `release/AEON_AIR_DEFENSE_RC1_MANIFEST.json`.

## Contents

| Path | Purpose |
|---|---|
| `gate-10/GATE_10_REPORT.md` | Gate 10 summary |
| `gate-10/TOOLCHAIN_VERSIONS.json` | Installed vs. pinned tool versions |
| `gate-10/DEPENDENCY_AUDIT.json` | Raw `cargo audit --json` |
| `gate-10/DEPENDENCY_AUDIT_DISPOSITIONED.json` | Cross-referenced against disposition ledger |
| `gate-10/SECRET_SCAN.json` | Raw `gitleaks detect --report-format json` |
| `gate-10/STATIC_ANALYSIS.json` | Clippy summary + raw JSON pointer |
| `gate-10/UNSAFE_CODE_INVENTORY.txt` | grep — every crate `#![forbid(unsafe_code)]` |
| `gate-10/SBOM_INDEX.json` | 10 CycloneDX SBOMs, 10/10 valid |
| `gate-10/sbom/*.cdx.json` | Per-crate CycloneDX 1.3 SBOMs |
| `gate-10/SECURITY_POLICY_RESULTS.json` | Consolidated Gate 10 policy summary |
| `gate-11/<profile>/SUMMARY.json` | Per-profile deployment SUMMARY (developer / edge / disconnected: `all_pass=true`) |
| `gate-11/<profile>/package-integrity.json` | Present for all six profiles |
| `gate-11/<profile>/{fresh-install,upgrade,rollback,backup-restore,offline-install,uninstall,configuration-validation}.json` | Present for the three REPOSITORY_VERIFIED profiles |

Cross-reference:

- Full release manifest: `../../release/AEON_AIR_DEFENSE_RC1_MANIFEST.json`
- Contract freeze: `../../interface-control-documents/RC1_CONTRACT_FREEZE.md`
- Deployment matrix: `../../docs/verification/DEPLOYMENT_VERIFICATION_MATRIX.md`
- Release-candidate report: `../../docs/verification/RELEASE_CANDIDATE.md`
- Sponsor handoff index: `../../deployment/sponsor-validation/HANDOFF_INDEX.md`
- Known limitations: `../../docs/verification/KNOWN_LIMITATIONS.md`

## Reproduction

Every command that produced this evidence is enumerated in
`release/AEON_AIR_DEFENSE_RC1_MANIFEST.json.evidence_generation_commands`.

Files in this directory MUST NOT be edited. Any subsequent
regeneration goes into a new `docs/evidence/releases/aeon-air-defense-rcN/`
folder for a new release candidate.
