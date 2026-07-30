# RC2 Audit-Finding Closure Matrix

Every row starts **OPEN** and may only advance to **INTERNALLY_VERIFIED**
when the remediating tests exist AND would fail without the fix.
**INDEPENDENTLY_VERIFIED** requires review by a second party against the
fixed source. Closure at `IMPLEMENTED` is prohibited.

## Current status

| # | Status | Notes |
|---|---|---|
| 1  | **OPEN** | Relay signature over caller-supplied `payload_digest_hex`; not remediated in this tranche. RC2-A closes. |
| 2  | **INTERNALLY_VERIFIED** | RC2-B (commit `6fe4aec` + integration `9e91bc8`). Fail-closed runner + versioned manifest + 5 negatives + real migration tests. |
| 3  | **INTERNALLY_VERIFIED** | RC2-C (commit `9b0b9f0`). v2 canonical manifest with every file inside the signature boundary + 21 tamper regressions. |
| 4  | **OPEN** | Operator API still unauthenticated. |
| 5  | **INTERNALLY_VERIFIED** | RC2-E (commit `545d135`). Raw-first gate-10 pipeline + consistency validator + 4 contradiction regressions. RC1 evidence preserved intact. |
| 6  | **INTERNALLY_VERIFIED** | RC2-F (commit `a4fc5b8`). Six-layer scanner + exclusions manifest + 6 negatives. Overall scope still FAIL — see finding 7. |
| 7  | **OPEN** | `RelayEnvelope.payload_json: serde_json::Value` unchanged. RC2-A closes. |
| 8  | **OPEN** | Deployment harness still doesn't boot the service. |
| 9  | **INTERNALLY_VERIFIED** | RC2-H (commit `8939750`). `DeterministicIdSource`, `BTreeMap`, expanded trace digest, cross-process replay. |
| 10 | **OPEN** | Packaged binary still the mock operator API. |

Five findings are now `INTERNALLY_VERIFIED` (2, 3, 5, 6, 9). Five
remain `OPEN` (1, 4, 7, 8, 10). Finding 6's closure means the
scanner and claim machinery are corrected — it does **not** promote
`OVERALL_SCOPE_BOUNDARY` to PASS while Finding 7 keeps
`typed_relay_boundary` at FAIL.

## Evidence

| Finding | Evidence artifact |
|---|---|
| 2 | `build/evidence/rc2/verification/verify-all-results.json`, `verify-all-report.md`, plus per-step logs under `logs/`. Negative suite: `tools/verify/tests/negative_paths.py` (5/5). |
| 3 | `build/evidence/rc2/packages/{package-build-results,package-integrity-results,package-negative-tests}.json`. 21/21 tamper regressions. |
| 5 | `build/evidence/rc2/gate-10/{toolchain-versions,dependency-audit,dependency-audit-dispositioned,secret-scan,sbom-index,gate-10-results,gate-10-report}.json/md`. Consistency negative: `tools/gate10/tests/consistency_negative.py` (4/4). |
| 6 | `build/evidence/rc2/scope/scope-results.json`, `scope-report.md`, `tools/scope/tests/negative_paths.py` (6/6). |
| 9 | `docs/evidence/rc2/replay/CROSS_PROCESS_DIGESTS.md` (committed) + `tools/verify/replay_cross_process.py` + `simulation/tests/rc2_determinism.rs` (4/4). |

## Directive-required verify:all outcome

`python3 tools/verify/run.py` on this commit:

```
format                      PASS
clippy                      PASS
build                       PASS
unit_and_property_tests     PASS
determinism_tests           PASS
replay_cross_process        PASS
migration_integrity         PASS
adversarial_persistence     PASS
security_toolchain          PASS
dependency_audit            PASS
secret_scan                 PASS
sbom                        PASS
gate10_consistency          PASS
scope_all                   FAIL   ← held FAIL by typed_relay_boundary=FAIL
                                     (Finding 7 open; RC2-A closes)
package_build_all           PASS
package_integrity_all       PASS
package_negative            PASS
documentation_presence      PASS
evidence_consistency        PASS
overall                     FAIL   ← correct: RC2 is not a release
```

## Rating

**RC2 REMEDIATION BUILD — INTERNAL VERIFICATION ONLY.**
This is the only claim permitted until every finding is
`INDEPENDENTLY_VERIFIED`.
