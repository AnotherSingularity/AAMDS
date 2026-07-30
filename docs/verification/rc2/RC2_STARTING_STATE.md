# RC2 Starting State

## Branch state

- Branch: `claude/aeon-air-defense-rc2-remediation`
- Base commit: `a770f1abc17e3e5d55226e701fbf5e663770aaa6`
- RC1 frozen commit: `d1b8414181bc164b426ef23e4f591ec5c3c5eeb7`
- Working tree: clean at the RC2 base
- RC1 tag object: `8bc2edac76afbb75ded176c2f39c717e39784297` (preserved
  in `release/rc1-tag-preservation/`)

## RC1 evidence integrity

The RC1 evidence directories under
`docs/evidence/releases/aeon-air-defense-rc1/` and the release
artefacts under `release/AEON_AIR_DEFENSE_RC1_MANIFEST.json`,
`release/AEON_AIR_DEFENSE_RC1_MANIFEST.schema.json`,
`release/AEON_AIR_DEFENSE_RC1_REJECTED.txt`, and
`release/rc1-tag-preservation/` are **frozen**. RC2 remediation may
not modify them.

## Audit findings — RC2 reproduction

Every finding was re-reproduced against the RC2 base commit before
this record was written. All ten remain OPEN at RC2-0.

| # | Finding | Reproduction on RC2 base | Status |
|---|---|---|---|
| 1 | Relay signature verifies only `payload_digest_hex`; `canonical_envelope_digest` is defined but not called from `submit()`. | `secure-relay/src/gateway.rs:149–152` verifies against `&envelope.payload_digest_hex`. | OPEN |
| 2 | `verify:all` fail-open (`\|\| true` on lint/property/integration/e2e; `no migrations yet`; `would run` for packages). | `tools/verify.sh:32,36,43,93,121,122,123`. | OPEN |
| 3 | `PROFILE`/`COMMIT`/`VERSION`/`package.manifest.json` written after `manifest.sha256` (excluded from the checksum). | `tools/deployment/build-package.sh:81–82` (find excludes manifest files), lines 148–156 write identity files after. | OPEN |
| 4 | Operator API has no authentication/authorization; readiness asserted. | `operator-api/src/routes.rs::ack_alert` uses caller-supplied actor; `operator-api/src/main.rs` sets `RuntimeState::Ready` without probes. | OPEN |
| 5 | Frozen `TOOLCHAIN_VERSIONS.json` says `cargo-audit expected 9.99.9 status version_mismatch` while `GATE_10_REPORT.md` claims PASS. | `docs/evidence/releases/aeon-air-defense-rc1/gate-10/TOOLCHAIN_VERSIONS.json` — file preserved intact. | OPEN (RC1 evidence frozen; RC2 will regenerate under `docs/evidence/rc2/gate-10/`) |
| 6 | `tools/scope_boundary_scan.sh` FAILS on the release tree because `release/AEON_AIR_DEFENSE_RC1_MANIFEST.{json,schema.json}` contain literal `no_firing_solution` / `no_aimpoint_or_engagement`. | Run of scanner produces 4 violations at RC2 base. | OPEN |
| 7 | Scope boundary is structurally bypassable: `RelayEnvelope.payload_json` is free-form `serde_json::Value`; prohibited scan is substring word-match. | `contracts/src/relay.rs::RelayEnvelope.payload_json: serde_json::Value`. | OPEN |
| 8 | Gate 11A never proves the service runs — `api_checked: false` in fresh-install-health. | `docs/evidence/gate-11/developer/fresh-install-health.json`. | OPEN |
| 9 | Determinism not established — `TrackId::new` uses `Uuid::new_v4`; `TrackEngine.tracks` is `HashMap`; `trace_digest` excludes non-deterministic fields. | `contracts/src/ids.rs:14`, `track-management/src/lib.rs:77,81`, `simulation/src/determinism.rs`. | OPEN |
| 10 | Packaged executable `aeon-operator-api` is a mock — `main.rs` builds `ApiState` with empty engine, hard-coded `Ready`, no adapters/persistence/relay/audit composition. | `operator-api/src/main.rs`. | OPEN |

## Toolchain observed at RC2 base

- rustc 1.94.1 stable
- cargo-audit 0.22.2 (installed in `.aeon-tools/bin/`; toolchain lock pins this)
- cargo-cyclonedx 0.5.7
- gitleaks 8.28.0
- sqlite3 3.45.1

## RC2 scope

Remediation of all ten audit findings additively, with the closure
matrix in `RC2_AUDIT_FINDING_MATRIX.md`. No finding is closed at
`IMPLEMENTED` — closure requires `INTERNALLY_VERIFIED` at minimum,
`INDEPENDENTLY_VERIFIED` before any release claim.

RC2 must not touch RC1 evidence. RC2 may not restore the previous
release rating by documentation alone; it must correct the
underlying implementation and mechanically prove every corrected
claim.
