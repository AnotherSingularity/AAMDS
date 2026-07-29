# Acceptance Plan

The 12 acceptance gates from section 28 of the implementation directive.
Each gate names its mechanical verification.

| # | Gate | Mechanical verification |
|---|---|---|
| 1 | Repository integrity | `git log --format=%s` on the additive commit sequence + `docs/implementation/STARTING_STATE.md` |
| 2 | Contract integrity | `cargo test -p aeon-contracts` |
| 3 | Runtime integrity | `cargo test -p aeon-core-runtime` (lifecycle, invalid-config-blocks-readiness, restart-recovery) |
| 4 | Adapter integrity | `cargo test -p aeon-sensor-adapter-sdk` (conformance harness) |
| 5 | Track integrity | `cargo test -p aeon-track-management` + deterministic replay |
| 6 | ML safety | `cargo test -p aeon-ml-correction` (bounded, OOD, rollback) |
| 7 | Relay security | `cargo test -p aeon-secure-relay` (allowlist, prohibited-content, anti-replay) |
| 8 | Scope boundary | `tools/verify.sh scope-boundary` + `RelayMessageKind` compile-time enumeration test |
| 9 | Determinism | `tools/verify.sh replay` — reruns fixture N times and diffs the trace |
| 10 | Cybersecurity | `tools/verify.sh security` + `tools/verify.sh sbom` |
| 11 | Deployment | manual per-profile smoke via `deployment/*/build.sh` |
| 12 | Integration readiness | ICD set complete, adapter can be authored from `interface-control-documents/SENSOR_ADAPTER_ICD.md` |

`tools/verify.sh all` chains every mandatory step. A missing tool is a
clear error, never a silent skip.
