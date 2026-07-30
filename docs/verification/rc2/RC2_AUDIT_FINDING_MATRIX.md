# RC2 Audit-Finding Closure Matrix

Every row is initially **OPEN**. Closure requires at minimum
`INTERNALLY_VERIFIED` (evidence exists AND the remediating test would
fail without the fix). `INDEPENDENTLY_VERIFIED` is required before any
RC2 release claim.

Allowed statuses (per RC2 directive §10.1):

```
OPEN | IMPLEMENTED | INTERNALLY_VERIFIED | INDEPENDENTLY_VERIFIED | REJECTED
```

A finding may **not** be marked closed at `IMPLEMENTED`.

## Matrix

| # | Original audit severity | Reproduction command | Current result | Affected files | Remediation owner / module | Required tests | Closure evidence | Status |
|---|---|---|---|---|---|---|---|---|
| 1 | Release-blocking | `grep -nA3 "(7) signature verification" secure-relay/src/gateway.rs` | verify against caller-supplied digest | `secure-relay/{gateway,signing}.rs`, `contracts/src/relay.rs` | secure-relay | envelope-mutation matrix (payload, digest, dest, kind, timestamps, sender, nonce, seq, classification, releasability, schema version, key id, alg); truncated / oversized / unknown-variant / unknown-field / invalid canonical encoding; replay across process restart; wrong key / revoked key / missing / malformed signature | `docs/evidence/rc2/relay-integrity/` | OPEN |
| 2 | Release-blocking | `grep -nE "\\|\\| true\|no migrations yet\|would run" tools/verify.sh` | 7 hits | `tools/verify.sh`, CI workflow | tooling | negative-path suite: clippy fail, property fail, integration fail, replay divergence, missing package, missing security tool, invalid SBOM, migration fail, scope-scan fail, evidence contradiction | `docs/evidence/rc2/verification/VERIFY_ALL_RESULTS.json`, `.../VERIFY_ALL_REPORT.md` | OPEN |
| 3 | Release-blocking | `grep -nE "manifest.sha256\|PROFILE\|VERSION\|package.manifest.json" tools/deployment/build-package.sh` | identity files written after manifest.sha256 | `tools/deployment/build-package.sh`, `deployment/_common/scripts/install.sh`, manifest schema | deployment | tamper matrix on every protected file; missing / added / duplicate / path-traversal / symlink; wrong profile / wrong commit / expired signing policy | `docs/evidence/rc2/deployment/package-integrity/` | OPEN |
| 4 | Release-blocking | inspect `operator-api/src/routes.rs::ack_alert` + `main.rs` | actor from body; hardcoded Ready | `operator-api/**`, new `operator-identity/` module | operator-api | anonymous request, expired token, wrong issuer, wrong audience, insufficient role, revoked identity, forged actor field, authorization bypass, audit persistence failure, audit tampering, readiness with missing dep | `docs/evidence/rc2/operator-auth/` | OPEN |
| 5 | Release-blocking | inspect frozen `docs/evidence/releases/aeon-air-defense-rc1/gate-10/TOOLCHAIN_VERSIONS.json` | contradicts frozen `GATE_10_REPORT.md` | RC1 evidence frozen — RC2 rebuilds under `docs/evidence/rc2/gate-10/` | tooling | evidence-consistency validator (raw ↔ summary ↔ markdown ↔ toolchain lock) | `docs/evidence/rc2/gate-10/` | OPEN |
| 6 | Release-blocking | `./tools/scope_boundary_scan.sh` | 4 violations in `release/AEON_AIR_DEFENSE_RC1_MANIFEST.{json,schema.json}` | scanner, manifest key names, or scanner scoping | tooling + release engineering | scanner run over: production source, schemas, public API, deployment config, release manifests; each must be a distinct pass | scanner log per scope in `docs/evidence/rc2/scope-boundary/` | OPEN |
| 7 | Release-blocking | inspect `contracts/src/relay.rs::RelayEnvelope.payload_json: serde_json::Value` | free-form payload | `contracts/src/relay.rs`, `secure-relay/**` | contracts + secure-relay | typed payload enum with schema validation per variant; unknown-variant/unknown-field/oversize rejected | co-located with RC2-A | OPEN |
| 8 | Release-blocking | `grep api_checked docs/evidence/gate-11/*/fresh-install-health.json` | `false` | `tools/deployment/test-profile.sh`, deployment harness | deployment | full boot-and-drive-service lifecycle: install → start → ingest → verify tracks via HTTP → exercise relay → audit event → restart → recovery → uninstall | `docs/evidence/rc2/deployment/lifecycle/` | OPEN |
| 9 | Release-blocking | `grep -n "Uuid::new_v4\|HashMap" contracts/src/ids.rs track-management/src/lib.rs` | random UUIDs; unordered HashMap | `contracts/src/ids.rs`, `track-management/src/lib.rs`, `simulation/src/determinism.rs` | contracts + track-management + simulation | full-state digest that includes track ids, kinematic state, uncertainty, confidence, classification hypotheses, source contributions, rejected observations, alerts, relay decisions, audit events, config/policy ids; run in-process twice, across restart, on different thread schedules | `docs/evidence/rc2/replay/` | OPEN |
| 10 | Release-blocking | inspect `operator-api/src/main.rs::main` composition | ApiState is a mock | new `aeon-runtime/` binary composing every subsystem | runtime | end-to-end: adapter → normalization → persistence → track update → correction → authenticated API → typed relay → persistent audit | `docs/evidence/rc2/runtime/` | OPEN |

## Reproduction commands

All reproductions above are shell-runnable from the repo root at
commit `a770f1abc17e3e5d55226e701fbf5e663770aaa6`. When a finding
becomes `INTERNALLY_VERIFIED`, the "current result" column must be
updated to the new (fixed) output, and the closure-evidence path
must contain a machine-readable JSON showing:

- the original failure was reproduced,
- the fix was applied,
- the reproduction now passes,
- the reproduction was rerun against a synthetic regression that
  would fail without the fix.

## Independent verification

Marking a finding `INDEPENDENTLY_VERIFIED` requires review by a
different reviewer than the implementer, with the reviewer executing
the reproduction command against the fixed source and recording the
result in `docs/evidence/rc2/independent-audit/FINDING_CLOSURE_INDEX.json`.
